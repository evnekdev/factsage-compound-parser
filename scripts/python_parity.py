#!/usr/bin/env python3
"""Emit aggregate-only comparison data from the original Python parser.

This script is deliberately separate from the Rust test suite: it requires a
local checkout of ``factsage-compound`` plus its Python dependencies, and it
never prints CDB record content. It does not call the Python project's save
method and does not modify the fixture.
"""

from __future__ import annotations

import argparse
import importlib.util
import math
from collections import Counter
from pathlib import Path
from types import ModuleType

import numpy as np


CHUNK_SIZE = 256
KNOWN_IDS = frozenset(range(1, 12))
CP_IDS = frozenset((2, 3, 4, 5, 6))
OLE_MIN = -657_435.0
OLE_MAX = 2_958_466.0


def load_python_parser(project_root: Path) -> ModuleType:
    """Load only the original parser module, avoiding optional Excel imports."""
    module_path = project_root / "src" / "factsage_compound" / "factsage_compound.py"
    if not module_path.is_file():
        raise FileNotFoundError(f"Python parser module not found: {module_path}")
    specification = importlib.util.spec_from_file_location("factsage_python_parser", module_path)
    if specification is None or specification.loader is None:
        raise RuntimeError(f"Could not load Python parser module: {module_path}")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def histogram(values: Counter[int]) -> str:
    return ",".join(f"{key}:{values[key]}" for key in sorted(values))


def phase_state(phase_id_raw: int) -> str:
    if phase_id_raw > 990:
        return "aqueous"
    if phase_id_raw > 900:
        return "gas"
    if phase_id_raw > 800:
        return "liquid"
    return "solid"


def ole_timestamp_is_invalid(value: float) -> bool:
    return not math.isfinite(value) or not (OLE_MIN < value < OLE_MAX)


def collect_summary(cdb_path: Path, parser: ModuleType) -> dict[str, str]:
    raw_bytes = cdb_path.read_bytes()
    if len(raw_bytes) % CHUNK_SIZE != 0:
        raise ValueError("CDB length is not divisible by 256")
    chunk_matrix = np.frombuffer(raw_bytes, dtype=np.uint8).reshape(-1, CHUNK_SIZE)
    chunk_ids = [int(value) for value in chunk_matrix[:, 0]]
    database = parser.Database(str(cdb_path))

    compounds = database.compounds
    phases = [phase for compound in compounds for phase in compound.phases]
    ranges = [range_ for phase in phases for range_ in phase.ranges]
    kappas = [kappa for phase in phases for kappa in phase.kappas]
    comments = [comment for compound in compounds for comment in compound.comment_chunks]

    energy_codes = Counter(int(compound.chunk["unit_energy"]) for compound in compounds)
    pressure_codes = Counter(int(compound.chunk["unit_pressure"]) for compound in compounds)
    state_counts = Counter(phase_state(int(phase.chunk["phase_id_raw"])) for phase in phases)

    duplicate_phase_ids = 0
    for compound in compounds:
        identifiers = Counter(int(phase.chunk["phase_id_raw"]) for phase in compound.phases)
        duplicate_phase_ids += sum(count - 1 for count in identifiers.values() if count > 1)

    cp_bounds_invalid = 0
    for index, chunk_id in enumerate(chunk_ids):
        if chunk_id not in CP_IDS:
            continue
        chunk = database.chunks[index].view(dtype=parser.dbtype_cp_chunk)
        t_min = float(chunk["t_min"])
        t_max = float(chunk["t_max"])
        cp_bounds_invalid += int(
            not math.isfinite(t_min) or not math.isfinite(t_max) or t_min > t_max
        )

    overlapping_sets = 0
    gap_sets = 0
    shared_endpoints = 0
    for phase in phases:
        intervals = sorted((float(range_.t_min), float(range_.t_max)) for range_ in phase.ranges)
        has_overlap = False
        has_gap = False
        for previous, following in zip(intervals, intervals[1:]):
            if following[0] == previous[1]:
                shared_endpoints += 1
            elif following[0] < previous[1]:
                has_overlap = True
            else:
                has_gap = True
        overlapping_sets += int(has_overlap)
        gap_sets += int(has_gap)

    timestamps = [np.frombuffer(chunk_matrix[0, 8:16].tobytes(), dtype="<f8")[0]]
    timestamps.extend(
        np.frombuffer(chunk_matrix[index, 22:30].tobytes(), dtype="<f8")[0]
        for index in range(1, len(chunk_ids))
    )

    raw_range_count = sum(chunk_id in CP_IDS or chunk_id == 11 for chunk_id in chunk_ids)
    attached_range_count = len(ranges) + len(kappas)
    return {
        "file_size": str(len(raw_bytes)),
        "chunk_count": str(len(chunk_ids)),
        "chunk_id_histogram": histogram(Counter(chunk_ids)),
        "unknown_chunk_count": str(sum(chunk_id not in KNOWN_IDS for chunk_id in chunk_ids)),
        "compound_count": str(len(compounds)),
        "ordinary_phase_count": str(sum(not phase.has_transition() for phase in phases)),
        "transition_phase_count": str(sum(phase.has_transition() for phase in phases)),
        "cp_range_count": str(len(ranges)),
        "kappa_count": str(len(kappas)),
        "comment_chunk_count": str(len(comments)),
        "orphan_range_count": str(max(0, raw_range_count - attached_range_count)),
        "duplicate_phase_id_count": str(duplicate_phase_ids),
        "phase_state_histogram": histogram(state_counts),
        "energy_unit_histogram": histogram(energy_codes),
        "pressure_unit_histogram": histogram(pressure_codes),
        "unknown_energy_unit_count": str(sum(value for key, value in energy_codes.items() if key not in (0, 1))),
        "unknown_pressure_unit_count": str(sum(value for key, value in pressure_codes.items() if key not in (0, 1))),
        "invalid_timestamp_count": str(sum(ole_timestamp_is_invalid(float(value)) for value in timestamps)),
        "invalid_density_count": str(
            sum(not math.isfinite(float(phase.chunk["density_raw"])) for phase in phases)
        ),
        "invalid_cp_bound_count": str(cp_bounds_invalid),
        "overlapping_cp_phase_count": str(overlapping_sets),
        "gapped_cp_phase_count": str(gap_sets),
        "shared_cp_endpoint_count": str(shared_endpoints),
    }


def main() -> int:
    argument_parser = argparse.ArgumentParser(description=__doc__)
    argument_parser.add_argument("cdb_path", type=Path)
    argument_parser.add_argument("--python-root", required=True, type=Path)
    arguments = argument_parser.parse_args()

    parser = load_python_parser(arguments.python_root)
    for key, value in collect_summary(arguments.cdb_path, parser).items():
        print(f"{key}={value}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

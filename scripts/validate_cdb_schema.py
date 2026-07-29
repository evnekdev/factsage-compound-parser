#!/usr/bin/env python3
"""Validate the FactSage Compound Kaitai schema against a local CDB fixture.

The generated Kaitai parser is validation output and is expected at
target/kaitai-python/factsage_compound.py by default. Generate it with
Kaitai Struct 0.11 before running this script.
"""

from __future__ import annotations

import argparse
import importlib
import math
import sys
from collections import Counter
from pathlib import Path
from typing import Any


CHUNK_SIZE = 256
BODY_SIZE = 255
KNOWN_IDS = set(range(1, 12))
PHASE_IDS = {7, 8}
CP_IDS = {2, 3, 4, 5, 6}
RANGE_IDS = CP_IDS | {11}

EXPECTED_BODY_CLASSES = {
    1: "CompoundBody",
    2: "CpBody",
    3: "CpBody",
    4: "CpBody",
    5: "CpBody",
    6: "CpBody",
    7: "PhaseOrdinaryBody",
    8: "PhaseTransitionBody",
    9: "DatabaseHeaderBody",
    10: "CommentBody",
    11: "KappaBody",
}

BODY_SIZE_PARTS = {
    "database_header_body": (1, 4, 2, 8, 1, 11, 80, 136, 12),
    "compound_body": (31, 40, 40, 40, 4, 4, 4, 12, 56, 24),
    "phase_ordinary_body": (31, 8, 8, 4, 4, 200),
    "phase_transition_body": (31, 8, 8, 4, 4, 200),
    "cp_body": (31, 8, 8, 4, 4, 8, 8, 64, 64, 56),
    "comment_body": (31, 80, 144),
    "kappa_body": (31, 8, 8, 4, 4, 80, 32, 24, 8, 40, 12, 4),
}

STRING_FIELDS = {
    9: (("comment", "header_comment", 28, 80, "ascii"),),
    1: (
        ("compound_name", "compound_name", 32, 40, "ascii"),
        ("reserved_string_1", "reserved_string_1", 72, 40, "ascii"),
        ("formula_name", "formula_name", 112, 40, "ascii"),
        ("reserved_string_2", "reserved_string_2", 164, 12, "ascii"),
    ),
    7: (("physical.phase_name", "phase_name", 136, 40, "ascii"),),
    8: (("physical.phase_name", "phase_name", 136, 40, "ascii"),),
    10: (("comment", "comment_fragment", 32, 80, "windows-1252"),),
}

KAPPA_FLOAT_FIELDS = (
    "f1_temperature_coefficients",
    "f1_temperature_powers",
    "f2_pressure_coefficients",
    "f2_pressure_powers",
    "f3_temperature_coefficients",
    "f3_temperature_powers",
)


def get_path(obj: Any, path: str) -> Any:
    for part in path.split("."):
        obj = getattr(obj, part)
    return obj


def histogram_text(counter: Counter[int]) -> str:
    return "{" + ", ".join(
        "{}: {}".format(key, counter[key]) for key in sorted(counter)
    ) + "}"


def finite_stats() -> dict[str, Any]:
    return {"checked": 0, "nonfinite": 0, "minimum": None, "maximum": None}


def record_floats(stats: dict[str, Any], values: Any) -> None:
    for value in values:
        stats["checked"] += 1
        value = float(value)
        if not math.isfinite(value):
            stats["nonfinite"] += 1
            continue
        if stats["minimum"] is None or value < stats["minimum"]:
            stats["minimum"] = value
        if stats["maximum"] is None or value > stats["maximum"]:
            stats["maximum"] = value


def int_stats() -> dict[str, Any]:
    return {"checked": 0, "out_of_range": 0, "minimum": None, "maximum": None}


def record_ints(stats: dict[str, Any], values: Any) -> None:
    for value in values:
        value = int(value)
        stats["checked"] += 1
        if not -(2**31) <= value <= 2**31 - 1:
            stats["out_of_range"] += 1
        if stats["minimum"] is None or value < stats["minimum"]:
            stats["minimum"] = value
        if stats["maximum"] is None or value > stats["maximum"]:
            stats["maximum"] = value


def validate_body_sizes(errors: list[str]) -> dict[str, int]:
    sizes = {}
    for name, parts in BODY_SIZE_PARTS.items():
        size = sum(parts)
        sizes[name] = size
        if size != BODY_SIZE:
            errors.append(
                "schema body-size assertion failed: {} totals {}, expected {}".format(
                    name, size, BODY_SIZE
                )
            )
    return sizes


def validate_order(ids: list[int]) -> list[tuple[int, int, str, int]]:
    violations: list[tuple[int, int, str, int]] = []
    if not ids:
        return [(0, 0, "start", -1)]
    if ids[0] != 9:
        violations.append((0, 0, "start", ids[0]))

    state = "between_compounds"
    for index, kind in enumerate(ids[1:], start=1):
        if state == "between_compounds":
            if kind == 1:
                state = "after_compound"
            else:
                violations.append((index, index * CHUNK_SIZE, state, kind))
        elif state == "after_compound":
            if kind in PHASE_IDS:
                state = "phases"
            elif kind in RANGE_IDS:
                state = "ranges"
            elif kind == 10:
                state = "comments"
            elif kind == 1:
                state = "after_compound"
            else:
                violations.append((index, index * CHUNK_SIZE, state, kind))
        elif state == "phases":
            if kind in PHASE_IDS:
                state = "phases"
            elif kind in RANGE_IDS:
                state = "ranges"
            elif kind == 10:
                state = "comments"
            elif kind == 1:
                state = "after_compound"
            else:
                violations.append((index, index * CHUNK_SIZE, state, kind))
        elif state == "ranges":
            if kind in RANGE_IDS:
                state = "ranges"
            elif kind == 10:
                state = "comments"
            elif kind == 1:
                state = "after_compound"
            else:
                violations.append((index, index * CHUNK_SIZE, state, kind))
        elif state == "comments":
            if kind == 10:
                state = "comments"
            elif kind == 1:
                state = "after_compound"
            else:
                violations.append((index, index * CHUNK_SIZE, state, kind))
    return violations


def collect_opaque_fields(obj: Any, seen: set[int]) -> tuple[int, int]:
    if obj is None or isinstance(obj, (str, int, float, bool, bytes, bytearray)):
        return (0, 0)
    if id(obj) in seen:
        return (0, 0)
    seen.add(id(obj))
    if isinstance(obj, (list, tuple)):
        fields = 0
        byte_count = 0
        for value in obj:
            nested_fields, nested_bytes = collect_opaque_fields(value, seen)
            fields += nested_fields
            byte_count += nested_bytes
        return fields, byte_count
    if not hasattr(obj, "__dict__"):
        return (0, 0)

    fields = 0
    byte_count = 0
    for name, value in vars(obj).items():
        if name.startswith('_'):
            continue
        lowered = name.lower()
        is_opaque_name = (
            "unknown" in lowered
            or "padding" in lowered
            or "reserved" in lowered
        )
        if is_opaque_name and isinstance(value, (bytes, bytearray)):
            fields += 1
            byte_count += len(value)
        elif is_opaque_name and isinstance(value, (list, tuple)):
            fields += 1
            byte_count += sum(
                len(item) for item in value if isinstance(item, (bytes, bytearray))
            )
        nested_fields, nested_bytes = collect_opaque_fields(value, seen)
        fields += nested_fields
        byte_count += nested_bytes
    return fields, byte_count


def print_float_stat(label: str, stats: dict[str, Any]) -> None:
    print(
        "{}: checked={}, nonfinite={}, min={}, max={}".format(
            label,
            stats["checked"],
            stats["nonfinite"],
            stats["minimum"],
            stats["maximum"],
        )
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate a CDB file against generated Kaitai Python code."
    )
    parser.add_argument("cdb", type=Path, help="path to the private/local CDB fixture")
    parser.add_argument(
        "--generated-dir",
        type=Path,
        default=None,
        help="directory containing generated factsage_compound.py",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    cdb_path = args.cdb
    repo_root = Path(__file__).resolve().parents[1]
    generated_dir = args.generated_dir or repo_root / "target" / "kaitai-python"
    errors: list[str] = []
    warnings: list[str] = []

    if not cdb_path.is_file():
        print("error: CDB path does not exist or is not a file: {}".format(cdb_path))
        return 2

    body_sizes = validate_body_sizes(errors)
    file_size = cdb_path.stat().st_size
    with cdb_path.open("rb") as handle:
        raw_data = handle.read()

    if len(raw_data) != file_size:
        errors.append(
            "raw read length {} differs from stat size {}".format(
                len(raw_data), file_size
            )
        )
    if file_size % CHUNK_SIZE != 0:
        errors.append(
            "file size {} is not divisible by {}".format(file_size, CHUNK_SIZE)
        )
    expected_chunk_count = file_size // CHUNK_SIZE
    raw_ids = [
        raw_data[offset]
        for offset in range(
            0, min(len(raw_data), expected_chunk_count * CHUNK_SIZE), CHUNK_SIZE
        )
    ]
    raw_histogram = Counter(raw_ids)
    unknown_offsets = [
        (index, index * CHUNK_SIZE, kind)
        for index, kind in enumerate(raw_ids)
        if kind not in KNOWN_IDS
    ]

    first_id = raw_ids[0] if raw_ids else None
    with cdb_path.open("rb") as handle:
        handle.seek(2)
        first_magic = handle.read(4)
    if first_id != 9:
        errors.append("first raw chunk ID is {}, expected 9".format(first_id))
    if first_magic != b"CMPD":
        errors.append("first raw header magic does not equal CMPD")

    ordering_violations = validate_order(raw_ids)
    for index, offset, state, kind in ordering_violations:
        errors.append(
            "ordering violation: chunk index {}, byte offset {}, state {}, encountered ID {}".format(
                index, offset, state, kind
            )
        )

    if unknown_offsets:
        warnings.append(
            "unknown chunk IDs: {}".format(
                ", ".join(
                    "index {} offset {} ID {}".format(index, offset, kind)
                    for index, offset, kind in unknown_offsets
                )
            )
        )

    print("CDB schema validation")
    print("file={}".format(cdb_path))
    print("file_size={}".format(file_size))
    print("expected_chunk_count={}".format(expected_chunk_count))
    print("raw_chunk_id_histogram={}".format(histogram_text(raw_histogram)))
    print("unknown_id_count={}".format(len(unknown_offsets)))
    print("first_raw_id={}".format(first_id))
    print("first_raw_magic_is_CMPD={}".format(first_magic == b"CMPD"))
    print("body_sizes={}".format(body_sizes))
    print("ordering_validation={}".format("PASS" if not ordering_violations else "FAIL"))

    if not generated_dir.is_dir():
        errors.append(
            "generated parser directory not found: {}; compile the schema first".format(
                generated_dir
            )
        )
        print("kaitai_parse=NOT_RUN")
        for error in errors:
            print("ERROR: " + error)
        return 2

    sys.path.insert(0, str(generated_dir.resolve()))
    try:
        from kaitaistruct import KaitaiStream
        module = importlib.import_module("factsage_compound")
        factsage_compound = module.FactsageCompound
    except Exception as exc:
        errors.append("could not import generated Kaitai parser: {}".format(exc))
        print("kaitai_parse=NOT_RUN")
        for error in errors:
            print("ERROR: " + error)
        return 2

    stream = None
    parsed = None
    parse_position = None
    root_position = None
    root_size = None
    try:
        with cdb_path.open("rb") as handle:
            stream = KaitaiStream(handle)
            try:
                parsed = factsage_compound(stream)
            except Exception:
                try:
                    parse_position = stream.pos()
                except Exception:
                    parse_position = None
                raise
            root_position = stream.pos()
            root_size = stream.size()
    except Exception as exc:
        position = parse_position
        if position is None:
            failing_index = None
            failing_offset = None
            failing_id = None
        else:
            failing_index = min(
                max(position // CHUNK_SIZE, 0),
                max(expected_chunk_count - 1, 0),
            )
            failing_offset = failing_index * CHUNK_SIZE
            failing_id = (
                raw_ids[failing_index]
                if 0 <= failing_index < len(raw_ids)
                else None
            )
        errors.append(
            "Kaitai parse failed at chunk index {}, byte offset {}, raw ID {}: {}".format(
                failing_index, failing_offset, failing_id, exc
            )
        )
        print("kaitai_parse=FAIL")
        for error in errors:
            print("ERROR: " + error)
        return 1

    parsed_ids: list[int] = []
    class_histogram: Counter[str] = Counter()
    parse_body_errors: list[str] = []
    for index, chunk in enumerate(parsed.chunks):
        kind = int(chunk.kind)
        parsed_ids.append(kind)
        body = chunk.body
        class_name = "raw" if isinstance(body, bytes) else type(body).__name__
        class_histogram[class_name] += 1
        raw_body = getattr(chunk, "_raw_body", None)
        if not isinstance(raw_body, bytes) or len(raw_body) != BODY_SIZE:
            parse_body_errors.append(
                "chunk index {} ID {} retained body length {}".format(
                    index, kind, len(raw_body) if raw_body is not None else None
                )
            )
        expected_class = EXPECTED_BODY_CLASSES.get(kind)
        if expected_class is None:
            if not isinstance(body, bytes) or len(body) != BODY_SIZE:
                parse_body_errors.append(
                    "unknown ID {} at chunk index {} was not retained as 255 raw bytes".format(
                        kind, index
                    )
                )
        elif class_name != expected_class:
            parse_body_errors.append(
                "chunk index {} ID {} decoded as {}, expected {}".format(
                    index, kind, class_name, expected_class
                )
            )
        elif not isinstance(body, bytes):
            body_io = getattr(body, "_io", None)
            if body_io is None:
                parse_body_errors.append(
                    "chunk index {} ID {} has no body stream".format(index, kind)
                )
            else:
                body_position = body_io.pos()
                body_size = body_io.size()
                if body_position != body_size or body_size != BODY_SIZE:
                    parse_body_errors.append(
                        "chunk index {} ID {} body stream position {}/{}".format(
                            index, kind, body_position, body_size
                        )
                    )

    errors.extend(parse_body_errors)
    parsed_histogram = Counter(parsed_ids)
    if len(parsed.chunks) != expected_chunk_count:
        errors.append(
            "parsed chunk count {} differs from raw expected {}".format(
                len(parsed.chunks), expected_chunk_count
            )
        )
    if parsed_histogram != raw_histogram:
        errors.append(
            "parsed and raw chunk-ID histograms differ: parsed={}, raw={}".format(
                histogram_text(parsed_histogram), histogram_text(raw_histogram)
            )
        )
    logical_sequence_error = None
    for logical_index, (parsed_kind, raw_kind) in enumerate(zip(parsed_ids, raw_ids)):
        if parsed_kind != raw_kind:
            logical_sequence_error = (
                logical_index, parsed_kind, raw_kind
            )
            break
    if logical_sequence_error is None and len(parsed_ids) != len(raw_ids):
        logical_sequence_error = (
            min(len(parsed_ids), len(raw_ids)), None, None
        )
    if logical_sequence_error is not None:
        logical_index, parsed_kind, raw_kind = logical_sequence_error
        errors.append(
            "parsed chunk sequence differs at logical index {}: parsed ID {}, raw ID {}".format(
                logical_index, parsed_kind, raw_kind
            )
        )
    if parsed_ids and parsed_ids[0] != 9:
        errors.append("first parsed chunk ID is {}, expected 9".format(parsed_ids[0]))
    if parsed.chunks:
        first_body = parsed.chunks[0].body
        if type(first_body).__name__ != "DatabaseHeaderBody":
            errors.append("first parsed body is not DatabaseHeaderBody")
        elif getattr(first_body, "magic", None) != b"CMPD":
            errors.append("parsed database-header magic validation did not yield CMPD")

    if root_position != root_size or root_position != file_size:
        errors.append(
            "Kaitai root stream position {}/{} does not equal file size {}".format(
                root_position, root_size, file_size
            )
        )

    print("kaitai_parse=PASS")
    print("parsed_chunk_count={}".format(len(parsed.chunks)))
    print("parsed_chunk_id_histogram={}".format(histogram_text(parsed_histogram)))
    print("logical_chunk_id_sequence={}".format("PASS" if logical_sequence_error is None else "FAIL"))
    print("parsed_body_class_histogram={}".format(dict(sorted(class_histogram.items()))))
    print("root_stream_position={}".format(root_position))
    print("root_stream_size={}".format(root_size))
    print("parsed_body_alignment={}".format("PASS" if not parse_body_errors else "FAIL"))

    timestamp_stats = finite_stats()
    phase_thermo_stats = finite_stats()
    bounds_stats = finite_stats()
    cp_coefficients_stats = finite_stats()
    cp_powers_stats = finite_stats()
    kappa_arrays_stats = finite_stats()
    phase_id_stats = int_stats()
    cp_ordered = 0
    cp_ranges = 0
    string_stats = {
        "ascii_fields_checked": 0,
        "ascii_decode_failures": 0,
        "ascii_non_ascii_bytes": 0,
        "extended_text_fields_checked": 0,
        "extended_text_decode_failures": 0,
        "extended_text_non_ascii_bytes": 0,
        "width_or_alignment_failures": 0,
    }
    raw_bodies_retained = 0
    opaque_field_count = 0
    opaque_byte_count = 0

    for index, (kind, chunk) in enumerate(zip(parsed_ids, parsed.chunks)):
        body = chunk.body
        raw_bodies_retained += len(getattr(chunk, "_raw_body", b""))
        if not isinstance(body, bytes):
            fields, byte_count = collect_opaque_fields(body, set())
            opaque_field_count += fields
            opaque_byte_count += byte_count

        if kind == 9:
            record_floats(timestamp_stats, (body.date_ole,))
        elif not isinstance(body, bytes):
            record_floats(timestamp_stats, (body.header.timestamp_ole,))

        if kind in (7, 8):
            if kind == 7:
                record_floats(phase_thermo_stats, (body.enthalpy, body.entropy))
                record_ints(phase_id_stats, (body.phase_id_raw_neg, body.phase_id_raw))
            else:
                record_floats(
                    phase_thermo_stats,
                    (body.transition_enthalpy, body.transition_temperature),
                )
                record_ints(
                    phase_id_stats,
                    (body.parent_phase_id_raw, body.phase_id_raw),
                )
        elif kind in CP_IDS:
            record_ints(phase_id_stats, (body.phase_id_raw,))
            record_floats(bounds_stats, (body.temperature_min, body.temperature_max))
            if math.isfinite(body.temperature_min) and math.isfinite(body.temperature_max):
                cp_ranges += 1
                if body.temperature_min <= body.temperature_max:
                    cp_ordered += 1
            record_floats(cp_coefficients_stats, body.coefficients)
            record_floats(cp_powers_stats, body.powers)
        elif kind == 11:
            record_ints(phase_id_stats, (body.phase_id_raw,))
            record_floats(bounds_stats, (body.temperature_min, body.temperature_max))
            for field in KAPPA_FLOAT_FIELDS:
                record_floats(kappa_arrays_stats, getattr(body, field))

        for path, label, offset, width, encoding in STRING_FIELDS.get(kind, ()):
            raw_field = raw_data[index * CHUNK_SIZE + offset:index * CHUNK_SIZE + offset + width]
            value = get_path(body, path)
            if len(value) > width:
                string_stats["width_or_alignment_failures"] += 1
            try:
                decoded = raw_field.rstrip(b"\x00").decode(encoding)
            except UnicodeDecodeError:
                if encoding == "ascii":
                    string_stats["ascii_decode_failures"] += 1
                else:
                    string_stats["extended_text_decode_failures"] += 1
                continue
            if encoding == "ascii":
                string_stats["ascii_fields_checked"] += 1
                string_stats["ascii_non_ascii_bytes"] += sum(byte >= 128 for byte in raw_field)
            else:
                string_stats["extended_text_fields_checked"] += 1
                string_stats["extended_text_non_ascii_bytes"] += sum(
                    byte >= 128 for byte in raw_field
                )
            if value != decoded:
                string_stats["width_or_alignment_failures"] += 1

    if raw_bodies_retained != expected_chunk_count * BODY_SIZE:
        errors.append(
            "retained raw body bytes {} differs from expected {}".format(
                raw_bodies_retained, expected_chunk_count * BODY_SIZE
            )
        )
    if string_stats["ascii_decode_failures"] or string_stats["width_or_alignment_failures"]:
        errors.append("fixed-string decoding or alignment checks failed")
    for label, stats in (
        ("timestamps", timestamp_stats),
        ("phase_thermo", phase_thermo_stats),
        ("range_bounds", bounds_stats),
        ("cp_coefficients", cp_coefficients_stats),
        ("cp_powers", cp_powers_stats),
        ("kappa_arrays", kappa_arrays_stats),
    ):
        if stats["nonfinite"]:
            errors.append("{} contains non-finite values".format(label))
    if phase_id_stats["out_of_range"]:
        errors.append("phase IDs outside signed 32-bit range were observed")
    if cp_ranges != cp_ordered:
        warnings.append(
            "CP ranges with t_min <= t_max: {}/{}".format(cp_ordered, cp_ranges)
        )

    print_float_stat("timestamps", timestamp_stats)
    print_float_stat("phase_thermo", phase_thermo_stats)
    print_float_stat("range_bounds", bounds_stats)
    print_float_stat("cp_coefficients", cp_coefficients_stats)
    print_float_stat("cp_powers", cp_powers_stats)
    print_float_stat("kappa_arrays", kappa_arrays_stats)
    print(
        "phase_ids: checked={}, out_of_range={}, min={}, max={}".format(
            phase_id_stats["checked"],
            phase_id_stats["out_of_range"],
            phase_id_stats["minimum"],
            phase_id_stats["maximum"],
        )
    )
    print("cp_t_min_le_t_max={}/{}".format(cp_ordered, cp_ranges))
    print("fixed_strings={}".format(string_stats))
    print(
        "retained_raw_body_bytes={}, opaque_fields={}, opaque_bytes={}".format(
            raw_bodies_retained, opaque_field_count, opaque_byte_count
        )
    )

    if errors:
        print("result=FAIL")
        for error in errors:
            print("ERROR: " + error)
        for warning in warnings:
            print("WARNING: " + warning)
        return 1
    if warnings:
        print("result=PASS_WITH_WARNINGS")
        for warning in warnings:
            print("WARNING: " + warning)
        return 0
    print("result=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use factsage_compound_parser::{
    Database, DiagnosticKind, EnergyUnit, PressureUnit, RawChunk, RawDatabase,
};

#[test]
fn parses_private_ms16base_when_available() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("MS16BASE.CDB");
    if !path.is_file() {
        return;
    }

    let bytes = fs::read(&path).expect("private fixture should be readable");
    assert_eq!(bytes.len(), 755_712);

    let database = RawDatabase::from_bytes(&bytes).expect("private fixture should parse");
    assert_eq!(database.chunks.len(), 2_952);

    match &database.chunks[0] {
        RawChunk::DatabaseHeader(header) => assert_eq!(header.magic, *b"CMPD"),
        _ => panic!("private fixture must begin with database header"),
    }

    let mut histogram = BTreeMap::new();
    for chunk in &database.chunks {
        *histogram.entry(chunk.id()).or_insert(0_usize) += 1;
    }

    let expected = BTreeMap::from([
        (1_u8, 537_usize),
        (2_u8, 1_517_usize),
        (5_u8, 1_usize),
        (7_u8, 620_usize),
        (8_u8, 64_usize),
        (9_u8, 1_usize),
        (10_u8, 212_usize),
    ]);
    assert_eq!(histogram, expected);
}

#[test]
fn groups_private_ms16base_when_available() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("MS16BASE.CDB");
    if !path.is_file() {
        return;
    }

    let bytes = fs::read(&path).expect("private fixture should be readable");
    let database = Database::from_bytes(&bytes).expect("private fixture should group");
    assert_eq!(database.compounds.len(), 537);
    assert_eq!(
        database
            .compounds
            .iter()
            .flat_map(|compound| compound.phases.iter())
            .filter(|phase| !phase.is_transition())
            .count(),
        620
    );
    assert_eq!(
        database
            .compounds
            .iter()
            .flat_map(|compound| compound.phases.iter())
            .filter(|phase| phase.is_transition())
            .count(),
        64
    );
    assert_eq!(
        database
            .compounds
            .iter()
            .flat_map(|compound| compound.phases.iter())
            .map(|phase| phase.heat_capacity_ranges.len())
            .sum::<usize>(),
        1_518
    );
    assert_eq!(
        database
            .compounds
            .iter()
            .map(|compound| compound.comment_fragments.len())
            .sum::<usize>(),
        212
    );
    assert_eq!(
        database
            .compounds
            .iter()
            .map(|compound| compound.orphan_ranges.len())
            .sum::<usize>(),
        0
    );
    assert_eq!(
        database
            .diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic.kind, DiagnosticKind::DuplicatePhaseId { .. }))
            .count(),
        0
    );
}

#[test]
fn evaluates_private_ms16base_thermodynamic_aggregates_when_available() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("MS16BASE.CDB");
    if !path.is_file() {
        return;
    }

    let bytes = fs::read(&path).expect("private fixture should be readable");
    let database = Database::from_bytes(&bytes).expect("private fixture should group");
    let mut energy_histogram = BTreeMap::new();
    let mut pressure_histogram = BTreeMap::new();
    let mut unknown_energy_units = 0;
    let mut unknown_pressure_units = 0;
    let mut thermodynamic_errors = 0;
    let mut timestamp_attempts = 0;
    let mut timestamp_errors = 0;
    let mut cp_ranges = 0;
    let mut cp_interior_points = 0;
    let mut cp_evaluation_errors = 0;
    let mut overlap_sets = 0;
    let mut gap_sets = 0;
    let mut density_errors = 0;

    timestamp_attempts += 1;
    timestamp_errors += usize::from(database.header.ole_date().is_err());
    for compound in &database.compounds {
        *energy_histogram
            .entry(compound.raw.unit_energy)
            .or_insert(0_usize) += 1;
        *pressure_histogram
            .entry(compound.raw.unit_pressure)
            .or_insert(0_usize) += 1;
        let energy_unit = compound.energy_unit();
        let pressure_unit = compound.pressure_unit();
        unknown_energy_units += usize::from(matches!(energy_unit, EnergyUnit::Unknown(_)));
        unknown_pressure_units += usize::from(matches!(pressure_unit, PressureUnit::Unknown(_)));

        timestamp_attempts += 1;
        timestamp_errors += usize::from(compound.ole_timestamp().is_err());
        for phase in &compound.phases {
            timestamp_attempts += 1;
            timestamp_errors += usize::from(phase.ole_timestamp().is_err());
            density_errors += usize::from(phase.density().is_err());
            if phase.is_transition() {
                thermodynamic_errors += usize::from(phase.transition_enthalpy_raw().is_err());
                thermodynamic_errors +=
                    usize::from(phase.transition_enthalpy_j_per_mol(energy_unit).is_err());
                thermodynamic_errors += usize::from(phase.transition_temperature_k().is_err());
                thermodynamic_errors += usize::from(phase.parent_phase_id_raw().is_err());
            } else {
                thermodynamic_errors += usize::from(phase.enthalpy_298_raw().is_err());
                thermodynamic_errors +=
                    usize::from(phase.enthalpy_298_j_per_mol(energy_unit).is_err());
                thermodynamic_errors += usize::from(phase.entropy_298_raw().is_err());
                thermodynamic_errors +=
                    usize::from(phase.entropy_298_j_per_mol_k(energy_unit).is_err());
            }

            let mut intervals = Vec::new();
            for range in &phase.heat_capacity_ranges {
                timestamp_attempts += 1;
                timestamp_errors += usize::from(range.ole_timestamp().is_err());
                cp_ranges += 1;
                intervals.push((range.temperature_min(), range.temperature_max()));
                thermodynamic_errors +=
                    usize::from(range.stored_enthalpy_j_per_mol(energy_unit).is_err());
                thermodynamic_errors +=
                    usize::from(range.stored_entropy_j_per_mol_k(energy_unit).is_err());
                if range.temperature_min().is_finite()
                    && range.temperature_max().is_finite()
                    && range.temperature_min() < range.temperature_max()
                {
                    cp_interior_points += 1;
                    let temperature = range.temperature_min()
                        + (range.temperature_max() - range.temperature_min()) / 2.0;
                    match range.heat_capacity_raw(temperature) {
                        Ok(value) if value.is_finite() => {}
                        _ => cp_evaluation_errors += 1,
                    }
                    match range.heat_capacity_j_per_mol_k(temperature, energy_unit) {
                        Ok(value) if value.is_finite() => {}
                        _ => cp_evaluation_errors += 1,
                    }
                }
            }
            for range in &phase.physical_property_ranges {
                timestamp_attempts += 1;
                timestamp_errors += usize::from(range.ole_timestamp().is_err());
            }
            if intervals.len() > 1 {
                intervals.sort_by(|left, right| left.0.total_cmp(&right.0));
                let mut overlap = false;
                let mut gap = false;
                for pair in intervals.windows(2) {
                    overlap |= pair[1].0 < pair[0].1;
                    gap |= pair[1].0 > pair[0].1;
                }
                overlap_sets += usize::from(overlap);
                gap_sets += usize::from(gap);
            }
        }
        for comment in &compound.comment_fragments {
            timestamp_attempts += 1;
            timestamp_errors += usize::from(comment.ole_timestamp().is_err());
        }
    }

    assert_eq!(
        energy_histogram,
        BTreeMap::from([(0_u32, 183_usize), (1_u32, 354_usize)])
    );
    assert_eq!(
        pressure_histogram,
        BTreeMap::from([(0_u32, 417_usize), (1_u32, 120_usize)])
    );
    assert_eq!(unknown_energy_units, 0);
    assert_eq!(unknown_pressure_units, 0);
    assert_eq!(thermodynamic_errors, 0);
    assert_eq!(timestamp_attempts, 2_952);
    assert_eq!(timestamp_errors, 0);
    assert_eq!(cp_ranges, 1_518);
    assert_eq!(cp_interior_points, 1_507);
    assert_eq!(cp_evaluation_errors, 0);
    assert_eq!(overlap_sets, 0);
    assert_eq!(gap_sets, 0);
    assert_eq!(density_errors, 0);
}

#[test]
fn round_trips_private_ms16base_when_available() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("MS16BASE.CDB");
    if !path.is_file() {
        return;
    }

    let input = fs::read(&path).expect("private fixture should be readable");
    let raw = RawDatabase::from_bytes(&input).expect("private fixture should parse");
    let serialized = raw.to_bytes().expect("private fixture should serialize");
    assert_eq!(serialized, input);

    let reparsed = RawDatabase::from_bytes(&serialized).expect("serialized fixture should parse");
    assert_eq!(reparsed.chunks.len(), raw.chunks.len());
    assert_eq!(
        reparsed.chunks.iter().map(RawChunk::id).collect::<Vec<_>>(),
        raw.chunks.iter().map(RawChunk::id).collect::<Vec<_>>()
    );

    let original_domain = Database::from_bytes(&input).expect("fixture should group");
    let reparsed_domain = Database::from_raw(reparsed).expect("serialized fixture should group");
    assert_eq!(
        reparsed_domain.compounds.len(),
        original_domain.compounds.len()
    );
    assert_eq!(
        reparsed_domain.diagnostics.len(),
        original_domain.diagnostics.len()
    );
    assert_eq!(
        reparsed_domain
            .compounds
            .iter()
            .flat_map(|compound| compound.phases.iter())
            .count(),
        original_domain
            .compounds
            .iter()
            .flat_map(|compound| compound.phases.iter())
            .count()
    );
}

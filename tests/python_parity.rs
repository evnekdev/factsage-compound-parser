use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use factsage_compound_parser::{Database, DiagnosticKind, RawChunk, RawDatabase};

fn histogram<K: Ord + std::fmt::Display>(values: BTreeMap<K, usize>) -> String {
    values
        .into_iter()
        .map(|(key, count)| format!("{key}:{count}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn state_name(phase: &factsage_compound_parser::Phase) -> &'static str {
    match phase.state() {
        factsage_compound_parser::PhaseState::Solid => "solid",
        factsage_compound_parser::PhaseState::Liquid => "liquid",
        factsage_compound_parser::PhaseState::Gas => "gas",
        factsage_compound_parser::PhaseState::Aqueous => "aqueous",
    }
}

fn parse_key_values(output: &[u8]) -> BTreeMap<String, String> {
    String::from_utf8(output.to_vec())
        .expect("parity script output must be UTF-8")
        .lines()
        .map(|line| {
            let (key, value) = line
                .split_once('=')
                .expect("parity script output must be key=value");
            (key.to_owned(), value.to_owned())
        })
        .collect()
}

fn rust_summary(bytes: &[u8]) -> BTreeMap<String, String> {
    let raw = RawDatabase::from_bytes(bytes).expect("fixture should parse");
    let database = Database::from_bytes(bytes).expect("fixture should group");

    let mut chunk_ids = BTreeMap::new();
    let mut unknown_chunk_count = 0;
    let mut invalid_cp_bound_count = 0;
    for chunk in &raw.chunks {
        *chunk_ids.entry(chunk.id()).or_insert(0_usize) += 1;
        unknown_chunk_count += usize::from(!matches!(chunk.id(), 1..=11));
        if let RawChunk::HeatCapacity { chunk, .. } = chunk {
            invalid_cp_bound_count += usize::from(
                !chunk.temperature_min.is_finite()
                    || !chunk.temperature_max.is_finite()
                    || chunk.temperature_min > chunk.temperature_max,
            );
        }
    }

    let mut energy_units = BTreeMap::new();
    let mut pressure_units = BTreeMap::new();
    let mut state_histogram = BTreeMap::new();
    let mut ordinary_phase_count = 0;
    let mut transition_phase_count = 0;
    let mut cp_range_count = 0;
    let mut kappa_count = 0;
    let mut comment_chunk_count = 0;
    let mut orphan_range_count = 0;
    let mut invalid_density_count = 0;
    let mut overlapping_cp_phase_count = 0;
    let mut gapped_cp_phase_count = 0;
    let mut shared_cp_endpoint_count = 0;
    let mut invalid_timestamp_count = usize::from(database.header.ole_date().is_err());

    for compound in &database.compounds {
        *energy_units
            .entry(compound.raw.unit_energy)
            .or_insert(0_usize) += 1;
        *pressure_units
            .entry(compound.raw.unit_pressure)
            .or_insert(0_usize) += 1;
        invalid_timestamp_count += usize::from(compound.ole_timestamp().is_err());
        comment_chunk_count += compound.comment_fragments.len();
        orphan_range_count += compound.orphan_ranges.len();

        for phase in &compound.phases {
            *state_histogram.entry(state_name(phase)).or_insert(0_usize) += 1;
            ordinary_phase_count += usize::from(!phase.is_transition());
            transition_phase_count += usize::from(phase.is_transition());
            invalid_timestamp_count += usize::from(phase.ole_timestamp().is_err());
            invalid_density_count += usize::from(phase.density().is_err());
            cp_range_count += phase.heat_capacity_ranges.len();
            kappa_count += phase.physical_property_ranges.len();

            let mut intervals = Vec::with_capacity(phase.heat_capacity_ranges.len());
            for range in &phase.heat_capacity_ranges {
                invalid_timestamp_count += usize::from(range.ole_timestamp().is_err());
                intervals.push((range.temperature_min(), range.temperature_max()));
            }
            for range in &phase.physical_property_ranges {
                invalid_timestamp_count += usize::from(range.ole_timestamp().is_err());
            }
            intervals.sort_by(|left, right| left.0.total_cmp(&right.0));
            let mut has_overlap = false;
            let mut has_gap = false;
            for pair in intervals.windows(2) {
                if pair[1].0 == pair[0].1 {
                    shared_cp_endpoint_count += 1;
                } else if pair[1].0 < pair[0].1 {
                    has_overlap = true;
                } else {
                    has_gap = true;
                }
            }
            overlapping_cp_phase_count += usize::from(has_overlap);
            gapped_cp_phase_count += usize::from(has_gap);
        }
        for comment in &compound.comment_fragments {
            invalid_timestamp_count += usize::from(comment.ole_timestamp().is_err());
        }
    }

    let duplicate_phase_id_count = database
        .diagnostics
        .iter()
        .filter(|diagnostic| matches!(diagnostic.kind, DiagnosticKind::DuplicatePhaseId { .. }))
        .count();

    BTreeMap::from([
        ("file_size".to_owned(), bytes.len().to_string()),
        ("chunk_count".to_owned(), raw.chunks.len().to_string()),
        ("chunk_id_histogram".to_owned(), histogram(chunk_ids)),
        (
            "unknown_chunk_count".to_owned(),
            unknown_chunk_count.to_string(),
        ),
        (
            "compound_count".to_owned(),
            database.compounds.len().to_string(),
        ),
        (
            "ordinary_phase_count".to_owned(),
            ordinary_phase_count.to_string(),
        ),
        (
            "transition_phase_count".to_owned(),
            transition_phase_count.to_string(),
        ),
        ("cp_range_count".to_owned(), cp_range_count.to_string()),
        ("kappa_count".to_owned(), kappa_count.to_string()),
        (
            "comment_chunk_count".to_owned(),
            comment_chunk_count.to_string(),
        ),
        (
            "orphan_range_count".to_owned(),
            orphan_range_count.to_string(),
        ),
        (
            "duplicate_phase_id_count".to_owned(),
            duplicate_phase_id_count.to_string(),
        ),
        (
            "phase_state_histogram".to_owned(),
            histogram(state_histogram),
        ),
        (
            "energy_unit_histogram".to_owned(),
            histogram(energy_units.clone()),
        ),
        (
            "pressure_unit_histogram".to_owned(),
            histogram(pressure_units.clone()),
        ),
        (
            "unknown_energy_unit_count".to_owned(),
            energy_units
                .iter()
                .filter(|(unit, _)| !matches!(**unit, 0 | 1))
                .map(|(_, count)| count)
                .sum::<usize>()
                .to_string(),
        ),
        (
            "unknown_pressure_unit_count".to_owned(),
            pressure_units
                .iter()
                .filter(|(unit, _)| !matches!(**unit, 0 | 1))
                .map(|(_, count)| count)
                .sum::<usize>()
                .to_string(),
        ),
        (
            "invalid_timestamp_count".to_owned(),
            invalid_timestamp_count.to_string(),
        ),
        (
            "invalid_density_count".to_owned(),
            invalid_density_count.to_string(),
        ),
        (
            "invalid_cp_bound_count".to_owned(),
            invalid_cp_bound_count.to_string(),
        ),
        (
            "overlapping_cp_phase_count".to_owned(),
            overlapping_cp_phase_count.to_string(),
        ),
        (
            "gapped_cp_phase_count".to_owned(),
            gapped_cp_phase_count.to_string(),
        ),
        (
            "shared_cp_endpoint_count".to_owned(),
            shared_cp_endpoint_count.to_string(),
        ),
    ])
}

#[test]
fn compares_aggregate_python_parity_when_configured() {
    let python_root = match env::var_os("FACTSAGE_COMPOUND_PYTHON_ROOT") {
        Some(path) => PathBuf::from(path),
        None => return,
    };
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("MS16BASE.CDB");
    if !fixture.is_file() {
        return;
    }

    let python =
        env::var_os("FACTSAGE_COMPOUND_PYTHON_EXECUTABLE").unwrap_or_else(|| "python".into());
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("python_parity.py");
    let output = Command::new(python)
        .arg(script)
        .arg(&fixture)
        .arg("--python-root")
        .arg(python_root)
        .output()
        .expect("configured Python executable should start");
    assert!(
        output.status.success(),
        "Python parity script failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let bytes = fs::read(fixture).expect("private fixture should be readable");
    assert_eq!(parse_key_values(&output.stdout), rust_summary(&bytes));
}

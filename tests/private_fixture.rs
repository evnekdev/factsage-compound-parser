use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

use factsage_compound_parser::{Database, RawChunk, RawDatabase};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("MS16BASE.CDB")
}

fn fixture_bytes() -> Option<Vec<u8>> {
    fs::read(fixture_path()).ok()
}

#[test]
fn private_fixture_raw_parse_and_streaming_reader_agree() {
    let Some(bytes) = fixture_bytes() else {
        return;
    };
    assert_eq!(bytes.len(), 755_712);
    let from_bytes = RawDatabase::from_bytes(&bytes).unwrap();
    let from_reader = RawDatabase::from_reader(Cursor::new(&bytes)).unwrap();
    assert_eq!(from_reader, from_bytes);
    assert_eq!(from_bytes.chunks().len(), 2_952);
    assert!(
        matches!(from_bytes.chunks().first(), Some(RawChunk::DatabaseHeader(header)) if header.magic == *b"CMPD")
    );
    let mut histogram = BTreeMap::<u8, usize>::new();
    for chunk in from_bytes.chunks() {
        *histogram.entry(chunk.id()).or_default() += 1;
    }
    assert_eq!(
        histogram,
        BTreeMap::from([
            (1, 537),
            (2, 1_517),
            (5, 1),
            (7, 620),
            (8, 64),
            (9, 1),
            (10, 212)
        ])
    );
}

#[test]
fn private_fixture_indexed_semantic_aggregate_counts_match() {
    let Some(bytes) = fixture_bytes() else {
        return;
    };
    let database = Database::from_bytes(&bytes).unwrap();
    let view = database.view().unwrap();
    let mut ordinary = 0;
    let mut transition = 0;
    let mut cp = 0;
    let mut kappa = 0;
    let mut comments = 0;
    let mut orphans = 0;
    for compound in view.compounds() {
        comments += compound.comment_fragments().count();
        orphans += compound.orphan_ranges().count();
        for phase in compound.phases() {
            if phase.is_transition() {
                transition += 1;
            } else {
                ordinary += 1;
            }
            cp += phase.heat_capacity_ranges().count();
            kappa += phase.physical_property_ranges().count();
        }
    }
    assert_eq!(view.compound_count(), 537);
    assert_eq!(ordinary, 620);
    assert_eq!(transition, 64);
    assert_eq!(cp, 1_518);
    assert_eq!(kappa, 0);
    assert_eq!(comments, 212);
    assert_eq!(orphans, 0);
    assert!(view.diagnostics().is_empty());
}

#[test]
fn private_fixture_thermodynamic_access_is_finite_at_safe_interior_points() {
    let Some(bytes) = fixture_bytes() else {
        return;
    };
    let database = Database::from_bytes(&bytes).unwrap();
    let view = database.view().unwrap();
    let mut energy_units = BTreeMap::<u32, usize>::new();
    let mut pressure_units = BTreeMap::<u32, usize>::new();
    let mut invalid_timestamps = usize::from(view.header().ole_date().is_err());
    let mut invalid_density = 0;
    let mut evaluated_cp = 0;
    let mut invalid_cp = 0;
    for compound in view.compounds() {
        *energy_units.entry(compound.raw().unit_energy).or_default() += 1;
        *pressure_units
            .entry(compound.raw().unit_pressure)
            .or_default() += 1;
        invalid_timestamps += usize::from(compound.ole_timestamp().is_err());
        invalid_timestamps += compound
            .comment_fragments()
            .map(|comment| usize::from(comment.ole_timestamp().is_err()))
            .sum::<usize>();
        for phase in compound.phases() {
            invalid_timestamps += usize::from(phase.ole_timestamp().is_err());
            invalid_density += usize::from(phase.density().is_err());
            for range in phase.heat_capacity_ranges() {
                invalid_timestamps += usize::from(range.ole_timestamp().is_err());
                let t_min = range.temperature_min();
                let t_max = range.temperature_max();
                if t_min.is_finite() && t_max.is_finite() && t_min <= t_max && t_max > 0.0 {
                    let temperature = if t_min > 0.0 {
                        t_min + (t_max - t_min) / 2.0
                    } else {
                        t_max / 2.0
                    };
                    if temperature.is_finite() && temperature > 0.0 {
                        evaluated_cp += 1;
                        if range.heat_capacity_raw(temperature).is_err() {
                            invalid_cp += 1;
                        }
                    }
                }
            }
            for range in phase.physical_property_ranges() {
                invalid_timestamps += usize::from(range.ole_timestamp().is_err());
            }
        }
    }
    assert_eq!(energy_units, BTreeMap::from([(0, 183), (1, 354)]));
    assert_eq!(pressure_units, BTreeMap::from([(0, 417), (1, 120)]));
    assert_eq!(invalid_timestamps, 0);
    assert_eq!(invalid_density, 0);
    assert_eq!(evaluated_cp, 1_518);
    assert_eq!(invalid_cp, 0);
}

#[test]
fn private_fixture_round_trips_and_reindexes_without_changing_aggregates() {
    let Some(bytes) = fixture_bytes() else {
        return;
    };
    let raw = RawDatabase::from_bytes(&bytes).unwrap();
    let serialized = raw.to_bytes().unwrap();
    assert_eq!(serialized, bytes);
    let reparsed = RawDatabase::from_bytes(&serialized).unwrap();
    assert_eq!(
        reparsed
            .chunks()
            .iter()
            .map(RawChunk::id)
            .collect::<Vec<_>>(),
        raw.chunks().iter().map(RawChunk::id).collect::<Vec<_>>()
    );
    let original = Database::from_raw(raw).unwrap();
    let reparsed = Database::from_raw(reparsed).unwrap();
    let original_view = original.view().unwrap();
    let reparsed_view = reparsed.view().unwrap();
    assert_eq!(
        original_view.compound_count(),
        reparsed_view.compound_count()
    );
    assert_eq!(original_view.diagnostics(), reparsed_view.diagnostics());
}

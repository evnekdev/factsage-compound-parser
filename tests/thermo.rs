use std::time::{Duration, SystemTime, UNIX_EPOCH};

use factsage_compound_parser::{
    Compound, Database, DateError, DensityError, DiagnosticKind, EnergyUnit, HeatCapacityError,
    PhaseKind, PhaseProperty, PhaseThermoError, PressureUnit, RawPhase, UnitError,
};

const CHUNK_SIZE: usize = 256;

fn put<const N: usize>(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: [u8; N]) {
    chunk[offset..offset + N].copy_from_slice(&value);
}

fn put_i32(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: i32) {
    put(chunk, offset, value.to_le_bytes());
}

fn put_u32(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: u32) {
    put(chunk, offset, value.to_le_bytes());
}

fn put_f64(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: f64) {
    put(chunk, offset, value.to_le_bytes());
}

fn common(chunk: &mut [u8; CHUNK_SIZE]) {
    chunk[1..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7]);
    chunk[8] = 0xaa;
    chunk[9..16].copy_from_slice(&[8, 9, 10, 11, 12, 13, 14]);
    chunk[16] = (-2_i8) as u8;
    chunk[17] = 42;
    put(chunk, 18, 0x1234_u16.to_le_bytes());
    put(chunk, 20, 0x5678_u16.to_le_bytes());
    put_f64(chunk, 22, 123.5);
    chunk[30..32].copy_from_slice(&[0xbb, 0xcc]);
}

fn header() -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 9;
    chunk[2..6].copy_from_slice(b"CMPD");
    put_f64(&mut chunk, 8, 0.5);
    chunk
}

fn compound(energy: u32, pressure: u32) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 1;
    common(&mut chunk);
    put_u32(&mut chunk, 156, energy);
    put_u32(&mut chunk, 160, pressure);
    chunk
}

fn ordinary(phase_id_raw: i32, density_raw: f64) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 7;
    common(&mut chunk);
    put_f64(&mut chunk, 32, 10.5);
    put_f64(&mut chunk, 40, 20.5);
    put_i32(&mut chunk, 48, -phase_id_raw);
    put_i32(&mut chunk, 52, phase_id_raw);
    put_f64(&mut chunk, 56, density_raw);
    chunk
}

fn transition(phase_id_raw: i32, parent_phase_id_raw: i32) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 8;
    common(&mut chunk);
    put_f64(&mut chunk, 32, 30.5);
    put_f64(&mut chunk, 40, 900.0);
    put_i32(&mut chunk, 48, parent_phase_id_raw);
    put_i32(&mut chunk, 52, phase_id_raw);
    chunk
}

fn cp(
    phase_id_raw: i32,
    t_min: f64,
    t_max: f64,
    enthalpy: f64,
    entropy: f64,
    coefficients: &[(usize, f64, f64)],
) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 2;
    common(&mut chunk);
    put_f64(&mut chunk, 32, enthalpy);
    put_f64(&mut chunk, 40, entropy);
    put_i32(&mut chunk, 48, phase_id_raw);
    put_f64(&mut chunk, 56, t_min);
    put_f64(&mut chunk, 64, t_max);
    for &(index, coefficient, power) in coefficients {
        put_f64(&mut chunk, 72 + index * 8, coefficient);
        put_f64(&mut chunk, 136 + index * 8, power);
    }
    chunk
}

fn database(chunks: impl IntoIterator<Item = [u8; CHUNK_SIZE]>) -> Database {
    let bytes = chunks.into_iter().flatten().collect::<Vec<_>>();
    Database::from_bytes(&bytes).expect("synthetic database should parse")
}

fn one_phase_database(
    energy: u32,
    phase: [u8; CHUNK_SIZE],
    ranges: impl IntoIterator<Item = [u8; CHUNK_SIZE]>,
) -> Database {
    let mut chunks = vec![header(), compound(energy, 0), phase];
    chunks.extend(ranges);
    database(chunks)
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}

fn ole_epoch() -> SystemTime {
    UNIX_EPOCH
        .checked_sub(Duration::from_secs(25_569 * 86_400))
        .unwrap()
}

#[test]
fn maps_energy_and_pressure_unit_codes() {
    assert_eq!(EnergyUnit::from_raw(0), EnergyUnit::Calories);
    assert_eq!(EnergyUnit::from_raw(1), EnergyUnit::Joules);
    assert_eq!(EnergyUnit::from_raw(99), EnergyUnit::Unknown(99));
    assert_eq!(PressureUnit::from_raw(0), PressureUnit::Atmospheres);
    assert_eq!(PressureUnit::from_raw(1), PressureUnit::Bars);
    assert_eq!(PressureUnit::from_raw(99), PressureUnit::Unknown(99));
    assert_eq!(EnergyUnit::Unknown(7).raw(), 7);
    assert_eq!(PressureUnit::Unknown(8).raw(), 8);
}

#[test]
fn converts_calories_and_passes_joules_through() {
    assert_close(EnergyUnit::Calories.to_joules(10.0).unwrap(), 41.84);
    assert_close(EnergyUnit::Calories.from_joules(41.84).unwrap(), 10.0);
    assert_close(EnergyUnit::Joules.to_joules(10.0).unwrap(), 10.0);
    assert_close(EnergyUnit::Joules.from_joules(10.0).unwrap(), 10.0);
}

#[test]
fn rejects_unknown_units_and_non_finite_conversion_values() {
    assert_eq!(
        EnergyUnit::Unknown(7).to_joules(1.0),
        Err(UnitError::UnknownEnergyUnit { raw: 7 })
    );
    assert_eq!(
        PressureUnit::Unknown(8).require_known(),
        Err(UnitError::UnknownPressureUnit { raw: 8 })
    );
    assert!(matches!(
        EnergyUnit::Joules.to_joules(f64::NAN),
        Err(UnitError::NonFiniteValue { .. })
    ));
}

#[test]
fn exposes_compound_units_and_ordinary_conversions() {
    let database = one_phase_database(0, ordinary(101, 1234.5), std::iter::empty());
    let compound = &database.compounds[0];
    let phase = &compound.phases[0];
    assert_eq!(compound.energy_unit(), EnergyUnit::Calories);
    assert_eq!(compound.pressure_unit(), PressureUnit::Atmospheres);
    assert_close(phase.enthalpy_298_raw().unwrap(), 10.5);
    assert_close(phase.entropy_298_raw().unwrap(), 20.5);
    assert_close(compound.enthalpy_298_j_per_mol(0).unwrap(), 10.5 * 4.184);
    assert_close(compound.entropy_298_j_per_mol_k(0).unwrap(), 20.5 * 4.184);
}

#[test]
fn exposes_transition_accessors_and_wrong_phase_errors() {
    let database = database([
        header(),
        compound(0, 0),
        ordinary(101, 100.0),
        transition(201, 101),
    ]);
    let ordinary = &database.compounds[0].phases[0];
    let transition = &database.compounds[0].phases[1];
    assert_close(
        transition
            .transition_enthalpy_j_per_mol(EnergyUnit::Calories)
            .unwrap(),
        30.5 * 4.184,
    );
    assert_eq!(transition.transition_temperature_k().unwrap(), 900.0);
    assert_eq!(transition.parent_phase_id_raw().unwrap(), 101);
    assert_eq!(
        ordinary.transition_enthalpy_raw(),
        Err(PhaseThermoError::WrongPhaseType {
            property: PhaseProperty::TransitionEnthalpy,
            phase_kind: PhaseKind::Ordinary,
        })
    );
    assert_eq!(
        transition.enthalpy_298_raw(),
        Err(PhaseThermoError::WrongPhaseType {
            property: PhaseProperty::Enthalpy298,
            phase_kind: PhaseKind::Transition,
        })
    );
}

#[test]
fn decodes_density_and_rejects_non_finite_density() {
    let database = one_phase_database(1, ordinary(101, 1_234.5), std::iter::empty());
    let phase = &database.compounds[0].phases[0];
    assert_eq!(phase.density_raw(), 1_234.5);
    assert_eq!(phase.density().unwrap(), 1_234.5);

    let database = one_phase_database(1, ordinary(101, f64::NAN), std::iter::empty());
    assert!(matches!(
        database.compounds[0].phases[0].density(),
        Err(DensityError::NonFinite { .. })
    ));
}

#[test]
fn converts_ole_epoch_fraction_and_negative_dates() {
    assert_eq!(
        factsage_compound_parser::OleAutomationDate::from_raw(0.0)
            .unwrap()
            .to_system_time()
            .unwrap(),
        ole_epoch()
    );
    assert_eq!(
        factsage_compound_parser::OleAutomationDate::from_raw(0.5)
            .unwrap()
            .to_system_time()
            .unwrap(),
        ole_epoch()
            .checked_add(Duration::from_secs(43_200))
            .unwrap()
    );
    assert_eq!(
        factsage_compound_parser::OleAutomationDate::from_raw(-0.25)
            .unwrap()
            .to_system_time()
            .unwrap(),
        ole_epoch()
            .checked_add(Duration::from_secs(21_600))
            .unwrap()
    );
}

#[test]
fn rejects_non_finite_and_out_of_range_ole_dates() {
    assert!(matches!(
        factsage_compound_parser::OleAutomationDate::from_raw(f64::NAN),
        Err(DateError::NonFinite { .. })
    ));
    assert_eq!(
        factsage_compound_parser::OleAutomationDate::from_raw(2_958_466.0),
        Err(DateError::OutOfRange {
            raw_days: 2_958_466.0
        })
    );
}

#[test]
fn exposes_entry_and_record_timestamps() {
    let database = one_phase_database(
        1,
        ordinary(101, 1.0),
        [cp(101, 100.0, 200.0, 1.0, 2.0, &[(0, 1.0, 0.0)])],
    );
    assert_eq!(database.header.ole_date().unwrap().raw_days, 0.5);
    assert_eq!(
        database.compounds[0].ole_timestamp().unwrap().raw_days,
        123.5
    );
    assert_eq!(
        database.compounds[0].phases[0]
            .ole_timestamp()
            .unwrap()
            .raw_days,
        123.5
    );
    assert_eq!(
        database.compounds[0].phases[0].heat_capacity_ranges[0]
            .ole_timestamp()
            .unwrap()
            .raw_days,
        123.5
    );
}

#[test]
fn evaluates_constant_and_multiple_power_cp_terms() {
    let constant = one_phase_database(
        1,
        ordinary(101, 1.0),
        [cp(101, 100.0, 500.0, 0.0, 0.0, &[(0, 2.0, 0.0)])],
    );
    assert_close(
        constant.compounds[0].heat_capacity_at(0, 300.0).unwrap(),
        2.0,
    );

    let multiple = one_phase_database(
        1,
        ordinary(101, 1.0),
        [cp(
            101,
            1.0,
            500.0,
            0.0,
            0.0,
            &[(0, 1.0, 0.0), (1, 2.0, 1.0), (2, 3.0, 0.5)],
        )],
    );
    assert_close(
        multiple.compounds[0].phases[0].heat_capacity_ranges[0]
            .heat_capacity_raw(4.0)
            .unwrap(),
        15.0,
    );
}

#[test]
fn evaluates_negative_and_fractional_powers_at_positive_temperature() {
    let database = one_phase_database(
        1,
        ordinary(101, 1.0),
        [cp(
            101,
            1.0,
            10.0,
            0.0,
            0.0,
            &[(0, 1.0, 0.0), (1, 2.0, -1.0), (2, 3.0, 0.5)],
        )],
    );
    assert_close(
        database.compounds[0].phases[0].heat_capacity_ranges[0]
            .heat_capacity_raw(4.0)
            .unwrap(),
        1.0 + 0.5 + 6.0,
    );
}

#[test]
fn uses_inclusive_boundaries_and_reports_outside_temperature() {
    let database = one_phase_database(
        1,
        ordinary(101, 1.0),
        [cp(101, 100.0, 300.0, 0.0, 0.0, &[(0, 2.0, 0.0)])],
    );
    let phase = &database.compounds[0].phases[0];
    assert!(phase.heat_capacity_ranges[0].contains_temperature(100.0));
    assert!(phase.heat_capacity_ranges[0].contains_temperature(300.0));
    assert_close(
        phase.heat_capacity_at(100.0, EnergyUnit::Joules).unwrap(),
        2.0,
    );
    assert_close(
        phase.heat_capacity_at(300.0, EnergyUnit::Joules).unwrap(),
        2.0,
    );
    assert!(matches!(
        phase.heat_capacity_at(99.0, EnergyUnit::Joules),
        Err(HeatCapacityError::TemperatureOutsideAllRanges { .. })
    ));
}

#[test]
fn detects_overlapping_and_unsorted_ranges() {
    let overlapping = one_phase_database(
        1,
        ordinary(101, 1.0),
        [
            cp(101, 100.0, 300.0, 0.0, 0.0, &[(0, 1.0, 0.0)]),
            cp(101, 200.0, 400.0, 0.0, 0.0, &[(0, 2.0, 0.0)]),
        ],
    );
    assert!(matches!(
        overlapping.compounds[0].phases[0].heat_capacity_at(250.0, EnergyUnit::Joules),
        Err(HeatCapacityError::OverlappingRanges {
            candidate_range_indexes,
            ..
        }) if candidate_range_indexes == vec![0, 1]
    ));

    let unsorted = one_phase_database(
        1,
        ordinary(101, 1.0),
        [
            cp(101, 300.0, 500.0, 0.0, 0.0, &[(0, 1.0, 0.0)]),
            cp(101, 100.0, 299.0, 0.0, 0.0, &[(0, 2.0, 0.0)]),
        ],
    );
    assert_close(
        unsorted.compounds[0].phases[0]
            .heat_capacity_at(150.0, EnergyUnit::Joules)
            .unwrap(),
        2.0,
    );
}

#[test]
fn detects_non_finite_terms_and_results() {
    let non_finite_coefficient = one_phase_database(
        1,
        ordinary(101, 1.0),
        [cp(101, 1.0, 100.0, 0.0, 0.0, &[(0, f64::NAN, 0.0)])],
    );
    assert!(matches!(
        non_finite_coefficient.compounds[0].phases[0].heat_capacity_ranges[0]
            .heat_capacity_raw(10.0),
        Err(HeatCapacityError::NonFiniteCoefficient {
            coefficient_index: 0,
            ..
        })
    ));

    let non_finite_power = one_phase_database(
        1,
        ordinary(101, 1.0),
        [cp(101, 1.0, 100.0, 0.0, 0.0, &[(0, 1.0, f64::INFINITY)])],
    );
    assert!(matches!(
        non_finite_power.compounds[0].phases[0].heat_capacity_ranges[0].heat_capacity_raw(10.0),
        Err(HeatCapacityError::NonFinitePower { power_index: 0, .. })
    ));

    let non_finite_result = one_phase_database(
        1,
        ordinary(101, 1.0),
        [cp(
            101,
            1.0,
            100.0,
            0.0,
            0.0,
            &[(0, f64::MAX, 0.0), (1, f64::MAX, 0.0)],
        )],
    );
    assert!(matches!(
        non_finite_result.compounds[0].phases[0].heat_capacity_ranges[0].heat_capacity_raw(10.0),
        Err(HeatCapacityError::NonFiniteResult { .. })
    ));
}

#[test]
fn converts_cp_for_calories_and_joules_and_preserves_stored_values() {
    let calories = one_phase_database(
        0,
        ordinary(101, 1.0),
        [cp(101, 100.0, 500.0, 10.0, 20.0, &[(0, 2.0, 0.0)])],
    );
    let range = &calories.compounds[0].phases[0].heat_capacity_ranges[0];
    assert_close(
        range
            .heat_capacity_j_per_mol_k(300.0, EnergyUnit::Calories)
            .unwrap(),
        2.0 * 4.184,
    );
    assert_close(range.stored_enthalpy_raw(), 10.0);
    assert_close(range.stored_entropy_raw(), 20.0);
    assert_close(
        range
            .stored_enthalpy_j_per_mol(EnergyUnit::Calories)
            .unwrap(),
        10.0 * 4.184,
    );
    assert_close(
        range
            .stored_entropy_j_per_mol_k(EnergyUnit::Calories)
            .unwrap(),
        20.0 * 4.184,
    );

    let joules = one_phase_database(
        1,
        ordinary(101, 1.0),
        [cp(101, 100.0, 500.0, 10.0, 20.0, &[(0, 2.0, 0.0)])],
    );
    assert_close(
        joules.compounds[0].phases[0].heat_capacity_ranges[0]
            .heat_capacity_j_per_mol_k(300.0, EnergyUnit::Joules)
            .unwrap(),
        2.0,
    );
}

#[test]
fn raw_phase_records_remain_available_to_thermo_accessors() {
    let database = database([
        header(),
        compound(1, 0),
        ordinary(101, 1.0),
        transition(201, 101),
    ]);
    assert!(matches!(
        database.compounds[0].phases[0].raw,
        RawPhase::Ordinary(_)
    ));
    assert!(matches!(
        database.compounds[0].phases[1].raw,
        RawPhase::Transition(_)
    ));
    assert!(database.diagnostics.iter().all(|diagnostic| !matches!(
        diagnostic.kind,
        DiagnosticKind::NonFiniteTemperatureRange { .. }
    )));
}

#[allow(dead_code)]
fn _compound_type_is_public(_: &Compound) {}

use std::time::{Duration, UNIX_EPOCH};

use factsage_compound_parser::{
    Database, DateError, DensityError, EnergyUnit, HeatCapacityError, PhaseKind, PhaseProperty,
    PhaseThermoError, PressureUnit, RawPhase, UnitError,
};

const CHUNK_SIZE: usize = 256;

fn put_i32(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: i32) {
    chunk[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: u32) {
    chunk[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_f64(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: f64) {
    chunk[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
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
    put_u32(&mut chunk, 156, energy);
    put_u32(&mut chunk, 160, pressure);
    chunk
}

fn ordinary(phase_id_raw: i32, density_raw: f64) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 7;
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
    coefficients: &[(usize, f64, f64)],
) -> [u8; CHUNK_SIZE] {
    let mut chunk = [0; CHUNK_SIZE];
    chunk[0] = 2;
    put_f64(&mut chunk, 32, 10.0);
    put_f64(&mut chunk, 40, 20.0);
    put_i32(&mut chunk, 48, phase_id_raw);
    put_f64(&mut chunk, 56, t_min);
    put_f64(&mut chunk, 64, t_max);
    for &(index, coefficient, power) in coefficients {
        put_f64(&mut chunk, 72 + index * 8, coefficient);
        put_f64(&mut chunk, 136 + index * 8, power);
    }
    chunk
}

fn source_database(chunks: impl IntoIterator<Item = [u8; CHUNK_SIZE]>) -> Database {
    Database::from_bytes(&chunks.into_iter().flatten().collect::<Vec<_>>()).unwrap()
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}

#[test]
fn maps_and_converts_units() {
    assert_eq!(EnergyUnit::from_raw(0), EnergyUnit::Calories);
    assert_eq!(EnergyUnit::from_raw(1), EnergyUnit::Joules);
    assert_eq!(PressureUnit::from_raw(1), PressureUnit::Bars);
    close(EnergyUnit::Calories.to_joules(10.0).unwrap(), 41.84);
    assert_eq!(
        EnergyUnit::Unknown(7).to_joules(1.0),
        Err(UnitError::UnknownEnergyUnit { raw: 7 })
    );
}

#[test]
fn exposes_phase_units_density_and_raw_variant_without_duplication() {
    let database = source_database([
        header(),
        compound(0, 1),
        ordinary(101, 1_234.5),
        transition(201, 101),
    ]);
    let view = database.view().unwrap();
    assert_eq!(view.header().ole_date().unwrap().raw_days, 0.5);
    let compound = view.compounds().next().unwrap();
    let phases = compound.phases().collect::<Vec<_>>();
    assert_eq!(compound.energy_unit(), EnergyUnit::Calories);
    assert_eq!(compound.pressure_unit(), PressureUnit::Bars);
    close(phases[0].enthalpy_298_raw().unwrap(), 10.5);
    close(compound.entropy_298_j_per_mol_k(0).unwrap(), 20.5 * 4.184);
    close(
        phases[1]
            .transition_enthalpy_j_per_mol(compound.energy_unit())
            .unwrap(),
        30.5 * 4.184,
    );
    assert_eq!(phases[1].transition_temperature_k().unwrap(), 900.0);
    assert_eq!(phases[1].parent_phase_id_raw().unwrap(), 101);
    assert_eq!(phases[0].density().unwrap(), 1_234.5);
    assert!(matches!(phases[0].raw(), RawPhase::Ordinary(_)));
    assert_eq!(
        phases[0].transition_enthalpy_raw(),
        Err(PhaseThermoError::WrongPhaseType {
            property: PhaseProperty::TransitionEnthalpy,
            phase_kind: PhaseKind::Ordinary,
        })
    );
}

#[test]
fn handles_ole_and_density_errors() {
    let epoch = UNIX_EPOCH
        .checked_sub(Duration::from_secs(25_569 * 86_400))
        .unwrap();
    assert_eq!(
        factsage_compound_parser::OleAutomationDate::from_raw(-0.25)
            .unwrap()
            .to_system_time()
            .unwrap(),
        epoch.checked_add(Duration::from_secs(21_600)).unwrap()
    );
    assert!(matches!(
        factsage_compound_parser::OleAutomationDate::from_raw(f64::NAN),
        Err(DateError::NonFinite { .. })
    ));
    let database = source_database([header(), compound(1, 0), ordinary(101, f64::NAN)]);
    let phase = database
        .view()
        .unwrap()
        .compounds()
        .next()
        .unwrap()
        .phases()
        .next()
        .unwrap();
    assert!(matches!(
        phase.density(),
        Err(DensityError::NonFinite { .. })
    ));
}

#[test]
fn evaluates_cp_expression_anchors_boundaries_and_overlap_policy() {
    let database = source_database([
        header(),
        compound(0, 0),
        ordinary(101, 1.0),
        cp(
            101,
            100.0,
            200.0,
            &[(0, 1.0, 0.0), (1, 2.0, 1.0), (2, 3.0, 0.5)],
        ),
        cp(101, 200.0, 300.0, &[(0, 2.0, 0.0)]),
    ]);
    let semantic_compound = database.view().unwrap().compounds().next().unwrap();
    let phase = semantic_compound.phases().next().unwrap();
    let first = phase.heat_capacity_ranges().next().unwrap();
    close(
        first.heat_capacity_raw(150.0).unwrap(),
        1.0 + 300.0 + 3.0 * 150.0_f64.sqrt(),
    );
    close(first.stored_enthalpy_raw(), 10.0);
    close(
        first
            .stored_entropy_j_per_mol_k(EnergyUnit::Calories)
            .unwrap(),
        20.0 * 4.184,
    );
    close(
        phase.heat_capacity_at(200.0, EnergyUnit::Joules).unwrap(),
        1.0 + 400.0 + 3.0 * 200.0_f64.sqrt(),
    );
    assert!(matches!(
        phase.heat_capacity_at(99.0, EnergyUnit::Joules),
        Err(HeatCapacityError::TemperatureOutsideAllRanges { .. })
    ));

    let overlapping = source_database([
        header(),
        compound(1, 0),
        ordinary(101, 1.0),
        cp(101, 100.0, 300.0, &[(0, 1.0, 0.0)]),
        cp(101, 200.0, 400.0, &[(0, 2.0, 0.0)]),
    ]);
    let phase = overlapping
        .view()
        .unwrap()
        .compounds()
        .next()
        .unwrap()
        .phases()
        .next()
        .unwrap();
    assert!(matches!(
        phase.heat_capacity_at(250.0, EnergyUnit::Joules),
        Err(HeatCapacityError::OverlappingRanges { .. })
    ));
}

#[test]
fn rejects_invalid_cp_inputs() {
    let database = source_database([
        header(),
        compound(1, 0),
        ordinary(101, 1.0),
        cp(101, 100.0, 200.0, &[(0, f64::NAN, 0.0)]),
    ]);
    let range = database
        .view()
        .unwrap()
        .compounds()
        .next()
        .unwrap()
        .phases()
        .next()
        .unwrap()
        .heat_capacity_ranges()
        .next()
        .unwrap();
    assert!(matches!(
        range.heat_capacity_raw(150.0),
        Err(HeatCapacityError::NonFiniteCoefficient { .. })
    ));
    assert!(matches!(
        range.heat_capacity_raw(0.0),
        Err(HeatCapacityError::NonPositiveTemperature { .. })
    ));
}

use factsage_compound_parser::fdb_build::{
    FdbAuxiliaryIntent, FdbBuildPlan, FdbChargeState, FdbCpTerm, FdbDatabaseMetadata,
    FdbElementAmount, FdbFormulaGroupPlan, FdbFreshMaterialization, FdbFunctionIdentity,
    FdbFunctionPlan, FdbMaterializeError, FdbNativeTemplates, FdbOrdinaryFunctionPlan,
    FdbStoichiometricAmount, FdbThermoRangePlan,
};
use factsage_compound_parser::{
    DomainIndex, EnergyUnit, PhaseState, PressureUnit, RawChunk, RawDatabase,
};

fn put_f64(record: &mut [u8; 256], offset: usize, value: f64) {
    record[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn native_templates() -> FdbNativeTemplates {
    let mut header = [0; 256];
    header[0] = 9;
    header[2..6].copy_from_slice(b"CMPD");
    let mut compound = [0; 256];
    compound[0] = 1;
    compound[17] = 1;
    compound[32..72].fill(b' ');
    compound[112..152].fill(b' ');
    let mut phase = [0; 256];
    phase[0] = 7;
    phase[17] = 2;
    phase[48..52].copy_from_slice(&(-101_i32).to_le_bytes());
    phase[52..56].copy_from_slice(&101_i32.to_le_bytes());
    phase[136..176].fill(b' ');
    let mut cp = [0; 256];
    cp[0] = 2;
    cp[17] = 3;
    cp[48..52].copy_from_slice(&101_i32.to_le_bytes());
    put_f64(&mut cp, 56, 298.15);
    put_f64(&mut cp, 64, 1000.0);
    let empty = RawDatabase::from_bytes(&header).unwrap();
    let source = RawDatabase::from_bytes(&[header, compound, phase, cp].concat()).unwrap();
    FdbNativeTemplates::from_examples(&empty, &source).unwrap()
}

fn group(
    formula: &str,
    symbols: &[&str],
    charge: i32,
    functions: Vec<FdbFunctionPlan>,
) -> FdbFormulaGroupPlan {
    FdbFormulaGroupPlan {
        formula: formula.into(),
        elements: symbols
            .iter()
            .map(|symbol| FdbElementAmount {
                symbol: (*symbol).into(),
                amount: FdbStoichiometricAmount {
                    numerator: 1,
                    denominator: 1,
                },
            })
            .collect(),
        charge: FdbChargeState::new(charge),
        energy_unit: EnergyUnit::Joules,
        pressure_unit: PressureUnit::Bars,
        functions,
    }
}

fn function(name: &str, ranges: Vec<FdbThermoRangePlan>) -> FdbFunctionPlan {
    FdbFunctionPlan::Ordinary(FdbOrdinaryFunctionPlan {
        identity: FdbFunctionIdentity::fresh_modern(name, PhaseState::Gas),
        phase_enthalpy: -100.0,
        phase_entropy: 10.0,
        ranges,
        auxiliary: FdbAuxiliaryIntent::Inactive,
    })
}

fn range(min: f64, max: f64, coefficient: f64) -> FdbThermoRangePlan {
    FdbThermoRangePlan {
        temperature_min_k: min,
        temperature_max_k: max,
        reference_enthalpy: -100.0,
        reference_entropy: 10.0,
        cp_terms: vec![FdbCpTerm {
            coefficient,
            power: 0.0,
        }],
    }
}

fn plan(groups: Vec<FdbFormulaGroupPlan>) -> FdbBuildPlan {
    FdbBuildPlan::new_fresh_modern(
        FdbDatabaseMetadata {
            comment: "synthetic".into(),
            date_ole: 45_000.0,
        },
        groups,
    )
    .unwrap()
}

#[test]
fn empty_fresh_plan_emits_only_versioned_database_header() {
    let raw = plan(vec![])
        .materialize_fresh(&native_templates(), &FdbFreshMaterialization::new(45_001.0))
        .unwrap();
    assert_eq!(
        raw.chunks().iter().map(RawChunk::id).collect::<Vec<_>>(),
        [9]
    );
    assert!(DomainIndex::build(&raw).unwrap().diagnostics().is_empty());
}

#[test]
fn builds_multiple_functions_and_charge_groups_in_physical_order() {
    let plan = plan(vec![
        group(
            "NiS",
            &["Ni", "S"],
            0,
            vec![
                function("First", vec![range(298.15, 1000.0, 100.0)]),
                function("Second", vec![range(298.15, 1000.0, 0.0)]),
            ],
        ),
        group(
            "CaO[+]",
            &["Ca", "O"],
            1,
            vec![function("Charged", vec![range(298.15, 1000.0, 25.0)])],
        ),
    ]);
    let mut options = FdbFreshMaterialization::new(45_001.0);
    options
        .ordinary_density
        .insert(("NiS".into(), "First".into()), 3.0);
    let raw = plan
        .materialize_fresh(&native_templates(), &options)
        .unwrap();
    assert_eq!(
        raw.chunks().iter().map(RawChunk::id).collect::<Vec<_>>(),
        [9, 1, 7, 7, 2, 2, 1, 7, 2]
    );
    assert!(DomainIndex::build(&raw).unwrap().diagnostics().is_empty());
    let RawChunk::Compound(first) = &raw.chunks()[1] else {
        panic!()
    };
    assert_eq!(first.header.element_ids[..2], [28, 16]);
    assert_eq!(first.header.element_coefficients[..2], [1, 1]);
    assert_eq!(first.header.charge_raw, 50);
    assert_eq!(first.header.entry_number, 1);
    assert_eq!(first.compound_name, [b' '; 40]);
    let RawChunk::PhaseOrdinary(phase_1) = &raw.chunks()[2] else {
        panic!()
    };
    let RawChunk::PhaseOrdinary(phase_2) = &raw.chunks()[3] else {
        panic!()
    };
    assert_eq!((phase_1.phase_id_raw, phase_2.phase_id_raw), (101, 102));
    assert_eq!(
        (phase_1.header.entry_number, phase_2.header.entry_number),
        (2, 3)
    );
    assert_eq!(phase_1.physical.density_raw, 3.0);
    let RawChunk::HeatCapacity { chunk: cp_1, .. } = &raw.chunks()[4] else {
        panic!()
    };
    let RawChunk::HeatCapacity { chunk: cp_2, .. } = &raw.chunks()[5] else {
        panic!()
    };
    assert_eq!((cp_1.phase_id_raw, cp_2.phase_id_raw), (101, 102));
    assert_eq!((cp_1.header.entry_number, cp_2.header.entry_number), (4, 5));
    assert!(cp_2.coefficients.iter().all(|value| *value == 0.0));
    assert!(cp_2.powers.iter().all(|value| *value == 0.0));
    let RawChunk::Compound(charged) = &raw.chunks()[6] else {
        panic!()
    };
    assert_eq!(charged.header.charge_raw, 51);
    assert_eq!(charged.header.entry_number, 1);
    assert_eq!(charged.header.timestamp_ole, 45_001.0);
}

#[test]
fn rejects_disagreement_between_formula_and_declared_composition() {
    let bad = plan(vec![group(
        "NiS",
        &["Ni", "O"],
        0,
        vec![function("Only", vec![range(298.15, 1000.0, 1.0)])],
    )]);
    assert!(matches!(
        bad.materialize_fresh(&native_templates(), &FdbFreshMaterialization::new(45_001.0)),
        Err(FdbMaterializeError::Formula { .. })
    ));
}

#[test]
fn preserves_formula_order_and_emits_empty_and_contiguous_functions() {
    let empty = FdbFunctionPlan::ExplicitZeroOrdinary(FdbFunctionIdentity::fresh_modern(
        "Empty",
        PhaseState::Solid,
    ));
    let plan = plan(vec![group(
        "ONi[-]",
        &["Ni", "O"],
        -1,
        vec![
            empty,
            function(
                "Ranged",
                vec![range(298.15, 700.0, 0.0), range(700.0, 1000.0, 0.0)],
            ),
        ],
    )]);
    let raw = plan
        .materialize_fresh(&native_templates(), &FdbFreshMaterialization::new(45_001.0))
        .unwrap();
    assert_eq!(
        raw.chunks().iter().map(RawChunk::id).collect::<Vec<_>>(),
        [9, 1, 7, 7, 2, 2]
    );
    let RawChunk::Compound(compound) = &raw.chunks()[1] else {
        panic!()
    };
    assert_eq!(compound.header.element_ids[..2], [8, 28]);
    assert_eq!(compound.header.charge_raw, 49);
    let RawChunk::PhaseOrdinary(empty_phase) = &raw.chunks()[2] else {
        panic!()
    };
    assert_eq!(empty_phase.phase_id_raw, 101);
    assert_eq!((empty_phase.enthalpy, empty_phase.entropy), (0.0, 0.0));
    let RawChunk::HeatCapacity { chunk: first, .. } = &raw.chunks()[4] else {
        panic!()
    };
    let RawChunk::HeatCapacity { chunk: second, .. } = &raw.chunks()[5] else {
        panic!()
    };
    assert_eq!((first.phase_id_raw, second.phase_id_raw), (102, 102));
    assert_eq!(first.temperature_max, second.temperature_min);
    assert_eq!(
        (first.header.entry_number, second.header.entry_number),
        (4, 5)
    );
}

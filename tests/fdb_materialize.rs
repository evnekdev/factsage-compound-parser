use factsage_compound_parser::fdb_build::{
    FdbAddedContribution, FdbAddedFunctionPlan, FdbAuxiliaryIntent, FdbBlockerClass, FdbBuildPlan,
    FdbChargeState, FdbCpTerm, FdbDatabaseMetadata, FdbElementAmount, FdbFormulaGroupPlan,
    FdbFreshMaterialization, FdbFunctionIdentity, FdbFunctionPlan, FdbFunctionRole,
    FdbMaterializeError, FdbNativeTemplates, FdbOrdinaryFunctionPlan, FdbStoichiometricAmount,
    FdbThermoRangePlan,
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

fn native_templates_with_id5() -> FdbNativeTemplates {
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
    cp[0] = 5;
    cp[17] = 3;
    cp[48..52].copy_from_slice(&101_i32.to_le_bytes());
    put_f64(&mut cp, 56, 298.15);
    put_f64(&mut cp, 64, 1000.0);
    put_f64(&mut cp, 136, 1.0);
    let example = RawDatabase::from_bytes(&[header, compound, phase, cp].concat()).unwrap();
    native_templates()
        .with_legacy_id5_example(&example)
        .unwrap()
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
            comment: String::new(),
            date_ole: 0.0,
        },
        groups,
    )
    .unwrap()
}

fn translated_pair(cp_terms: Vec<FdbCpTerm>, added: FdbAddedContribution) -> FdbBuildPlan {
    let base_identity = FdbFunctionIdentity {
        source_phase_id: "SYNX".into(),
        source_token: "synthetic".into(),
        source_g_index: 0,
        target_state: PhaseState::Solid,
        target_name: "SYNX_0000".into(),
        role: FdbFunctionRole::Base,
    };
    let added_identity = FdbFunctionIdentity {
        target_name: "SYNX_0000A".into(),
        role: FdbFunctionRole::Added,
        ..base_identity.clone()
    };
    let mut base = match function("unused", vec![range(298.15, 1000.0, 100.0)]) {
        FdbFunctionPlan::Ordinary(base) => base,
        _ => unreachable!(),
    };
    base.identity = base_identity.clone();
    base.ranges[0].cp_terms = cp_terms;
    let added = FdbAddedFunctionPlan {
        identity: added_identity,
        base: base_identity,
        contribution: added,
        auxiliary: FdbAuxiliaryIntent::Inactive,
    };
    FdbBuildPlan::new(
        FdbDatabaseMetadata {
            comment: String::new(),
            date_ole: 0.0,
        },
        vec![group(
            "NiS",
            &["Ni", "S"],
            0,
            vec![
                FdbFunctionPlan::Ordinary(base),
                FdbFunctionPlan::Added(added),
            ],
        )],
    )
    .unwrap()
}

#[test]
fn bounded_translated_writer_omits_explicit_zero_a_and_keeps_base_identity() {
    let plan = translated_pair(
        vec![FdbCpTerm {
            coefficient: 100.0,
            power: 0.0,
        }],
        FdbAddedContribution::ExplicitZero,
    );
    assert!(
        !plan
            .native_blockers()
            .iter()
            .any(|blocker| { blocker.class == FdbBlockerClass::Engineering })
    );
    let raw = plan
        .materialize_legacy_zero_added(&native_templates(), &FdbFreshMaterialization::new(45_001.0))
        .unwrap();
    assert_eq!(
        raw.chunks().iter().map(RawChunk::id).collect::<Vec<_>>(),
        [9, 1, 7, 2]
    );
    let RawChunk::PhaseOrdinary(phase) = &raw.chunks()[2] else {
        panic!("expected one translated base");
    };
    assert_eq!(phase.physical.phase_name_lossy(), "SYNX_0000");
}

#[test]
fn bounded_translated_writer_selects_id5_for_zero_cp() {
    let zero_cp = translated_pair(
        vec![
            FdbCpTerm {
                coefficient: 1.0,
                power: 0.0,
            },
            FdbCpTerm {
                coefficient: -1.0,
                power: 0.0,
            },
        ],
        FdbAddedContribution::ExplicitZero,
    );
    assert!(
        !zero_cp
            .native_blockers()
            .iter()
            .any(|blocker| blocker.class == FdbBlockerClass::Engineering)
    );
    assert!(matches!(
        zero_cp.materialize_legacy_zero_added(
            &native_templates(),
            &FdbFreshMaterialization::new(45_001.0)
        ),
        Err(FdbMaterializeError::Template(_))
    ));
    let raw = zero_cp
        .materialize_legacy_zero_added(
            &native_templates_with_id5(),
            &FdbFreshMaterialization::new(45_001.0),
        )
        .unwrap();
    assert_eq!(
        raw.chunks().iter().map(RawChunk::id).collect::<Vec<_>>(),
        [9, 1, 7, 5]
    );
    let RawChunk::HeatCapacity { chunk, .. } = &raw.chunks()[3] else {
        panic!("expected ID-5 range");
    };
    assert_eq!(chunk.coefficients, [0.0; 8]);
    assert_eq!(chunk.powers[0], 1.0);
}

#[test]
fn translated_mixed_cp_kinds_have_a_typed_refusal() {
    let source = translated_pair(vec![], FdbAddedContribution::ExplicitZero);
    let mut groups = source.groups().to_vec();
    let FdbFunctionPlan::Ordinary(base) = &mut groups[0].functions[0] else {
        panic!("expected translated base");
    };
    base.ranges[0].temperature_max_k = 500.0;
    base.ranges.push(FdbThermoRangePlan {
        temperature_min_k: 500.0,
        temperature_max_k: 1000.0,
        reference_enthalpy: -100.0 - 100.0 * (500.0 - 298.15),
        reference_entropy: 10.0 - 100.0 * (500.0_f64 / 298.15).ln(),
        cp_terms: vec![FdbCpTerm {
            coefficient: 100.0,
            power: 0.0,
        }],
    });
    let plan = FdbBuildPlan::new(source.metadata().clone(), groups).unwrap();
    assert!(matches!(
        plan.materialize_legacy_zero_added(
            &native_templates_with_id5(),
            &FdbFreshMaterialization::new(45_001.0)
        ),
        Err(FdbMaterializeError::Unsupported { .. })
    ));
}

#[test]
fn translated_nine_range_zero_cp_reparses_and_tenth_is_refused() {
    let source = translated_pair(vec![], FdbAddedContribution::ExplicitZero);
    let mut groups = source.groups().to_vec();
    let FdbFunctionPlan::Ordinary(base) = &mut groups[0].functions[0] else {
        panic!("expected translated base");
    };
    base.ranges = (0..9)
        .map(|index| FdbThermoRangePlan {
            temperature_min_k: 298.15 + f64::from(index) * 100.0,
            temperature_max_k: 398.15 + f64::from(index) * 100.0,
            reference_enthalpy: -100.0,
            reference_entropy: 10.0,
            cp_terms: vec![],
        })
        .collect();
    let plan = FdbBuildPlan::new(source.metadata().clone(), groups.clone()).unwrap();
    let raw = plan
        .materialize_legacy_zero_added(
            &native_templates_with_id5(),
            &FdbFreshMaterialization::new(45_001.0),
        )
        .unwrap();
    let reopened = RawDatabase::from_bytes(&raw.to_bytes().unwrap()).unwrap();
    assert_eq!(
        reopened
            .chunks()
            .iter()
            .filter(|chunk| chunk.id() == 5)
            .count(),
        9
    );
    assert!(
        DomainIndex::build(&reopened)
            .unwrap()
            .diagnostics()
            .is_empty()
    );

    let FdbFunctionPlan::Ordinary(base) = &mut groups[0].functions[0] else {
        unreachable!();
    };
    base.ranges.push(FdbThermoRangePlan {
        temperature_min_k: 1198.15,
        temperature_max_k: 1298.15,
        reference_enthalpy: -100.0,
        reference_entropy: 10.0,
        cp_terms: vec![],
    });
    let too_many = FdbBuildPlan::new(source.metadata().clone(), groups).unwrap();
    assert!(matches!(
        too_many.materialize_legacy_zero_added(
            &native_templates_with_id5(),
            &FdbFreshMaterialization::new(45_001.0)
        ),
        Err(FdbMaterializeError::Unsupported { .. })
    ));
}

#[test]
fn translated_distinct_formula_units_with_same_ratio_reparse_as_two_groups() {
    let source = translated_pair(vec![], FdbAddedContribution::ExplicitZero);
    let mut groups = source.groups().to_vec();
    let mut second = groups[0].clone();
    second.formula = "Ni2S2".into();
    for element in &mut second.elements {
        element.amount.numerator = 2;
    }
    for function in &mut second.functions {
        match function {
            FdbFunctionPlan::Ordinary(base) => {
                base.identity.source_g_index = 1;
                base.identity.source_token = "synthetic-two".into();
                base.identity.target_name = "SYNX_0001".into();
            }
            FdbFunctionPlan::Added(added) => {
                added.identity.source_g_index = 1;
                added.identity.source_token = "synthetic-two".into();
                added.identity.target_name = "SYNX_0001A".into();
                added.base.source_g_index = 1;
                added.base.source_token = "synthetic-two".into();
                added.base.target_name = "SYNX_0001".into();
            }
            FdbFunctionPlan::ExplicitZeroOrdinary(_) => unreachable!(),
        }
    }
    groups.push(second);
    let plan =
        FdbBuildPlan::new_legacy_translation_distinct_units(source.metadata().clone(), groups)
            .unwrap();
    let raw = plan
        .materialize_legacy_zero_added(
            &native_templates_with_id5(),
            &FdbFreshMaterialization::new(45_001.0),
        )
        .unwrap();
    let reopened = RawDatabase::from_bytes(&raw.to_bytes().unwrap()).unwrap();
    let index = DomainIndex::build(&reopened).unwrap();
    assert!(index.diagnostics().is_empty());
    assert_eq!(
        reopened
            .chunks()
            .iter()
            .filter(|chunk| chunk.id() == 1)
            .count(),
        2
    );
    assert_eq!(
        reopened
            .chunks()
            .iter()
            .filter(|chunk| chunk.id() == 7)
            .count(),
        2
    );
}

#[test]
fn bounded_translated_writer_refuses_active_a() {
    let active = translated_pair(
        vec![FdbCpTerm {
            coefficient: 100.0,
            power: 0.0,
        }],
        FdbAddedContribution::Thermodynamic {
            phase_enthalpy: 1.0,
            phase_entropy: 0.0,
            ranges: vec![range(298.15, 1000.0, 1.0)],
        },
    );
    assert!(
        active
            .native_blockers()
            .iter()
            .any(|blocker| blocker.class == FdbBlockerClass::Engineering)
    );
    assert!(matches!(
        active.materialize_legacy_zero_added(
            &native_templates(),
            &FdbFreshMaterialization::new(45_001.0)
        ),
        Err(FdbMaterializeError::Unsupported { .. })
    ));
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
fn bounded_writer_rejects_changes_to_the_native_database_header() {
    let templates = native_templates();
    for metadata in [
        FdbDatabaseMetadata {
            comment: "changed".into(),
            date_ole: templates.database_date_ole(),
        },
        FdbDatabaseMetadata {
            comment: String::new(),
            date_ole: templates.database_date_ole() + 1.0,
        },
    ] {
        let plan = FdbBuildPlan::new_fresh_modern(metadata, vec![]).unwrap();
        assert!(matches!(
            plan.materialize_fresh(&templates, &FdbFreshMaterialization::new(45_001.0)),
            Err(FdbMaterializeError::Unsupported { .. })
        ));
    }
}

#[test]
fn template_pair_must_share_the_same_database_header() {
    let raw = plan(vec![group(
        "NiS",
        &["Ni", "S"],
        0,
        vec![function("Only", vec![range(298.15, 1000.0, 1.0)])],
    )])
    .materialize_fresh(&native_templates(), &FdbFreshMaterialization::new(45_001.0))
    .unwrap();
    let mut bytes = raw.to_bytes().unwrap();
    let empty = RawDatabase::from_bytes(&bytes[..256]).unwrap();
    bytes[8..16].copy_from_slice(&1.0_f64.to_le_bytes());
    let changed_exemplar = RawDatabase::from_bytes(&bytes).unwrap();
    assert!(matches!(
        FdbNativeTemplates::from_examples(&empty, &changed_exemplar),
        Err(FdbMaterializeError::Template(_))
    ));
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
fn hydrogen_uses_the_last_native_slot_without_rewriting_the_formula_label() {
    let plan = plan(vec![FdbFormulaGroupPlan {
        formula: "H2O".into(),
        elements: vec![
            FdbElementAmount {
                symbol: "H".into(),
                amount: FdbStoichiometricAmount {
                    numerator: 2,
                    denominator: 1,
                },
            },
            FdbElementAmount {
                symbol: "O".into(),
                amount: FdbStoichiometricAmount {
                    numerator: 1,
                    denominator: 1,
                },
            },
        ],
        charge: FdbChargeState::new(0),
        energy_unit: EnergyUnit::Joules,
        pressure_unit: PressureUnit::Bars,
        functions: vec![function("Only", vec![range(298.15, 1000.0, 1.0)])],
    }]);
    let raw = plan
        .materialize_fresh(&native_templates(), &FdbFreshMaterialization::new(45_001.0))
        .unwrap();
    let RawChunk::Compound(compound) = &raw.chunks()[1] else {
        panic!()
    };
    assert_eq!(compound.formula_name_lossy(), "H2O");
    assert_eq!(compound.header.element_ids[..2], [8, 1]);
    assert_eq!(compound.header.element_coefficients[..2], [1, 2]);
    assert_eq!(compound.real_stoichiometric_coefficients[..2], [1.0, 2.0]);
    for chunk in &raw.chunks()[2..] {
        match chunk {
            RawChunk::PhaseOrdinary(phase) => {
                assert_eq!(phase.header.element_ids[..2], [8, 1]);
                assert_eq!(phase.header.element_coefficients[..2], [1, 2]);
            }
            RawChunk::HeatCapacity { chunk, .. } => {
                assert_eq!(chunk.header.element_ids[..2], [8, 1]);
                assert_eq!(chunk.header.element_coefficients[..2], [1, 2]);
            }
            _ => panic!(),
        }
    }
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

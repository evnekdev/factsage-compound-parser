use factsage_compound_parser::fdb_build::{
    FdbAddedContribution, FdbAddedFunctionPlan, FdbAuxiliaryIntent, FdbBlockerClass, FdbBuildError,
    FdbBuildPlan, FdbChargeState, FdbConstructionProfile, FdbCpTerm, FdbDatabaseMetadata,
    FdbElementAmount, FdbFormulaGroupPlan, FdbFunctionIdentity, FdbFunctionPlan, FdbFunctionRole,
    FdbOrdinaryFunctionPlan, FdbStoichiometricAmount, FdbThermoRangePlan,
};
use factsage_compound_parser::{EnergyUnit, PhaseState, PressureUnit};
use proptest::prelude::*;

fn metadata() -> FdbDatabaseMetadata {
    FdbDatabaseMetadata {
        comment: "synthetic FDB".into(),
        date_ole: 45_000.0,
    }
}

fn range() -> FdbThermoRangePlan {
    FdbThermoRangePlan {
        temperature_min_k: 298.15,
        temperature_max_k: 1000.0,
        reference_enthalpy: -100.0,
        reference_entropy: 10.0,
        cp_terms: vec![
            FdbCpTerm {
                coefficient: 0.0,
                power: 0.0,
            };
            4
        ],
    }
}

fn identity(phase: &str, source: &str, index: usize, role: FdbFunctionRole) -> FdbFunctionIdentity {
    FdbFunctionIdentity {
        source_phase_id: phase.into(),
        source_token: source.into(),
        source_g_index: index,
        target_state: PhaseState::Solid,
        target_name: format!(
            "{phase}_{index:04}{}",
            if role == FdbFunctionRole::Added {
                "A"
            } else {
                ""
            }
        ),
        role,
    }
}

fn pair(phase: &str, source: &str, index: usize) -> [FdbFunctionPlan; 2] {
    let base = identity(phase, source, index, FdbFunctionRole::Base);
    let added = identity(phase, source, index, FdbFunctionRole::Added);
    [
        FdbFunctionPlan::Ordinary(FdbOrdinaryFunctionPlan {
            identity: base.clone(),
            phase_enthalpy: -100.0,
            phase_entropy: 10.0,
            ranges: vec![range()],
            auxiliary: FdbAuxiliaryIntent::Inactive,
        }),
        FdbFunctionPlan::Added(FdbAddedFunctionPlan {
            identity: added,
            base,
            contribution: FdbAddedContribution::ExplicitZero,
            auxiliary: FdbAuxiliaryIntent::Inactive,
        }),
    ]
}

fn group(functions: Vec<FdbFunctionPlan>) -> FdbFormulaGroupPlan {
    FdbFormulaGroupPlan {
        formula: "NiS".into(),
        elements: vec![
            FdbElementAmount {
                symbol: "Ni".into(),
                amount: FdbStoichiometricAmount {
                    numerator: 1,
                    denominator: 1,
                },
            },
            FdbElementAmount {
                symbol: "S".into(),
                amount: FdbStoichiometricAmount {
                    numerator: 1,
                    denominator: 1,
                },
            },
        ],
        charge: FdbChargeState::new(0),
        energy_unit: EnergyUnit::Joules,
        pressure_unit: PressureUnit::Bars,
        functions,
    }
}

fn plan(functions: Vec<FdbFunctionPlan>) -> Result<FdbBuildPlan, FdbBuildError> {
    FdbBuildPlan::new(metadata(), vec![group(functions)])
}

#[test]
fn fresh_modern_names_are_independent_of_legacy_provenance() {
    let fresh = FdbFunctionIdentity::fresh_modern("Caller Function", PhaseState::Solid);
    let function = FdbFunctionPlan::Ordinary(FdbOrdinaryFunctionPlan {
        identity: fresh.clone(),
        phase_enthalpy: -100.0,
        phase_entropy: 10.0,
        ranges: vec![range()],
        auxiliary: FdbAuxiliaryIntent::Inactive,
    });
    let built =
        FdbBuildPlan::new_fresh_modern(metadata(), vec![group(vec![function.clone()])]).unwrap();
    assert_eq!(built.profile(), FdbConstructionProfile::FreshModern);
    assert_eq!(
        built.groups()[0].functions[0].identity().target_name,
        "Caller Function"
    );
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![group(vec![function])]),
        Err(FdbBuildError::InvalidField {
            field: "source_phase_id",
            ..
        })
    ));

    let mut false_provenance = fresh;
    false_provenance.source_phase_id = "LEGACY".into();
    assert!(matches!(
        FdbBuildPlan::new_fresh_modern(
            metadata(),
            vec![group(vec![FdbFunctionPlan::ExplicitZeroOrdinary(
                false_provenance
            )])]
        ),
        Err(FdbBuildError::InvalidField {
            field: "source_provenance",
            ..
        })
    ));
}

#[test]
fn fresh_groups_keep_charge_identity_when_function_names_repeat() {
    let function = || {
        FdbFunctionPlan::Ordinary(FdbOrdinaryFunctionPlan {
            identity: FdbFunctionIdentity::fresh_modern("SharedName", PhaseState::Solid),
            phase_enthalpy: -100.0,
            phase_entropy: 10.0,
            ranges: vec![range()],
            auxiliary: FdbAuxiliaryIntent::Inactive,
        })
    };
    let mut neutral = group(vec![function()]);
    neutral.formula = "SyntheticNeutral".into();
    let mut charged = group(vec![function()]);
    charged.formula = "SyntheticCharged".into();
    charged.charge = FdbChargeState::new(1);

    let built = FdbBuildPlan::new_fresh_modern(metadata(), vec![neutral, charged]).unwrap();
    assert_eq!(built.groups().len(), 2);
    assert_eq!(built.groups()[0].elements, built.groups()[1].elements);
    assert_ne!(built.groups()[0].charge, built.groups()[1].charge);
    assert_eq!(
        built.groups()[0].functions[0].identity().target_name,
        built.groups()[1].functions[0].identity().target_name
    );
    assert!(
        !built
            .native_blockers()
            .iter()
            .any(|blocker| blocker.field == "RawCommonHeader.charge_raw")
    );
}

#[test]
fn fdb_charge_uses_the_native_neutral_offset_rule() {
    for (semantic, raw) in [(-50, 0), (-1, 49), (0, 50), (1, 51), (50, 100)] {
        let charge = FdbChargeState::new(semantic);
        assert_eq!(charge.fdb_raw_byte(), Some(raw));
        assert_eq!(
            FdbChargeState::from_fdb_raw_byte(raw),
            Some(charge)
        );
    }
    for unsupported in [i32::MIN, -51, 51, i32::MAX] {
        assert_eq!(
            FdbChargeState::new(unsupported).fdb_raw_byte(),
            None
        );
    }
    for unsupported in [101, 127, 128, 255] {
        assert_eq!(
            FdbChargeState::from_fdb_raw_byte(unsupported),
            None
        );
    }

    let function = FdbFunctionPlan::Ordinary(FdbOrdinaryFunctionPlan {
        identity: FdbFunctionIdentity::fresh_modern("SyntheticSolid", PhaseState::Solid),
        phase_enthalpy: -100.0,
        phase_entropy: 10.0,
        ranges: vec![range()],
        auxiliary: FdbAuxiliaryIntent::Inactive,
    });
    let mut admitted = group(vec![function.clone()]);
    admitted.charge = FdbChargeState::new(50);
    let built = FdbBuildPlan::new_fresh_modern(metadata(), vec![admitted]).unwrap();
    assert!(!built.native_blockers().iter().any(|blocker| {
        blocker.field == "RawCommonHeader.charge_raw"
            || blocker.field == "RawCommonHeader.charge_raw range"
    }));

    for unsupported in [-51, 51] {
        let mut group = group(vec![function.clone()]);
        group.charge = FdbChargeState::new(unsupported);
        assert!(matches!(
            FdbBuildPlan::new_fresh_modern(metadata(), vec![group]),
            Err(FdbBuildError::InvalidField {
                field: "charge",
                ..
            })
        ));
    }
}

#[test]
fn fresh_nonzero_cp_uses_a_distinct_kind_blocker_policy() {
    let mut nonzero_range = range();
    nonzero_range.cp_terms[0].coefficient = 1.0;
    let identity = FdbFunctionIdentity::fresh_modern("FreshSolid", PhaseState::Solid);
    let function = FdbFunctionPlan::Ordinary(FdbOrdinaryFunctionPlan {
        identity,
        phase_enthalpy: -100.0,
        phase_entropy: 10.0,
        ranges: vec![nonzero_range],
        auxiliary: FdbAuxiliaryIntent::Inactive,
    });
    let fresh = FdbBuildPlan::new_fresh_modern(metadata(), vec![group(vec![function])]).unwrap();
    assert!(
        !fresh
            .native_blockers()
            .iter()
            .any(|b| b.field.contains("CP.kind selection"))
    );
    for resolved in [
        "RawCommonHeader.entry_number",
        "RawCommonHeader.reference[2]",
        "ID-1.compound_name",
        "ID-7.phase_id_raw allocation",
    ] {
        assert!(!fresh.native_blockers().iter().any(|b| b.field == resolved));
    }
    let zero_range_function = FdbFunctionPlan::Ordinary(FdbOrdinaryFunctionPlan {
        identity: FdbFunctionIdentity::fresh_modern("ZeroCp", PhaseState::Solid),
        phase_enthalpy: -100.0,
        phase_entropy: 10.0,
        ranges: vec![range()],
        auxiliary: FdbAuxiliaryIntent::Inactive,
    });
    let zero_cp =
        FdbBuildPlan::new_fresh_modern(metadata(), vec![group(vec![zero_range_function])]).unwrap();
    assert!(
        !zero_cp
            .native_blockers()
            .iter()
            .any(|b| b.field.contains("CP.kind selection"))
    );
    let zero = FdbBuildPlan::new_fresh_modern(
        metadata(),
        vec![group(vec![FdbFunctionPlan::ExplicitZeroOrdinary(
            FdbFunctionIdentity::fresh_modern("EmptyFresh", PhaseState::Solid),
        )])],
    )
    .unwrap();
    assert!(
        !zero
            .native_blockers()
            .iter()
            .any(|b| b.field.contains("zero-base"))
    );
    assert!(zero.native_blockers().iter().any(|b| {
        b.field == "fresh empty-function verification" && b.class == FdbBlockerClass::Engineering
    }));
}

fn iron_oxide_group(label: &str, charge: i32, phase: &str) -> FdbFormulaGroupPlan {
    let mut result = group(pair(phase, phase, 0).to_vec());
    result.formula = label.into();
    result.charge = FdbChargeState::new(charge);
    result.elements = vec![
        FdbElementAmount {
            symbol: "Fe".into(),
            amount: FdbStoichiometricAmount {
                numerator: 3,
                denominator: 1,
            },
        },
        FdbElementAmount {
            symbol: "O".into(),
            amount: FdbStoichiometricAmount {
                numerator: 4,
                denominator: 1,
            },
        },
    ];
    result
}

#[test]
fn charged_and_neutral_equal_compositions_are_distinct_groups() {
    let neutral = iron_oxide_group("Fe3O4", 0, "NEUT");
    let charged = iron_oxide_group("Fe3O4[+]", 1, "PLUS");
    assert_ne!(
        neutral.semantic_identity().unwrap(),
        charged.semantic_identity().unwrap()
    );
    let built = FdbBuildPlan::new(metadata(), vec![neutral, charged]).unwrap();
    assert_eq!(built.groups().len(), 2);
    assert_eq!(built.groups()[1].charge.value(), 1);
}

#[test]
fn same_function_name_is_qualified_by_distinct_charge_group() {
    let mut neutral = iron_oxide_group("Fe3O4", 0, "PHAS");
    let mut charged = iron_oxide_group("Fe3O4[+]", 1, "PHAS");
    for function in &mut charged.functions {
        match function {
            FdbFunctionPlan::Ordinary(base) => base.identity.source_token = "charged-source".into(),
            FdbFunctionPlan::Added(added) => {
                added.identity.source_token = "charged-source".into();
                added.base.source_token = "charged-source".into();
            }
            FdbFunctionPlan::ExplicitZeroOrdinary(_) => unreachable!(),
        }
    }
    let built = FdbBuildPlan::new(metadata(), vec![neutral.clone(), charged]).unwrap();
    assert_eq!(
        built.groups()[0].functions[0].identity().target_name,
        "PHAS_0000"
    );
    assert_eq!(
        built.groups()[1].functions[0].identity().target_name,
        "PHAS_0000"
    );
    neutral.functions.push(neutral.functions[0].clone());
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![neutral]),
        Err(FdbBuildError::DuplicateIdentity { .. })
    ));
}

#[test]
fn opposite_charges_remain_distinct_without_parsing_label_syntax() {
    let positive = iron_oxide_group("X[+]", 1, "PLUS");
    let negative = iron_oxide_group("X[-]", -1, "MINU");
    assert!(FdbBuildPlan::new(metadata(), vec![positive, negative]).is_ok());
}

#[test]
fn proportional_exact_rationals_with_same_charge_have_one_identity() {
    let first = iron_oxide_group("Fe3O4[+]", 1, "PLUS");
    let mut second = iron_oxide_group("Fe6O8[+]", 1, "OTHR");
    second.elements[0].amount = FdbStoichiometricAmount {
        numerator: 6,
        denominator: 1,
    };
    second.elements[1].amount = FdbStoichiometricAmount {
        numerator: 8,
        denominator: 1,
    };
    second.elements.reverse();
    assert_eq!(
        first.semantic_identity().unwrap(),
        second.semantic_identity().unwrap()
    );
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![first, second]),
        Err(FdbBuildError::AmbiguousGroup { .. })
    ));
    let mut fractional = iron_oxide_group("fractional", 1, "FRAC");
    fractional.elements[0].amount = FdbStoichiometricAmount {
        numerator: 3,
        denominator: 2,
    };
    fractional.elements[1].amount = FdbStoichiometricAmount {
        numerator: 2,
        denominator: 1,
    };
    assert_eq!(
        fractional.semantic_identity().unwrap(),
        iron_oxide_group("Fe3O4[+]", 1, "PLUS")
            .semantic_identity()
            .unwrap()
    );
}

#[test]
fn exact_ratio_products_accept_u64_limits_without_overflow() {
    let mut group = group(pair("PHAS", "source", 0).to_vec());
    group.elements[0].amount = FdbStoichiometricAmount {
        numerator: u64::MAX,
        denominator: u64::MAX - 1,
    };
    group.elements[1].amount = FdbStoichiometricAmount {
        numerator: u64::MAX - 1,
        denominator: u64::MAX,
    };
    assert!(group.semantic_identity().is_ok());
}

proptest! {
    #[test]
    fn exact_ratio_key_ignores_order_and_common_scale(
        first in 1_u64..1_000_000,
        second in 1_u64..1_000_000,
        scale in 1_u64..1_000,
        charge in -20_i32..20,
    ) {
        let mut left = group(pair("LEFT", "left", 0).to_vec());
        left.elements[0].amount.numerator = first;
        left.elements[1].amount.numerator = second;
        left.charge = FdbChargeState::new(charge);
        let mut right = left.clone();
        right.elements[0].amount.numerator *= scale;
        right.elements[1].amount.numerator *= scale;
        right.elements.reverse();
        prop_assert_eq!(left.semantic_identity().unwrap(), right.semantic_identity().unwrap());
        right.charge = FdbChargeState::new(charge + 1);
        prop_assert_ne!(left.semantic_identity().unwrap(), right.semantic_identity().unwrap());
    }
}

#[test]
fn same_label_with_conflicting_charge_and_alias_with_same_identity_fail() {
    let neutral = iron_oxide_group("Fe3O4", 0, "NEUT");
    let mut same_label_charged = iron_oxide_group("Fe3O4", 1, "PLUS");
    assert!(matches!(
        FdbBuildPlan::new(
            metadata(),
            vec![neutral.clone(), same_label_charged.clone()]
        ),
        Err(FdbBuildError::AmbiguousGroup { .. })
    ));
    same_label_charged.formula = "magnetite-alias".into();
    same_label_charged.charge = FdbChargeState::new(0);
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![neutral, same_label_charged]),
        Err(FdbBuildError::AmbiguousGroup { .. })
    ));
}

#[test]
fn equal_values_remain_distinct_across_source_phases_and_one_global_group() {
    let functions = [pair("MHEX", "solution-1", 4), pair("MILL", "solution-2", 0)].concat();
    let built = plan(functions).unwrap();
    assert_eq!(built.groups().len(), 1);
    assert_eq!(built.groups()[0].functions.len(), 4);
    assert_eq!(
        built.groups()[0].functions[0].identity().target_name,
        "MHEX_0004"
    );
    assert_eq!(
        built.groups()[0].functions[2].identity().target_name,
        "MILL_0000"
    );
}

#[test]
fn equal_values_same_phase_different_g_entries_remain_distinct() {
    let built = plan([pair("PHAS", "solution", 0), pair("PHAS", "solution", 1)].concat()).unwrap();
    let names: Vec<_> = built.groups()[0]
        .functions
        .iter()
        .map(|f| f.identity().target_name.as_str())
        .collect();
    assert_eq!(
        names,
        ["PHAS_0000", "PHAS_0000A", "PHAS_0001", "PHAS_0001A"]
    );
}

#[test]
fn base_a_pairing_is_exact_and_cross_pairing_fails() {
    let functions = [pair("PHAS", "solution", 0), pair("PHAS", "solution", 1)].concat();
    assert!(plan(functions.clone()).is_ok());
    let mut crossed = functions;
    let FdbFunctionPlan::Added(added) = &mut crossed[1] else {
        panic!()
    };
    added.base = identity("PHAS", "solution", 1, FdbFunctionRole::Base);
    assert!(matches!(
        plan(crossed),
        Err(FdbBuildError::InvalidPairing { .. })
    ));
}

#[test]
fn typed_pairing_rejects_missing_base_wrong_source_index_and_cross_group() {
    let added_only = pair("PHAS", "source", 0)[1].clone();
    assert!(matches!(
        plan(vec![added_only]),
        Err(FdbBuildError::InvalidPairing { .. })
    ));

    let mut functions = [pair("PHAS", "source", 0), pair("PHAS", "source", 1)].concat();
    let FdbFunctionPlan::Added(added) = &mut functions[1] else {
        panic!()
    };
    added.base = identity("PHAS", "source", 1, FdbFunctionRole::Base);
    assert!(matches!(
        plan(functions),
        Err(FdbBuildError::InvalidPairing { .. })
    ));

    let mut first = group(vec![pair("PHAS", "source", 0)[0].clone()]);
    first.formula = "NiS".into();
    let mut second = group(vec![pair("PHAS", "source", 0)[1].clone()]);
    second.formula = "Ni2S".into();
    second.elements[0].amount.numerator = 2;
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![first, second]),
        Err(FdbBuildError::InvalidPairing { .. })
    ));

    let mut state_mismatch = pair("PHAS", "source", 0).to_vec();
    let FdbFunctionPlan::Added(added) = &mut state_mismatch[1] else {
        panic!()
    };
    added.identity.target_state = PhaseState::Gas;
    assert!(matches!(
        plan(state_mismatch),
        Err(FdbBuildError::InvalidPairing { .. })
    ));
}

#[test]
fn target_phase_state_is_explicit_and_shared_by_each_pair() {
    let mut gas_pair = pair("PHAS", "source", 0).to_vec();
    for function in &mut gas_pair {
        match function {
            FdbFunctionPlan::Ordinary(base) => base.identity.target_state = PhaseState::Gas,
            FdbFunctionPlan::Added(added) => {
                added.identity.target_state = PhaseState::Gas;
                added.base.target_state = PhaseState::Gas;
            }
            FdbFunctionPlan::ExplicitZeroOrdinary(_) => unreachable!(),
        }
    }
    let built = plan(gas_pair).unwrap();
    assert_eq!(
        built.groups()[0].functions[0].identity().target_state,
        PhaseState::Gas
    );
    assert!(
        built
            .native_blockers()
            .iter()
            .any(|blocker| blocker.field == "ID-7.phase_id_raw allocation")
    );
}

#[test]
fn two_a_claims_and_wrong_source_token_are_rejected() {
    let mut functions = pair("PHAS", "source", 0).to_vec();
    functions.push(functions[1].clone());
    assert!(matches!(
        plan(functions),
        Err(FdbBuildError::DuplicateIdentity { .. })
    ));

    let mut functions = pair("PHAS", "source", 0).to_vec();
    let FdbFunctionPlan::Added(added) = &mut functions[1] else {
        panic!()
    };
    added.base.source_token = "other-solution".into();
    assert!(matches!(
        plan(functions),
        Err(FdbBuildError::InvalidPairing { .. })
    ));
}

#[test]
fn matching_base_and_a_values_do_not_collapse_identities() {
    let mut functions = pair("PHAS", "source", 0).to_vec();
    let FdbFunctionPlan::Added(added) = &mut functions[1] else {
        panic!()
    };
    added.contribution = FdbAddedContribution::Thermodynamic {
        phase_enthalpy: -100.0,
        phase_entropy: 10.0,
        ranges: vec![range()],
    };
    let built = plan(functions).unwrap();
    assert_ne!(
        built.groups()[0].functions[0].identity(),
        built.groups()[0].functions[1].identity()
    );
}

#[test]
fn source_phase_case_and_g_index_limits_are_preserved() {
    let built = plan([pair("Phas", "source-a", 0), pair("PHAS", "source-b", 0)].concat()).unwrap();
    assert_ne!(
        built.groups()[0].functions[0].identity().target_name,
        built.groups()[0].functions[2].identity().target_name
    );
    assert!(plan(pair("PHAS", "source", 9999).to_vec()).is_ok());
    assert!(matches!(
        plan(pair("PHAS", "source", 10000).to_vec()),
        Err(FdbBuildError::InvalidField {
            field: "source_g_index",
            ..
        })
    ));
    assert!(matches!(
        plan(pair("P\nHAS", "source", 0).to_vec()),
        Err(FdbBuildError::InvalidField {
            field: "source_phase_id",
            ..
        })
    ));
}

#[test]
fn function_variant_cannot_disagree_with_structural_role() {
    let mut functions = pair("PHAS", "solution", 0).to_vec();
    let FdbFunctionPlan::Ordinary(base) = &mut functions[0] else {
        panic!()
    };
    base.identity.role = FdbFunctionRole::Added;
    base.identity.target_name.push('A');
    assert!(matches!(
        plan(functions),
        Err(FdbBuildError::InvalidField { field: "role", .. })
    ));
}

#[test]
fn zero_a_is_retained_and_nonzero_a_is_representable() {
    let mut functions = pair("PHAS", "solution", 0).to_vec();
    let built = plan(functions.clone()).unwrap();
    assert!(matches!(
        &built.groups()[0].functions[1],
        FdbFunctionPlan::Added(FdbAddedFunctionPlan {
            contribution: FdbAddedContribution::ExplicitZero,
            ..
        })
    ));
    let FdbFunctionPlan::Added(added) = &mut functions[1] else {
        panic!()
    };
    added.contribution = FdbAddedContribution::Thermodynamic {
        phase_enthalpy: -100.0,
        phase_entropy: 10.0,
        ranges: vec![range()],
    };
    assert!(plan(functions).is_ok());
}

#[test]
fn zero_base_and_zero_a_still_have_distinct_identities() {
    let mut functions = pair("PHAS", "solution", 0).to_vec();
    let base = functions[0].identity().clone();
    functions[0] = FdbFunctionPlan::ExplicitZeroOrdinary(base);
    let built = plan(functions).unwrap();
    assert_eq!(
        built.groups()[0].functions[0].identity().target_name,
        "PHAS_0000"
    );
    assert_eq!(
        built.groups()[0].functions[1].identity().target_name,
        "PHAS_0000A"
    );
    assert!(matches!(
        built.groups()[0].functions[0],
        FdbFunctionPlan::ExplicitZeroOrdinary(_)
    ));
}

#[test]
fn duplicate_name_and_duplicate_source_role_are_rejected() {
    let mut functions = pair("PHAS", "solution", 0).to_vec();
    functions.push(functions[0].clone());
    assert!(matches!(
        plan(functions),
        Err(FdbBuildError::DuplicateIdentity { .. })
    ));
    let mut functions = pair("PHAS", "solution", 0).to_vec();
    let FdbFunctionPlan::Ordinary(base) = &mut functions[0] else {
        panic!()
    };
    base.identity.source_token = "other".into();
    assert!(matches!(
        plan(functions),
        Err(FdbBuildError::InvalidPairing { .. })
    ));
}

#[test]
fn nonfinite_h_s_cp_bounds_and_date_are_rejected() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut functions = pair("PHAS", "solution", 0).to_vec();
        let FdbFunctionPlan::Ordinary(base) = &mut functions[0] else {
            panic!()
        };
        base.phase_enthalpy = invalid;
        assert!(matches!(
            plan(functions),
            Err(FdbBuildError::InvalidThermodynamics { .. })
        ));

        let mut functions = pair("PHAS", "solution", 0).to_vec();
        let FdbFunctionPlan::Ordinary(base) = &mut functions[0] else {
            panic!()
        };
        base.ranges[0].cp_terms[3].power = invalid;
        assert!(matches!(
            plan(functions),
            Err(FdbBuildError::InvalidThermodynamics { .. })
        ));

        let mut functions = pair("PHAS", "solution", 0).to_vec();
        let FdbFunctionPlan::Ordinary(base) = &mut functions[0] else {
            panic!()
        };
        base.ranges[0].reference_entropy = invalid;
        assert!(matches!(
            plan(functions),
            Err(FdbBuildError::InvalidThermodynamics { .. })
        ));

        let mut meta = metadata();
        meta.date_ole = invalid;
        assert!(matches!(
            FdbBuildPlan::new(meta, vec![group(pair("PHAS", "solution", 0).to_vec())]),
            Err(FdbBuildError::InvalidField {
                field: "date_ole",
                ..
            })
        ));
    }
    let mut meta = metadata();
    meta.date_ole = 3_000_000.0;
    assert!(matches!(
        FdbBuildPlan::new(meta, vec![group(pair("PHAS", "solution", 0).to_vec())]),
        Err(FdbBuildError::InvalidField {
            field: "date_ole",
            ..
        })
    ));
}

#[test]
fn invalid_temperature_domains_gaps_and_discontinuities_are_rejected() {
    for (lower, upper) in [(0.0, 1000.0), (1000.0, 900.0), (f64::NAN, 1000.0)] {
        let mut functions = pair("PHAS", "solution", 0).to_vec();
        let FdbFunctionPlan::Ordinary(base) = &mut functions[0] else {
            panic!()
        };
        base.ranges[0].temperature_min_k = lower;
        base.ranges[0].temperature_max_k = upper;
        assert!(matches!(
            plan(functions),
            Err(FdbBuildError::InvalidThermodynamics { .. })
        ));
    }
    let mut functions = pair("PHAS", "solution", 0).to_vec();
    let FdbFunctionPlan::Ordinary(base) = &mut functions[0] else {
        panic!()
    };
    let mut second = range();
    second.temperature_min_k = 1001.0;
    second.temperature_max_k = 1500.0;
    base.ranges.push(second.clone());
    assert!(matches!(
        plan(functions.clone()),
        Err(FdbBuildError::InvalidThermodynamics { .. })
    ));
    let FdbFunctionPlan::Ordinary(base) = &mut functions[0] else {
        panic!()
    };
    base.ranges[1].temperature_min_k = 1000.0;
    base.ranges[1].reference_enthalpy += 1.0;
    assert!(matches!(
        plan(functions),
        Err(FdbBuildError::InvalidThermodynamics { .. })
    ));

    let mut functions = pair("PHAS", "solution", 0).to_vec();
    let FdbFunctionPlan::Ordinary(base) = &mut functions[0] else {
        panic!()
    };
    let mut second = range();
    second.temperature_min_k = 999.0;
    second.temperature_max_k = 1500.0;
    base.ranges.push(second);
    assert!(matches!(
        plan(functions),
        Err(FdbBuildError::InvalidThermodynamics { .. })
    ));
}

#[test]
fn cp_capacity_is_checked_without_requiring_native_padding() {
    assert!(plan(pair("PHAS", "source", 0).to_vec()).is_ok());
    let mut functions = pair("PHAS", "source", 0).to_vec();
    let FdbFunctionPlan::Ordinary(base) = &mut functions[0] else {
        panic!()
    };
    base.ranges[0].cp_terms = vec![
        FdbCpTerm {
            coefficient: 0.0,
            power: 0.0
        };
        9
    ];
    assert!(matches!(
        plan(functions),
        Err(FdbBuildError::InvalidThermodynamics { field, .. }) if field.contains("cp_terms")
    ));
}

#[test]
fn deterministic_order_and_global_stoichiometry_are_enforced() {
    let functions = [pair("PHAS", "source", 0), pair("PHAS", "source", 1)].concat();
    let first = plan(functions.clone()).unwrap();
    let second = plan(functions).unwrap();
    assert_eq!(first, second);
    let mut duplicate = group(pair("OTHR", "other", 0).to_vec());
    duplicate.formula = "Ni1S1".into();
    duplicate.elements.reverse();
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![first.groups()[0].clone(), duplicate]),
        Err(FdbBuildError::AmbiguousGroup { .. })
    ));

    let mut proportional = group(pair("OTHR", "other", 0).to_vec());
    proportional.formula = "Ni2S2".into();
    for element in &mut proportional.elements {
        element.amount.numerator *= 2;
    }
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![first.groups()[0].clone(), proportional]),
        Err(FdbBuildError::AmbiguousGroup { .. })
    ));
}

#[test]
fn one_formula_label_cannot_claim_two_compositions() {
    let first = group(pair("PHAS", "source", 0).to_vec());
    let mut second = group(pair("OTHR", "other", 0).to_vec());
    second.elements[0].amount.numerator = 2;
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![first, second]),
        Err(FdbBuildError::AmbiguousGroup { .. })
    ));
}

#[test]
fn unsupported_auxiliary_and_missing_a_fail_closed() {
    let mut functions = pair("PHAS", "source", 0).to_vec();
    let FdbFunctionPlan::Ordinary(base) = &mut functions[0] else {
        panic!()
    };
    base.auxiliary = FdbAuxiliaryIntent::Active {
        field: "magnetic_temperature".into(),
    };
    assert!(
        matches!(plan(functions), Err(FdbBuildError::UnsupportedAuxiliary { field, .. }) if field == "magnetic_temperature")
    );
    assert!(matches!(
        plan(vec![pair("PHAS", "source", 0)[0].clone()]),
        Err(FdbBuildError::InvalidPairing { .. })
    ));
}

#[test]
fn unknown_units_and_each_active_auxiliary_family_fail_closed() {
    let mut bad_energy = group(pair("PHAS", "source", 0).to_vec());
    bad_energy.energy_unit = EnergyUnit::Unknown(9);
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![bad_energy]),
        Err(FdbBuildError::InvalidField {
            field: "energy_unit",
            ..
        })
    ));
    let mut bad_pressure = group(pair("PHAS", "source", 0).to_vec());
    bad_pressure.pressure_unit = PressureUnit::Unknown(9);
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![bad_pressure]),
        Err(FdbBuildError::InvalidField {
            field: "pressure_unit",
            ..
        })
    ));
    for field in ["magnetic_temperature", "density_raw", "ID-11"] {
        let mut functions = pair("PHAS", "source", 0).to_vec();
        let FdbFunctionPlan::Ordinary(base) = &mut functions[0] else {
            panic!()
        };
        base.auxiliary = FdbAuxiliaryIntent::Active {
            field: field.into(),
        };
        assert!(matches!(
            plan(functions),
            Err(FdbBuildError::UnsupportedAuxiliary { field: found, .. }) if found == field
        ));
    }
    let mut functions = pair("PHAS", "source", 0).to_vec();
    let FdbFunctionPlan::Ordinary(base) = &mut functions[0] else {
        panic!()
    };
    base.auxiliary = FdbAuxiliaryIntent::Active { field: " ".into() };
    assert!(matches!(
        plan(functions),
        Err(FdbBuildError::InvalidField {
            field: "auxiliary.field",
            ..
        })
    ));
}

#[test]
fn added_phase_anchors_and_exact_amount_inputs_are_checked() {
    let mut functions = pair("PHAS", "source", 0).to_vec();
    let FdbFunctionPlan::Added(added) = &mut functions[1] else {
        panic!()
    };
    added.contribution = FdbAddedContribution::Thermodynamic {
        phase_enthalpy: f64::NAN,
        phase_entropy: 0.0,
        ranges: vec![range()],
    };
    assert!(matches!(
        plan(functions),
        Err(FdbBuildError::InvalidThermodynamics { .. })
    ));

    let mut bad_amount = group(pair("PHAS", "source", 0).to_vec());
    bad_amount.elements[0].amount.denominator = 0;
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![bad_amount]),
        Err(FdbBuildError::AmbiguousGroup { .. })
    ));

    let mut bad_symbol = group(pair("PHAS", "source", 0).to_vec());
    bad_symbol.elements[0].symbol = "NI".into();
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![bad_symbol]),
        Err(FdbBuildError::AmbiguousGroup { .. })
    ));
}

#[test]
fn native_element_slot_capacity_is_checked() {
    let mut oversized = group(pair("PHAS", "source", 0).to_vec());
    oversized.elements = ["H", "B", "C", "N", "O", "F", "P", "S"]
        .into_iter()
        .map(|symbol| FdbElementAmount {
            symbol: symbol.into(),
            amount: FdbStoichiometricAmount {
                numerator: 1,
                denominator: 1,
            },
        })
        .collect();
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![oversized]),
        Err(FdbBuildError::AmbiguousGroup { .. })
    ));
}

#[test]
fn semantic_validity_is_separate_from_native_materialization_readiness() {
    let built = plan(pair("PHAS", "source", 0).to_vec()).unwrap();
    assert!(built.validate().is_ok());
    assert!(
        built
            .native_blockers()
            .iter()
            .any(|blocker| blocker.object == "PHAS_0000A" && blocker.field.contains("zero-A"))
    );

    let blockers = built.native_blockers();
    assert!(blockers.iter().any(|blocker| {
        blocker.class == FdbBlockerClass::Engineering
            && !blocker.user_evidence_can_unblock
            && blocker.exact_evidence.is_none()
    }));
    assert!(!blockers.iter().any(|blocker| {
        matches!(
            blocker.field,
            "ID-9.read_flag"
                | "RawCommonHeader.entry_number"
                | "RawCommonHeader.reference[2]"
                | "RawCommonHeader.charge_raw"
                | "CP unused coefficient/power slots"
        )
    }));


    assert!(
        !blockers
            .iter()
            .any(|blocker| blocker.field.contains("phase_id_raw_neg"))
    );
    assert!(
        blockers
            .iter()
            .any(|blocker| blocker.class == FdbBlockerClass::Verification)
    );
    assert_eq!(blockers, built.native_blockers());

    let mut nonzero_added = pair("PHAS", "source", 0).to_vec();
    let FdbFunctionPlan::Added(added) = &mut nonzero_added[1] else {
        panic!()
    };
    added.contribution = FdbAddedContribution::Thermodynamic {
        phase_enthalpy: -100.0,
        phase_entropy: 10.0,
        ranges: vec![range()],
    };
    let added_blockers = plan(nonzero_added).unwrap().native_blockers();
    assert!(
        !added_blockers
            .iter()
            .any(|blocker| blocker.field.contains("CP.kind selection"))
    );

    let mut wide_charge = group(pair("PHAS", "source", 0).to_vec());
    wide_charge.charge = FdbChargeState::new(200);
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![wide_charge]),
        Err(FdbBuildError::InvalidField {
            field: "charge",
            ..
        })
    ));
}

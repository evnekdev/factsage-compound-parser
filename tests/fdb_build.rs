use factsage_compound_parser::fdb_build::{
    FdbAddedContribution, FdbAddedFunctionPlan, FdbAuxiliaryIntent, FdbBuildError, FdbBuildPlan,
    FdbCpTerm, FdbDatabaseMetadata, FdbElementAmount, FdbFormulaGroupPlan, FdbFunctionIdentity,
    FdbFunctionPlan, FdbFunctionRole, FdbOrdinaryFunctionPlan, FdbThermoRangePlan,
};
use factsage_compound_parser::{EnergyUnit, PressureUnit};

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
            base_target_name: base.target_name,
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
                amount: 1.0,
            },
            FdbElementAmount {
                symbol: "S".into(),
                amount: 1.0,
            },
        ],
        energy_unit: EnergyUnit::Joules,
        pressure_unit: PressureUnit::Bars,
        functions,
    }
}

fn plan(functions: Vec<FdbFunctionPlan>) -> Result<FdbBuildPlan, FdbBuildError> {
    FdbBuildPlan::new(metadata(), vec![group(functions)])
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
    added.base_target_name = "PHAS_0001".into();
    assert!(matches!(
        plan(crossed),
        Err(FdbBuildError::InvalidPairing { .. })
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
        element.amount *= 2.0;
    }
    assert!(matches!(
        FdbBuildPlan::new(metadata(), vec![first.groups()[0].clone(), proportional]),
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
fn semantic_validity_is_separate_from_native_materialization_readiness() {
    let built = plan(pair("PHAS", "source", 0).to_vec()).unwrap();
    assert!(built.validate().is_ok());
    assert!(
        built
            .native_blockers()
            .iter()
            .any(|blocker| blocker.field.contains("zero-base/A"))
    );
}

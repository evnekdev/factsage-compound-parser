//! Build a synthetic translated base with a physically omitted zero A.
//!
//! Usage: cargo run --example fdb_legacy_zero_added_probe -- <empty.fdb> <one-range.fdb> <output.fdb> [--zero-cp <translated-id5-example.fdb>]
//! Keep all paths under ignored local evidence storage.

use std::env;
use std::ffi::OsStr;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use factsage_compound_parser::fdb_build::{
    FdbAddedContribution, FdbAddedFunctionPlan, FdbAuxiliaryIntent, FdbBuildPlan, FdbChargeState,
    FdbCpTerm, FdbDatabaseMetadata, FdbElementAmount, FdbFormulaGroupPlan, FdbFreshMaterialization,
    FdbFunctionIdentity, FdbFunctionPlan, FdbFunctionRole, FdbNativeTemplates,
    FdbOrdinaryFunctionPlan, FdbStoichiometricAmount, FdbThermoRangePlan,
};
use factsage_compound_parser::{EnergyUnit, PhaseState, PressureUnit, RawDatabase};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 3 && (args.len() != 5 || args[3] != OsStr::new("--zero-cp")) {
        return Err("expected three local paths and optional --zero-cp native example".into());
    }
    let zero_cp = args.len() == 5;
    let mut templates = FdbNativeTemplates::from_examples(
        &RawDatabase::from_path(&args[0])?,
        &RawDatabase::from_path(&args[1])?,
    )?;
    if zero_cp {
        templates = templates.with_legacy_id5_example(&RawDatabase::from_path(&args[4])?)?;
    }
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
    let plan = FdbBuildPlan::new(
        FdbDatabaseMetadata {
            comment: String::new(),
            date_ole: templates.database_date_ole(),
        },
        vec![FdbFormulaGroupPlan {
            formula: "NiS".into(),
            elements: ["Ni", "S"]
                .into_iter()
                .map(|symbol| FdbElementAmount {
                    symbol: symbol.into(),
                    amount: FdbStoichiometricAmount {
                        numerator: 1,
                        denominator: 1,
                    },
                })
                .collect(),
            charge: FdbChargeState::new(0),
            energy_unit: EnergyUnit::Joules,
            pressure_unit: PressureUnit::Bars,
            functions: vec![
                FdbFunctionPlan::Ordinary(FdbOrdinaryFunctionPlan {
                    identity: base_identity.clone(),
                    phase_enthalpy: -10_000.0,
                    phase_entropy: 99.0,
                    ranges: vec![FdbThermoRangePlan {
                        temperature_min_k: 298.15,
                        temperature_max_k: 1250.0,
                        reference_enthalpy: -10_000.0,
                        reference_entropy: 99.0,
                        cp_terms: vec![FdbCpTerm {
                            coefficient: if zero_cp { 0.0 } else { 30.0 },
                            power: 0.0,
                        }],
                    }],
                    auxiliary: FdbAuxiliaryIntent::Inactive,
                }),
                FdbFunctionPlan::Added(FdbAddedFunctionPlan {
                    identity: added_identity,
                    base: base_identity,
                    contribution: FdbAddedContribution::ExplicitZero,
                    auxiliary: FdbAuxiliaryIntent::Inactive,
                }),
            ],
        }],
    )?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let timestamp = 25_569.0 + now.as_secs_f64() / 86_400.0;
    let raw =
        plan.materialize_legacy_zero_added(&templates, &FdbFreshMaterialization::new(timestamp))?;
    raw.write_to_path(&args[2])?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => {
            println!("FDB-LEGACY-ZERO-ADDED-PROBE: BUILT-REPARSED-INDEXED");
            ExitCode::SUCCESS
        }
        Err(_) => {
            eprintln!("FDB-LEGACY-ZERO-ADDED-PROBE: FAILED; details kept local");
            ExitCode::FAILURE
        }
    }
}

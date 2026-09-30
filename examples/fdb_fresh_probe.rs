//! Generate a small synthetic FreshModern FDB using local, controlled templates.
//!
//! Usage: cargo run --example fdb_fresh_probe -- <empty.fdb> <one-range.fdb> <output.fdb>
//! Inputs and output should stay under ignored local evidence storage. The caller
//! must validate fixture admission against its local receipt before running this.

use std::env;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use factsage_compound_parser::fdb_build::{
    FdbAuxiliaryIntent, FdbBuildPlan, FdbChargeState, FdbCpTerm, FdbDatabaseMetadata,
    FdbElementAmount, FdbFormulaGroupPlan, FdbFreshMaterialization, FdbFunctionIdentity,
    FdbFunctionPlan, FdbNativeTemplates, FdbOrdinaryFunctionPlan, FdbStoichiometricAmount,
    FdbThermoRangePlan,
};
use factsage_compound_parser::{EnergyUnit, PhaseState, PressureUnit, RawDatabase};

fn main() -> ExitCode {
    match run() {
        Ok(()) => {
            println!("FDB-FRESH-PROBE: BUILT-REPARSED-INDEXED");
            ExitCode::SUCCESS
        }
        Err(_) => {
            eprintln!("FDB-FRESH-PROBE: FAILED; details kept local");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let (Some(empty), Some(one_range), Some(output), None) =
        (args.next(), args.next(), args.next(), args.next())
    else {
        return Err("expected three local paths".into());
    };
    let empty = RawDatabase::from_path(empty)?;
    let one_range = RawDatabase::from_path(one_range)?;
    let templates = FdbNativeTemplates::from_examples(&empty, &one_range)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let timestamp = 25_569.0 + now.as_secs_f64() / 86_400.0;
    let plan = FdbBuildPlan::new_fresh_modern(
        FdbDatabaseMetadata {
            comment: "Synthetic writer acceptance probe".into(),
            date_ole: timestamp,
        },
        vec![FdbFormulaGroupPlan {
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
            functions: vec![FdbFunctionPlan::Ordinary(FdbOrdinaryFunctionPlan {
                identity: FdbFunctionIdentity::fresh_modern("SYNTHETIC", PhaseState::Solid),
                phase_enthalpy: -10_000.0,
                phase_entropy: 99.0,
                ranges: vec![FdbThermoRangePlan {
                    temperature_min_k: 298.15,
                    temperature_max_k: 1250.0,
                    reference_enthalpy: -10_000.0,
                    reference_entropy: 99.0,
                    cp_terms: vec![FdbCpTerm {
                        coefficient: 30.0,
                        power: 0.0,
                    }],
                }],
                auxiliary: FdbAuxiliaryIntent::Inactive,
            })],
        }],
    )?;
    let built = plan.materialize_fresh(&templates, &FdbFreshMaterialization::new(timestamp))?;
    built.write_to_path(output)?;
    Ok(())
}

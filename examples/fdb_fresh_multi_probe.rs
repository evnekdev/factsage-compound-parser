//! Build a synthetic two-function neutral and one-function charged FDB.
//!
//! Usage: cargo run --example fdb_fresh_multi_probe -- <empty.fdb> <one-range.fdb> <output.fdb>
//! Keep all paths under ignored local evidence storage.

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

fn function(name: &str, cp: f64) -> FdbFunctionPlan {
    FdbFunctionPlan::Ordinary(FdbOrdinaryFunctionPlan {
        identity: FdbFunctionIdentity::fresh_modern(name, PhaseState::Solid),
        phase_enthalpy: -10_000.0,
        phase_entropy: 99.0,
        ranges: vec![FdbThermoRangePlan {
            temperature_min_k: 298.15,
            temperature_max_k: 1250.0,
            reference_enthalpy: -10_000.0,
            reference_entropy: 99.0,
            cp_terms: vec![FdbCpTerm {
                coefficient: cp,
                power: 0.0,
            }],
        }],
        auxiliary: FdbAuxiliaryIntent::Inactive,
    })
}

fn group(formula: &str, charge: i32, functions: Vec<FdbFunctionPlan>) -> FdbFormulaGroupPlan {
    FdbFormulaGroupPlan {
        formula: formula.into(),
        elements: ["Ca", "O"]
            .into_iter()
            .map(|symbol| FdbElementAmount {
                symbol: symbol.into(),
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
            comment: String::new(),
            date_ole: templates.database_date_ole(),
        },
        vec![
            group(
                "CaO",
                0,
                vec![function("FIRST", 100.0), function("SECOND", 30.0)],
            ),
            group("CaO[+]", 1, vec![function("CHARGED", 50.0)]),
        ],
    )?;
    let built = plan.materialize_fresh(&templates, &FdbFreshMaterialization::new(timestamp))?;
    built.write_to_path(output)?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => {
            println!("FDB-FRESH-MULTI-PROBE: BUILT-REPARSED-INDEXED");
            ExitCode::SUCCESS
        }
        Err(_) => {
            eprintln!("FDB-FRESH-MULTI-PROBE: FAILED; details kept local");
            ExitCode::FAILURE
        }
    }
}

//! Categorical, local-only falsification of bounded FDB field rules.
//!
//! Usage: cargo run --example fdb_native_rule_audit -- <ignored-report> <receipt-admitted FDB>...
//! The caller must apply its local receipt gate before passing any fixture.
//! The detailed result stays at the caller-selected ignored path; stdout is
//! deliberately categorical and never includes fixture identity or values.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::process::ExitCode;
use std::str::FromStr;

use chemformula::Formula;
use factsage_compound_parser::fdb_build::FdbChargeState;
use factsage_compound_parser::{DomainIndex, RawChunk, RawCommonHeader, RawDatabase};

#[derive(Default)]
struct Results {
    files: usize,
    groups: usize,
    headers_checked: usize,
    headers_matching: usize,
    entries_checked: usize,
    entries_matching: usize,
    phase_ids_checked: usize,
    phase_ids_matching: usize,
    cp_links_checked: usize,
    cp_links_matching: usize,
    formula_eligible: usize,
    formula_matching: usize,
    formula_outside_profile: usize,
    formula_unparsable: usize,
    formula_charge_mismatch: usize,
    formula_nonordinary: usize,
    contradictions: usize,
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => {
            println!("FDB-NATIVE-RULE-AUDIT: PASS; DETAIL-LOCAL-ONLY");
            ExitCode::SUCCESS
        }
        Ok(false) => {
            println!("FDB-NATIVE-RULE-AUDIT: CONTRADICTION; DETAIL-LOCAL-ONLY");
            ExitCode::FAILURE
        }
        Err(_) => {
            eprintln!("FDB-NATIVE-RULE-AUDIT: ERROR; DETAIL-LOCAL-ONLY");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<bool, Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let Some(report) = args.next() else {
        return Err("report path required".into());
    };
    let files = args.collect::<Vec<_>>();
    if files.is_empty() {
        return Err("at least one admitted FDB path required".into());
    }
    let mut results = Results::default();
    for file in files {
        let raw = RawDatabase::from_path(file)?;
        DomainIndex::build(&raw)?;
        results.files += 1;
        audit_file(&raw, &mut results);
    }
    let mut detail = String::new();
    for (label, value) in [
        ("files", results.files),
        ("groups", results.groups),
        ("headers_checked", results.headers_checked),
        ("headers_matching", results.headers_matching),
        ("entries_checked", results.entries_checked),
        ("entries_matching", results.entries_matching),
        ("phase_ids_checked", results.phase_ids_checked),
        ("phase_ids_matching", results.phase_ids_matching),
        ("cp_links_checked", results.cp_links_checked),
        ("cp_links_matching", results.cp_links_matching),
        ("formula_eligible", results.formula_eligible),
        ("formula_matching", results.formula_matching),
        ("formula_outside_profile", results.formula_outside_profile),
        ("formula_unparsable", results.formula_unparsable),
        ("formula_charge_mismatch", results.formula_charge_mismatch),
        ("formula_nonordinary", results.formula_nonordinary),
        ("contradictions", results.contradictions),
    ] {
        writeln!(detail, "{label}={value}")?;
    }
    fs::write(report, detail)?;
    Ok(results.groups > 0 && results.contradictions == 0)
}

fn audit_file(raw: &RawDatabase, results: &mut Results) {
    let chunks = raw.chunks();
    let mut position = 1;
    while position < chunks.len() {
        let RawChunk::Compound(compound) = &chunks[position] else {
            results.contradictions += 1;
            return;
        };
        let start = position;
        position += 1;
        while position < chunks.len() && !matches!(chunks[position], RawChunk::Compound(_)) {
            position += 1;
        }
        let group = &chunks[start..position];
        results.groups += 1;
        audit_group(group, compound, results);
    }
}

fn audit_group(
    group: &[RawChunk],
    compound: &factsage_compound_parser::RawCompoundChunk,
    results: &mut Results,
) {
    let mut phase_ids = Vec::new();
    let mut phase_count = 0;
    let mut ranges_started = false;
    let start_entry = compound.header.entry_number as usize;
    for (index, record) in group.iter().enumerate() {
        let header = match record {
            RawChunk::Compound(item) => &item.header,
            RawChunk::PhaseOrdinary(item) => &item.header,
            RawChunk::HeatCapacity { chunk, .. } => &chunk.header,
            _ => continue,
        };
        results.headers_checked += 1;
        if same_composition_and_charge(&compound.header, header) {
            results.headers_matching += 1;
        } else {
            results.contradictions += 1;
        }
        results.entries_checked += 1;
        if start_entry + index <= u8::MAX as usize
            && header.entry_number as usize == start_entry + index
        {
            results.entries_matching += 1;
        } else {
            results.contradictions += 1;
        }
        match record {
            RawChunk::PhaseOrdinary(phase) => {
                phase_count += 1;
                results.phase_ids_checked += 1;
                if !ranges_started
                    && phase.phase_id_raw == 100 + phase_count
                    && phase.phase_id_raw_neg == -phase.phase_id_raw
                {
                    results.phase_ids_matching += 1;
                } else {
                    results.contradictions += 1;
                }
                phase_ids.push(phase.phase_id_raw);
            }
            RawChunk::HeatCapacity { chunk, .. } => {
                ranges_started = true;
                results.cp_links_checked += 1;
                if phase_ids.contains(&chunk.phase_id_raw) {
                    results.cp_links_matching += 1;
                } else {
                    results.contradictions += 1;
                }
            }
            _ => {}
        }
    }
    match ordinary_formula_matches(compound) {
        FormulaCheck::Match => {
            results.formula_eligible += 1;
            results.formula_matching += 1;
        }
        FormulaCheck::Contradiction => {
            results.formula_eligible += 1;
            results.contradictions += 1;
        }
        FormulaCheck::Unparsable => {
            results.formula_outside_profile += 1;
            results.formula_unparsable += 1;
        }
        FormulaCheck::ChargeMismatch => {
            results.formula_outside_profile += 1;
            results.formula_charge_mismatch += 1;
        }
        FormulaCheck::Nonordinary => {
            results.formula_outside_profile += 1;
            results.formula_nonordinary += 1;
        }
    }
}

fn same_composition_and_charge(left: &RawCommonHeader, right: &RawCommonHeader) -> bool {
    left.element_ids == right.element_ids
        && left.element_coefficients == right.element_coefficients
        && left.charge_raw == right.charge_raw
}

enum FormulaCheck {
    Match,
    Contradiction,
    Unparsable,
    ChargeMismatch,
    Nonordinary,
}

/// Out-of-profile forms are recorded separately rather than guessed or hidden.
fn ordinary_formula_matches(compound: &factsage_compound_parser::RawCompoundChunk) -> FormulaCheck {
    let Ok(formula) = Formula::from_str(&compound.formula_name_lossy()) else {
        return FormulaCheck::Unparsable;
    };
    let Some(charge) = FdbChargeState::from_fdb_raw_byte(compound.header.charge_raw as u8) else {
        return FormulaCheck::ChargeMismatch;
    };
    if formula.charge != f64::from(charge.value()) {
        return FormulaCheck::ChargeMismatch;
    }
    if formula.pairs.len() > 7 {
        return FormulaCheck::Nonordinary;
    }
    let mut ids = [0; 7];
    let mut coefficients = [0; 7];
    for (index, (element, amount)) in formula.pairs.iter().enumerate() {
        let id = element.index();
        if id == 0
            || id > u8::MAX as usize
            || !amount.is_finite()
            || *amount < 1.0
            || *amount > u8::MAX as f64
            || amount.fract() != 0.0
        {
            return FormulaCheck::Nonordinary;
        }
        ids[index] = id as u8;
        coefficients[index] = *amount as u8;
    }
    if compound.header.element_ids == ids
        && compound.header.element_coefficients == coefficients
        && compound.real_stoichiometric_coefficients == coefficients.map(f64::from)
    {
        FormulaCheck::Match
    } else {
        FormulaCheck::Contradiction
    }
}

#[cfg(test)]
mod tests {
    use super::{FormulaCheck, ordinary_formula_matches};
    use factsage_compound_parser::{RawChunk, RawDatabase};

    #[test]
    fn audit_detects_wrong_native_element_order() {
        let mut header = [0_u8; 256];
        header[0] = 9;
        header[2..6].copy_from_slice(b"CMPD");
        let mut compound = [0_u8; 256];
        compound[0] = 1;
        compound[1..3].copy_from_slice(&[1, 8]);
        compound[9..11].copy_from_slice(&[2, 1]);
        compound[16] = 50;
        compound[112..152].fill(b' ');
        compound[112..115].copy_from_slice(b"H2O");
        compound[176..184].copy_from_slice(&2.0_f64.to_le_bytes());
        compound[184..192].copy_from_slice(&1.0_f64.to_le_bytes());
        let raw = RawDatabase::from_bytes(&[header, compound].concat()).unwrap();
        let RawChunk::Compound(correct) = &raw.chunks()[1] else {
            panic!()
        };
        assert!(matches!(
            ordinary_formula_matches(correct),
            FormulaCheck::Match
        ));
        let mut reversed = correct.clone();
        reversed.header.element_ids.swap(0, 1);
        assert!(matches!(
            ordinary_formula_matches(&reversed),
            FormulaCheck::Contradiction
        ));
    }
}

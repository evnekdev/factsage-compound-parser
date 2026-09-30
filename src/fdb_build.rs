//! Rigorous, ordered FDB construction intent. This module does not create native records.
//!
//! A [`crate::fdb_build::FdbBuildPlan`] is sealed by validation. Target names and provenance are
//! structural identities; thermodynamic equality never participates in grouping
//! or pairing. [`crate::fdb_build::FdbBuildPlan::native_blockers`] reports remaining construction
//! evidence separately from semantic validity.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::thermo::{EnergyUnit, OleAutomationDate, PressureUnit};

/// Metadata actually represented by the FDB-compatible database header.
#[derive(Debug, Clone, PartialEq)]
pub struct FdbDatabaseMetadata {
    /// Caller-selected header comment, limited to the native 80-byte ASCII field.
    pub comment: String,
    /// Caller-supplied OLE Automation date; no current-clock default is inferred.
    pub date_ole: f64,
}

/// One element and its nonzero amount in a formula group.
#[derive(Debug, Clone, PartialEq)]
pub struct FdbElementAmount {
    /// Chemical element symbol; the provider does not yet map it to native IDs.
    pub symbol: String,
    /// Finite positive stoichiometric coefficient.
    pub amount: f64,
}

/// A database-global stoichiometry group containing ordered, distinct functions.
#[derive(Debug, Clone, PartialEq)]
pub struct FdbFormulaGroupPlan {
    /// Target formula label; its native composition encoding remains a C3 blocker.
    pub formula: String,
    /// Complete composition, in caller order; comparison canonicalizes symbol order.
    pub elements: Vec<FdbElementAmount>,
    /// Native compound-level energy convention.
    pub energy_unit: EnergyUnit,
    /// Native compound-level pressure-unit convention.
    pub pressure_unit: PressureUnit,
    /// Independent function objects in deterministic target order.
    pub functions: Vec<FdbFunctionPlan>,
}

/// Structural base versus added-companion role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdbFunctionRole {
    /// Ordinary/base Function object.
    Base,
    /// One-to-one additional Function object.
    Added,
}

/// Caller-owned source identity and deterministic target name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FdbFunctionIdentity {
    /// Exact FactSage phase ID from the Legacy FILE header, not its display name.
    pub source_phase_id: String,
    /// Stable source solution/provenance token; distinct source solutions need distinct tokens.
    pub source_token: String,
    /// Zero-based source G-entry encounter index.
    pub source_g_index: usize,
    /// Exact target name, `<PHASEID>_<NNNN>` with optional `A` suffix.
    pub target_name: String,
    /// Base or added role.
    pub role: FdbFunctionRole,
}

/// One coefficient and power in `Cp(T) = Σ coefficient × T^power`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FdbCpTerm {
    /// Stored coefficient in the formula group's energy convention.
    pub coefficient: f64,
    /// Stored exponent, without basis transformation.
    pub power: f64,
}

/// One ordered ordinary FDB H/S/Cp interval.
#[derive(Debug, Clone, PartialEq)]
pub struct FdbThermoRangePlan {
    /// Strictly positive lower bound in kelvin.
    pub temperature_min_k: f64,
    /// Strictly greater upper bound in kelvin.
    pub temperature_max_k: f64,
    /// Range-specific H constant at 298.15 K, in the group energy convention.
    pub reference_enthalpy: f64,
    /// Range-specific S constant at 298.15 K, in the group energy convention.
    pub reference_entropy: f64,
    /// Ordered Cp terms; no caller-supplied native slot padding is required.
    pub cp_terms: Vec<FdbCpTerm>,
}

/// Declared extra physics. Active fields are blocked until their native semantics are proved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FdbAuxiliaryIntent {
    /// No magnetic, pressure-volume, transition, or ID-11 contribution requested.
    Inactive,
    /// Source requires a contribution outside the admitted pure H/S/Cp profile.
    Active {
        /// Field or capability, such as `magnetic_temperature` or `ID-11`.
        field: String,
    },
}

/// An ordinary/base function with separately retained phase and CP H/S anchors.
#[derive(Debug, Clone, PartialEq)]
pub struct FdbOrdinaryFunctionPlan {
    /// Structural identity, independent of numerical values.
    pub identity: FdbFunctionIdentity,
    /// ID-7 phase H field; its relation to CP anchors is not assumed.
    pub phase_enthalpy: f64,
    /// ID-7 phase S field; its relation to CP anchors is not assumed.
    pub phase_entropy: f64,
    /// Ordered ID-2-style thermodynamic intervals.
    pub ranges: Vec<FdbThermoRangePlan>,
    /// Explicit disposition of auxiliary physics.
    pub auxiliary: FdbAuxiliaryIntent,
}

/// Rigorous A intent, including a physically zero object with retained identity.
#[derive(Debug, Clone, PartialEq)]
pub enum FdbAddedContribution {
    /// The source A section is zero, but the A identity still exists.
    ExplicitZero,
    /// Nonzero contribution retained as ordered H/S/Cp ranges for later ID-5 mapping.
    Thermodynamic {
        /// A contribution intervals, with no assumed native default bounds.
        ranges: Vec<FdbThermoRangePlan>,
    },
}

/// An added Function explicitly paired to one named base.
#[derive(Debug, Clone, PartialEq)]
pub struct FdbAddedFunctionPlan {
    /// Distinct A structural identity.
    pub identity: FdbFunctionIdentity,
    /// Exact target name of its owning base.
    pub base_target_name: String,
    /// Zero or nonzero A intent.
    pub contribution: FdbAddedContribution,
    /// Explicit disposition of auxiliary physics.
    pub auxiliary: FdbAuxiliaryIntent,
}

/// Function variant; source order is the vector order in its formula group.
#[derive(Debug, Clone, PartialEq)]
pub enum FdbFunctionPlan {
    /// Ordinary/base object.
    Ordinary(FdbOrdinaryFunctionPlan),
    /// Rigorous base identity for a G entry with no ordinary intervals.
    ExplicitZeroOrdinary(FdbFunctionIdentity),
    /// Additional companion object.
    Added(FdbAddedFunctionPlan),
}

impl FdbFunctionPlan {
    /// Returns the structural identity without inspecting values.
    pub fn identity(&self) -> &FdbFunctionIdentity {
        match self {
            Self::Ordinary(function) => &function.identity,
            Self::ExplicitZeroOrdinary(identity) => identity,
            Self::Added(function) => &function.identity,
        }
    }
}

/// Construction rejection with object, field, and actionable reason.
#[derive(Debug, Clone, PartialEq)]
pub enum FdbBuildError {
    /// A required text or metadata value is invalid.
    InvalidField {
        /// Object bearing the field.
        object: String,
        /// Native or semantic field name.
        field: &'static str,
        /// Corrective explanation.
        reason: String,
    },
    /// Two objects claim one structural name or source role.
    DuplicateIdentity {
        /// Repeated object identity.
        object: String,
        /// First claim or conflicting source role.
        previous: String,
    },
    /// A formula group is missing or repeats an existing stoichiometry.
    AmbiguousGroup {
        /// Formula group or database scope.
        group: String,
        /// Corrective explanation.
        reason: String,
    },
    /// A companion does not identify exactly its corresponding base.
    InvalidPairing {
        /// Added target identity.
        added: String,
        /// Claimed or missing base identity.
        base: String,
        /// Corrective explanation.
        reason: String,
    },
    /// H/S/Cp data violate the provider's ordinary-range constraints.
    InvalidThermodynamics {
        /// Function identity or database scope.
        object: String,
        /// Failing field or indexed range path.
        field: String,
        /// Corrective explanation.
        reason: String,
    },
    /// Active auxiliary physics has no established fresh-construction semantics.
    UnsupportedAuxiliary {
        /// Function identity.
        object: String,
        /// Active auxiliary field or capability.
        field: String,
        /// Missing semantic rule.
        reason: &'static str,
    },
}

impl fmt::Display for FdbBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for FdbBuildError {}

/// A missing native rule or implementation capability, separate from plan validity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FdbBuildBlocker {
    /// Object or profile affected.
    pub object: String,
    /// Exact native field or capability.
    pub field: &'static str,
    /// Specific missing evidence or implementation.
    pub reason: &'static str,
    /// Whether a paired fixture or domain expert can provide the missing fact.
    pub user_evidence_can_unblock: bool,
}

/// Validated, immutable rigorous-semantic FDB intent.
#[derive(Debug, Clone, PartialEq)]
pub struct FdbBuildPlan {
    metadata: FdbDatabaseMetadata,
    groups: Vec<FdbFormulaGroupPlan>,
}

impl FdbBuildPlan {
    /// Seals caller input only after complete semantic validation.
    pub fn new(
        metadata: FdbDatabaseMetadata,
        groups: Vec<FdbFormulaGroupPlan>,
    ) -> Result<Self, FdbBuildError> {
        let plan = Self { metadata, groups };
        plan.validate()?;
        Ok(plan)
    }

    /// Returns validated database-level input.
    pub fn metadata(&self) -> &FdbDatabaseMetadata {
        &self.metadata
    }

    /// Returns groups and functions in supplied order, without deduplication.
    pub fn groups(&self) -> &[FdbFormulaGroupPlan] {
        &self.groups
    }

    /// Rechecks the pure semantic plan without creating native bytes or chunks.
    pub fn validate(&self) -> Result<(), FdbBuildError> {
        check_text("database", "comment", &self.metadata.comment, 80, true)?;
        OleAutomationDate::from_raw(self.metadata.date_ole).map_err(|error| {
            FdbBuildError::InvalidField {
                object: "database".into(),
                field: "date_ole",
                reason: error.to_string(),
            }
        })?;
        if self.groups.is_empty() {
            return Err(FdbBuildError::AmbiguousGroup {
                group: "database".into(),
                reason: "at least one formula group is required".into(),
            });
        }
        let mut group_keys = BTreeMap::new();
        let mut formula_names = BTreeSet::new();
        let mut names = BTreeMap::<String, String>::new();
        let mut source_roles = BTreeSet::new();
        let mut bases = BTreeMap::<String, (String, usize, String)>::new();
        let mut additions = Vec::new();
        for group in &self.groups {
            check_text(&group.formula, "formula", &group.formula, 40, false)?;
            if !formula_names.insert(group.formula.clone()) {
                return Err(FdbBuildError::AmbiguousGroup {
                    group: group.formula.clone(),
                    reason: "formula label already belongs to another group".into(),
                });
            }
            let key = composition_key(group)?;
            if let Some(previous) = group_keys.insert(key.clone(), group.formula.clone()) {
                return Err(FdbBuildError::AmbiguousGroup {
                    group: group.formula.clone(),
                    reason: format!("stoichiometry already belongs to formula group {previous}"),
                });
            }
            if matches!(group.energy_unit, EnergyUnit::Unknown(_)) {
                return Err(FdbBuildError::InvalidField {
                    object: group.formula.clone(),
                    field: "energy_unit",
                    reason: "unknown native energy code".into(),
                });
            }
            if matches!(group.pressure_unit, PressureUnit::Unknown(_)) {
                return Err(FdbBuildError::InvalidField {
                    object: group.formula.clone(),
                    field: "pressure_unit",
                    reason: "unknown native pressure code".into(),
                });
            }
            if group.functions.is_empty() {
                return Err(FdbBuildError::AmbiguousGroup {
                    group: group.formula.clone(),
                    reason: "formula group has no functions".into(),
                });
            }
            for function in &group.functions {
                let id = function.identity();
                validate_identity(id)?;
                if let Some(previous) = names.insert(id.target_name.clone(), group.formula.clone())
                {
                    return Err(FdbBuildError::DuplicateIdentity {
                        object: id.target_name.clone(),
                        previous,
                    });
                }
                if !source_roles.insert((
                    id.source_token.clone(),
                    id.source_phase_id.clone(),
                    id.source_g_index,
                    id.role as u8,
                )) {
                    return Err(FdbBuildError::DuplicateIdentity {
                        object: id.target_name.clone(),
                        previous: "source G-entry role already present".into(),
                    });
                }
                match function {
                    FdbFunctionPlan::Ordinary(base) => {
                        if id.role != FdbFunctionRole::Base {
                            return Err(FdbBuildError::InvalidField {
                                object: id.target_name.clone(),
                                field: "role",
                                reason: "ordinary variant requires Base role".into(),
                            });
                        }
                        validate_aux(&id.target_name, &base.auxiliary)?;
                        check_finite(&id.target_name, "phase_enthalpy", base.phase_enthalpy)?;
                        check_finite(&id.target_name, "phase_entropy", base.phase_entropy)?;
                        validate_ranges(&id.target_name, &base.ranges)?;
                        bases.insert(
                            id.target_name.clone(),
                            (id.source_token.clone(), id.source_g_index, key.clone()),
                        );
                    }
                    FdbFunctionPlan::ExplicitZeroOrdinary(_) => {
                        if id.role != FdbFunctionRole::Base {
                            return Err(FdbBuildError::InvalidField {
                                object: id.target_name.clone(),
                                field: "role",
                                reason: "zero ordinary variant requires Base role".into(),
                            });
                        }
                        bases.insert(
                            id.target_name.clone(),
                            (id.source_token.clone(), id.source_g_index, key.clone()),
                        );
                    }
                    FdbFunctionPlan::Added(added) => {
                        if id.role != FdbFunctionRole::Added {
                            return Err(FdbBuildError::InvalidField {
                                object: id.target_name.clone(),
                                field: "role",
                                reason: "added variant requires Added role".into(),
                            });
                        }
                        validate_aux(&id.target_name, &added.auxiliary)?;
                        if let FdbAddedContribution::Thermodynamic { ranges } = &added.contribution
                        {
                            validate_ranges(&id.target_name, ranges)?;
                        }
                        additions.push((id, &added.base_target_name, key.clone()));
                    }
                }
            }
        }
        let mut paired = BTreeSet::new();
        for (id, base_name, group_key) in additions {
            let Some((source, index, base_group)) = bases.get(base_name) else {
                return Err(FdbBuildError::InvalidPairing {
                    added: id.target_name.clone(),
                    base: base_name.clone(),
                    reason: "base identity is missing".into(),
                });
            };
            if base_group != &group_key
                || source != &id.source_token
                || *index != id.source_g_index
                || id.target_name != format!("{base_name}A")
            {
                return Err(FdbBuildError::InvalidPairing {
                    added: id.target_name.clone(),
                    base: base_name.clone(),
                    reason:
                        "A must share source G entry and formula group and use the base name plus A"
                            .into(),
                });
            }
            if !paired.insert(base_name) {
                return Err(FdbBuildError::InvalidPairing {
                    added: id.target_name.clone(),
                    base: base_name.clone(),
                    reason: "base already owns another A".into(),
                });
            }
        }
        for base_name in bases.keys() {
            if !paired.contains(base_name) {
                return Err(FdbBuildError::InvalidPairing {
                    added: format!("{base_name}A"),
                    base: base_name.clone(),
                    reason: "rigorous profile requires one explicit A companion per base".into(),
                });
            }
        }
        Ok(())
    }

    /// Lists exact blockers for provider-owned native materialization (FDB-C3).
    /// Semantic validity never implies these fields can already be encoded.
    pub fn native_blockers(&self) -> Vec<FdbBuildBlocker> {
        let mut blockers = vec![FdbBuildBlocker {
            object: "database header".into(),
            field: "header padding/unknown bytes and native date policy",
            reason: "fresh FDB defaults have not been established by paired construction evidence",
            user_evidence_can_unblock: true,
        }];
        for group in &self.groups {
            blockers.push(FdbBuildBlocker {
                object: group.formula.clone(),
                field: "common header element IDs, charge, entry/reference/timestamp, compound name, real stoichiometry and reserved fields",
                reason: "native generation, formula-label consistency and default rules are not established for fresh groups",
                user_evidence_can_unblock: true,
            });
            for function in &group.functions {
                let object = function.identity().target_name.clone();
                blockers.push(FdbBuildBlocker {
                    object: object.clone(),
                    field: "phase IDs, negative ID, common metadata and phase padding",
                    reason: "fresh ID allocation/link and default rules are not established",
                    user_evidence_can_unblock: true,
                });
                match function {
                    FdbFunctionPlan::Ordinary(_) => blockers.push(FdbBuildBlocker {
                        object,
                        field: "CP unknown bytes/padding, unused Cp slots and ordinary phase H/S selection",
                        reason: "fresh record defaults and phase-anchor selection need paired construction evidence",
                        user_evidence_can_unblock: true,
                    }),
                    FdbFunctionPlan::ExplicitZeroOrdinary(_) => blockers.push(FdbBuildBlocker {
                        object,
                        field: "zero-base physical encoding or versioned omission policy",
                        reason: "rigorous zero base identity is established but physical representation is not",
                        user_evidence_can_unblock: true,
                    }),
                    FdbFunctionPlan::Added(_) => blockers.push(FdbBuildBlocker {
                        object,
                        field: "ID-5 bounds, powers, zero-A encoding and exceptional A entropy sign",
                        reason: "paired evidence does not establish a general source-to-native A rule or zero-object policy",
                        user_evidence_can_unblock: true,
                    }),
                }
            }
        }
        blockers
    }
}

fn check_text(
    object: &str,
    field: &'static str,
    value: &str,
    max: usize,
    allow_empty: bool,
) -> Result<(), FdbBuildError> {
    if (!allow_empty && value.is_empty())
        || value.len() > max
        || !value.bytes().all(|b| (0x20..=0x7e).contains(&b))
    {
        return Err(FdbBuildError::InvalidField {
            object: object.into(),
            field,
            reason: format!(
                "requires printable ASCII, {}1..={max} bytes",
                if allow_empty { "0 or " } else { "" }
            ),
        });
    }
    Ok(())
}

fn check_finite(object: &str, field: &'static str, value: f64) -> Result<(), FdbBuildError> {
    if !value.is_finite() {
        return Err(FdbBuildError::InvalidThermodynamics {
            object: object.into(),
            field: field.into(),
            reason: "value must be finite".into(),
        });
    }
    Ok(())
}

fn composition_key(group: &FdbFormulaGroupPlan) -> Result<String, FdbBuildError> {
    if group.elements.is_empty() {
        return Err(FdbBuildError::AmbiguousGroup {
            group: group.formula.clone(),
            reason: "composition is empty".into(),
        });
    }
    let mut elements = BTreeMap::new();
    for element in &group.elements {
        if element.symbol.is_empty()
            || !element.symbol.bytes().all(|b| b.is_ascii_alphabetic())
            || !element.amount.is_finite()
            || element.amount <= 0.0
        {
            return Err(FdbBuildError::AmbiguousGroup {
                group: group.formula.clone(),
                reason: format!(
                    "invalid element symbol or positive amount: {}",
                    element.symbol
                ),
            });
        }
        if elements
            .insert(element.symbol.clone(), element.amount)
            .is_some()
        {
            return Err(FdbBuildError::AmbiguousGroup {
                group: group.formula.clone(),
                reason: format!("duplicate element {}", element.symbol),
            });
        }
    }
    // Stoichiometry is a ratio: Ni1S1 and Ni2S2 must occupy the same group.
    // Exact f64 ratios are used deliberately; identity never uses a tolerance.
    let scale = *elements.values().next().expect("nonempty checked above");
    let mut key = String::new();
    for (symbol, amount) in elements {
        let ratio = amount / scale;
        if !ratio.is_finite() {
            return Err(FdbBuildError::AmbiguousGroup {
                group: group.formula.clone(),
                reason: "normalized stoichiometry is non-finite".into(),
            });
        }
        key.push_str(&format!("{symbol}:{};", ratio.to_bits()));
    }
    Ok(key)
}

fn validate_identity(id: &FdbFunctionIdentity) -> Result<(), FdbBuildError> {
    check_text(
        &id.target_name,
        "source_phase_id",
        &id.source_phase_id,
        40,
        false,
    )?;
    if id.source_token.is_empty() {
        return Err(FdbBuildError::InvalidField {
            object: id.target_name.clone(),
            field: "source_token",
            reason: "source provenance token must be nonempty".into(),
        });
    }
    check_text(&id.target_name, "target_name", &id.target_name, 40, false)?;
    if id.source_g_index > 9999 {
        return Err(FdbBuildError::InvalidField {
            object: id.target_name.clone(),
            field: "source_g_index",
            reason: "four-digit naming supports indices 0000..9999".into(),
        });
    }
    let suffix = if id.role == FdbFunctionRole::Added {
        "A"
    } else {
        ""
    };
    let expected = format!("{}_{:04}{suffix}", id.source_phase_id, id.source_g_index);
    if id.target_name != expected {
        return Err(FdbBuildError::InvalidField {
            object: id.target_name.clone(),
            field: "target_name",
            reason: format!("expected {expected} from FILE phase ID and encounter index"),
        });
    }
    Ok(())
}

fn validate_aux(object: &str, auxiliary: &FdbAuxiliaryIntent) -> Result<(), FdbBuildError> {
    if let FdbAuxiliaryIntent::Active { field } = auxiliary {
        return Err(FdbBuildError::UnsupportedAuxiliary {
            object: object.into(),
            field: field.clone(),
            reason: "active auxiliary contribution lacks established fresh-FDB construction semantics",
        });
    }
    Ok(())
}

fn validate_ranges(object: &str, ranges: &[FdbThermoRangePlan]) -> Result<(), FdbBuildError> {
    if ranges.is_empty() {
        return Err(FdbBuildError::InvalidThermodynamics {
            object: object.into(),
            field: "ranges".into(),
            reason: "at least one H/S/Cp interval is required; use ExplicitZero for a zero A"
                .into(),
        });
    }
    let mut previous: Option<&FdbThermoRangePlan> = None;
    for (index, range) in ranges.iter().enumerate() {
        if range.cp_terms.len() > 8 {
            return Err(FdbBuildError::InvalidThermodynamics {
                object: object.into(),
                field: format!("ranges[{index}].cp_terms"),
                reason: "ordinary FDB range supports at most eight Cp terms".into(),
            });
        }
        for (field, value) in [
            ("temperature_min_k", range.temperature_min_k),
            ("temperature_max_k", range.temperature_max_k),
            ("reference_enthalpy", range.reference_enthalpy),
            ("reference_entropy", range.reference_entropy),
        ] {
            check_finite(object, field, value)?;
        }
        if range.temperature_min_k <= 0.0 || range.temperature_min_k >= range.temperature_max_k {
            return Err(FdbBuildError::InvalidThermodynamics {
                object: object.into(),
                field: format!("ranges[{index}].temperature"),
                reason: "requires 0 < lower < upper".into(),
            });
        }
        for (term_index, term) in range.cp_terms.iter().enumerate() {
            for (field, value) in [("coefficient", term.coefficient), ("power", term.power)] {
                if !value.is_finite() {
                    return Err(FdbBuildError::InvalidThermodynamics {
                        object: object.into(),
                        field: format!("ranges[{index}].cp_terms[{term_index}].{field}"),
                        reason: "value must be finite".into(),
                    });
                }
            }
        }
        if let Some(left) = previous {
            if left.temperature_max_k.to_bits() != range.temperature_min_k.to_bits() {
                return Err(FdbBuildError::InvalidThermodynamics { object: object.into(), field: format!("ranges[{index}].temperature_min_k"), reason: "source-order ranges must meet at exactly the same boundary without gap or overlap".into() });
            }
            for (quantity, left_anchor, right_anchor, entropy) in [
                (
                    "enthalpy",
                    left.reference_enthalpy,
                    range.reference_enthalpy,
                    false,
                ),
                (
                    "entropy",
                    left.reference_entropy,
                    range.reference_entropy,
                    true,
                ),
            ] {
                let left_value =
                    integrated_at(left, left.temperature_max_k, entropy).map_err(|reason| {
                        FdbBuildError::InvalidThermodynamics {
                            object: object.into(),
                            field: format!("ranges[{}].{quantity}", index - 1),
                            reason,
                        }
                    })? + left_anchor;
                let right_value =
                    integrated_at(range, range.temperature_min_k, entropy).map_err(|reason| {
                        FdbBuildError::InvalidThermodynamics {
                            object: object.into(),
                            field: format!("ranges[{index}].{quantity}"),
                            reason,
                        }
                    })? + right_anchor;
                let tolerance = crate::thermo::PROVIDER_RANGE_CONTINUITY_ABSOLUTE_TOLERANCE
                    + crate::thermo::PROVIDER_RANGE_CONTINUITY_RELATIVE_TOLERANCE
                        * left_value.abs().max(right_value.abs());
                if !left_value.is_finite()
                    || !right_value.is_finite()
                    || (left_value - right_value).abs() > tolerance
                {
                    return Err(FdbBuildError::InvalidThermodynamics { object: object.into(), field: format!("ranges[{index}].{quantity}"), reason: "adjacent H/S evaluations are non-finite or discontinuous at shared boundary".into() });
                }
            }
        }
        previous = Some(range);
    }
    Ok(())
}

// Uses the provider's 298.15 K integration rule. FDB-C3 will share the
// evaluator directly when raw construction and strict reparse are available.
fn integrated_at(
    range: &FdbThermoRangePlan,
    temperature: f64,
    entropy: bool,
) -> Result<f64, String> {
    let mut sum = 0.0;
    for term in &range.cp_terms {
        let p = if entropy {
            term.power
        } else {
            term.power + 1.0
        };
        let integral = crate::thermo::integrate_power(temperature, p);
        let value = term.coefficient * integral;
        if !value.is_finite() {
            return Err("Cp integral term is non-finite".into());
        }
        sum += value;
        if !sum.is_finite() {
            return Err("Cp integral sum is non-finite".into());
        }
    }
    Ok(sum)
}

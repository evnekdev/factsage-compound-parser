//! Rigorous, ordered FDB construction intent. This module does not create native records.
//!
//! A [`crate::fdb_build::FdbBuildPlan`] is sealed by validation. Target names are
//! structural identities; Legacy provenance is required only for the translation profile.
//! Thermodynamic equality never participates in grouping
//! or pairing. [`crate::fdb_build::FdbBuildPlan::native_blockers`] reports remaining construction
//! evidence separately from semantic validity.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::PhaseState;
use crate::thermo::{EnergyUnit, OleAutomationDate, PressureUnit};

mod blockers;
pub use blockers::{FdbBlockerClass, FdbBuildBlocker};

/// Metadata actually represented by the FDB-compatible database header.
#[derive(Debug, Clone, PartialEq)]
pub struct FdbDatabaseMetadata {
    /// Caller-selected header comment, limited to the native 80-byte ASCII field.
    pub comment: String,
    /// Caller-supplied OLE Automation date; no current-clock default is inferred.
    pub date_ole: f64,
}

/// Exact positive source stoichiometric amount; no floating rounding is implied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FdbStoichiometricAmount {
    /// Positive numerator.
    pub numerator: u64,
    /// Positive denominator.
    pub denominator: u64,
}

/// Semantic electrical charge, independent of formula-label spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FdbChargeState(i32);

impl FdbChargeState {
    /// Retains the exact signed source charge independently of native encoding.
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    /// Returns the exact signed semantic charge.
    pub const fn value(self) -> i32 {
        self.0
    }

    /// Encodes the bounded fresh-modern charge profile as one native byte.
    /// The admitted range is -50..=50; its output also fits the parser's `i8` slot.
    pub const fn fresh_modern_raw_byte(self) -> Option<u8> {
        if self.0 < -50 || self.0 > 50 {
            None
        } else {
            Some((self.0 + 50) as u8)
        }
    }

    /// Decodes one byte under the bounded fresh-modern charge rule.
    pub const fn from_fresh_modern_raw_byte(raw: u8) -> Option<Self> {
        if raw > 100 {
            None
        } else {
            Some(Self(raw as i32 - 50))
        }
    }
}

/// One element and its exact, nonzero amount in a formula group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FdbElementAmount {
    /// Canonically cased one- or two-letter element symbol; native ID mapping is pending.
    pub symbol: String,
    /// Positive exact stoichiometric amount.
    pub amount: FdbStoichiometricAmount,
}

/// Canonical semantic group key: ordered element ratios and explicit charge.
///
/// Ratio normalization uses exact integer arithmetic. It does not assert how
/// FactSage should encode source coefficients in native ID-1 fields.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FdbFormulaGroupIdentity {
    elements: Vec<(String, u128, u128)>,
    charge: FdbChargeState,
}

impl FdbFormulaGroupIdentity {
    /// Canonical symbol and reduced numerator/denominator ratios.
    pub fn element_ratios(&self) -> &[(String, u128, u128)] {
        &self.elements
    }

    /// Exact semantic charge.
    pub const fn charge(&self) -> FdbChargeState {
        self.charge
    }
}

/// A database-global composition-plus-charge group containing distinct functions.
#[derive(Debug, Clone, PartialEq)]
pub struct FdbFormulaGroupPlan {
    /// Target formula label; its native composition encoding remains a C3 blocker.
    pub formula: String,
    /// Complete composition, in caller order; comparison canonicalizes symbol order.
    pub elements: Vec<FdbElementAmount>,
    /// Explicit signed semantic charge; neutral is zero, not an inferred default.
    pub charge: FdbChargeState,
    /// Native compound-level energy convention.
    pub energy_unit: EnergyUnit,
    /// Native compound-level pressure-unit convention.
    pub pressure_unit: PressureUnit,
    /// Independent function objects in deterministic target order.
    pub functions: Vec<FdbFunctionPlan>,
}

impl FdbFormulaGroupPlan {
    /// Computes exact, charge-aware semantic identity independently of the label.
    pub fn semantic_identity(&self) -> Result<FdbFormulaGroupIdentity, FdbBuildError> {
        composition_key(self)
    }
}

/// Structural base versus added-companion role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FdbFunctionRole {
    /// Ordinary/base Function object.
    Base,
    /// One-to-one additional Function object.
    Added,
}

/// Naming and pairing contract for the construction request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdbConstructionProfile {
    /// Legacy G entries retain FILE phase ID, encounter index, and base/A pairing.
    LegacyTranslation,
    /// Directly authored modern functions retain caller-supplied names.
    FreshModern,
}

/// Target function identity with profile-specific source provenance.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FdbFunctionIdentity {
    /// Exact Legacy FILE phase ID; empty for fresh-modern construction.
    pub source_phase_id: String,
    /// Stable Legacy source token; empty for fresh-modern construction.
    pub source_token: String,
    /// Zero-based Legacy G-entry index; zero for fresh-modern construction.
    pub source_g_index: usize,
    /// Explicit semantic target state; the provider owns the native ID encoding.
    pub target_state: PhaseState,
    /// Exact native function name. Legacy translation constrains its spelling.
    pub target_name: String,
    /// Base or added role.
    pub role: FdbFunctionRole,
}

impl FdbFunctionIdentity {
    /// Names a directly authored modern function without inventing Legacy provenance.
    pub fn fresh_modern(target_name: impl Into<String>, target_state: PhaseState) -> Self {
        Self {
            source_phase_id: String::new(),
            source_token: String::new(),
            source_g_index: 0,
            target_state,
            target_name: target_name.into(),
            role: FdbFunctionRole::Base,
        }
    }
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
    /// Ordered thermodynamic intervals; native CP kind is selected by the provider.
    pub ranges: Vec<FdbThermoRangePlan>,
    /// Explicit disposition of auxiliary physics.
    pub auxiliary: FdbAuxiliaryIntent,
}

/// Rigorous A intent, including a physically zero object with retained identity.
#[derive(Debug, Clone, PartialEq)]
pub enum FdbAddedContribution {
    /// The source A section is zero, but the A identity still exists.
    ExplicitZero,
    /// Explicit H/S/Cp contribution; native CP kind needs source-model evidence.
    Thermodynamic {
        /// Explicit ID-7 phase H field in the group energy convention.
        phase_enthalpy: f64,
        /// Explicit ID-7 phase S field in the group energy convention.
        phase_entropy: f64,
        /// A contribution intervals, with no assumed native default bounds.
        ranges: Vec<FdbThermoRangePlan>,
    },
}

/// An added Function explicitly paired to one typed base identity.
#[derive(Debug, Clone, PartialEq)]
pub struct FdbAddedFunctionPlan {
    /// Distinct A structural identity.
    pub identity: FdbFunctionIdentity,
    /// Exact structural identity of its owning base.
    pub base: FdbFunctionIdentity,
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
    /// A formula group is missing or duplicates a full composition-plus-charge key.
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

/// Validated, immutable rigorous-semantic FDB intent.
#[derive(Debug, Clone, PartialEq)]
pub struct FdbBuildPlan {
    profile: FdbConstructionProfile,
    metadata: FdbDatabaseMetadata,
    groups: Vec<FdbFormulaGroupPlan>,
}

impl FdbBuildPlan {
    /// Seals caller input only after complete semantic validation.
    pub fn new(
        metadata: FdbDatabaseMetadata,
        groups: Vec<FdbFormulaGroupPlan>,
    ) -> Result<Self, FdbBuildError> {
        Self::with_profile(FdbConstructionProfile::LegacyTranslation, metadata, groups)
    }

    /// Seals direct-modern intent with caller-supplied function names and no Legacy A pairing.
    pub fn new_fresh_modern(
        metadata: FdbDatabaseMetadata,
        groups: Vec<FdbFormulaGroupPlan>,
    ) -> Result<Self, FdbBuildError> {
        Self::with_profile(FdbConstructionProfile::FreshModern, metadata, groups)
    }

    fn with_profile(
        profile: FdbConstructionProfile,
        metadata: FdbDatabaseMetadata,
        groups: Vec<FdbFormulaGroupPlan>,
    ) -> Result<Self, FdbBuildError> {
        let plan = Self {
            profile,
            metadata,
            groups,
        };
        plan.validate()?;
        Ok(plan)
    }

    /// Returns the source-specific naming and pairing contract.
    pub const fn profile(&self) -> FdbConstructionProfile {
        self.profile
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
        let mut group_keys = BTreeMap::<FdbFormulaGroupIdentity, String>::new();
        let mut formula_names = BTreeSet::new();
        let mut source_roles = BTreeSet::new();
        let mut bases = BTreeMap::<FdbFunctionIdentity, FdbFormulaGroupIdentity>::new();
        let mut additions = Vec::new();
        for group in &self.groups {
            let mut names = BTreeMap::<String, String>::new();
            check_text(&group.formula, "formula", &group.formula, 40, false)?;
            if self.profile == FdbConstructionProfile::FreshModern
                && group.charge.fresh_modern_raw_byte().is_none()
            {
                return Err(FdbBuildError::InvalidField {
                    object: group.formula.clone(),
                    field: "charge",
                    reason: "fresh-modern charge must be within -50..=50".into(),
                });
            }
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
                validate_identity(id, self.profile)?;
                if let Some(previous) = names.insert(id.target_name.clone(), group.formula.clone())
                {
                    return Err(FdbBuildError::DuplicateIdentity {
                        object: id.target_name.clone(),
                        previous,
                    });
                }
                if self.profile == FdbConstructionProfile::LegacyTranslation
                    && !source_roles.insert((
                        id.source_token.clone(),
                        id.source_phase_id.clone(),
                        id.source_g_index,
                        id.role,
                    ))
                {
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
                        bases.insert(id.clone(), key.clone());
                    }
                    FdbFunctionPlan::ExplicitZeroOrdinary(_) => {
                        if id.role != FdbFunctionRole::Base {
                            return Err(FdbBuildError::InvalidField {
                                object: id.target_name.clone(),
                                field: "role",
                                reason: "zero ordinary variant requires Base role".into(),
                            });
                        }
                        bases.insert(id.clone(), key.clone());
                    }
                    FdbFunctionPlan::Added(added) => {
                        if self.profile == FdbConstructionProfile::FreshModern {
                            return Err(FdbBuildError::InvalidField {
                                object: id.target_name.clone(),
                                field: "role",
                                reason:
                                    "fresh-modern construction has no evidenced A-companion policy"
                                        .into(),
                            });
                        }
                        if id.role != FdbFunctionRole::Added {
                            return Err(FdbBuildError::InvalidField {
                                object: id.target_name.clone(),
                                field: "role",
                                reason: "added variant requires Added role".into(),
                            });
                        }
                        validate_aux(&id.target_name, &added.auxiliary)?;
                        if let FdbAddedContribution::Thermodynamic {
                            phase_enthalpy,
                            phase_entropy,
                            ranges,
                        } = &added.contribution
                        {
                            check_finite(&id.target_name, "phase_enthalpy", *phase_enthalpy)?;
                            check_finite(&id.target_name, "phase_entropy", *phase_entropy)?;
                            validate_ranges(&id.target_name, ranges)?;
                        }
                        additions.push((id, &added.base, key.clone()));
                    }
                }
            }
        }
        let mut paired = BTreeSet::new();
        for (id, base, group_key) in additions {
            validate_identity(base, self.profile)?;
            let Some(base_group) = bases.get(base) else {
                return Err(FdbBuildError::InvalidPairing {
                    added: id.target_name.clone(),
                    base: base.target_name.clone(),
                    reason: "base identity is missing".into(),
                });
            };
            if base_group != &group_key
                || base.role != FdbFunctionRole::Base
                || base.source_token != id.source_token
                || base.source_phase_id != id.source_phase_id
                || base.source_g_index != id.source_g_index
                || base.target_state != id.target_state
                || id.target_name != format!("{}A", base.target_name)
            {
                return Err(FdbBuildError::InvalidPairing {
                    added: id.target_name.clone(),
                    base: base.target_name.clone(),
                    reason:
                        "A must share source G entry, target state and formula group and use the base name plus A"
                            .into(),
                });
            }
            if !paired.insert(base) {
                return Err(FdbBuildError::InvalidPairing {
                    added: id.target_name.clone(),
                    base: base.target_name.clone(),
                    reason: "base already owns another A".into(),
                });
            }
        }
        if self.profile == FdbConstructionProfile::LegacyTranslation {
            for base in bases.keys() {
                if !paired.contains(base) {
                    return Err(FdbBuildError::InvalidPairing {
                        added: format!("{}A", base.target_name),
                        base: base.target_name.clone(),
                        reason: "rigorous profile requires one explicit A companion per base"
                            .into(),
                    });
                }
            }
        }
        Ok(())
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

fn composition_key(group: &FdbFormulaGroupPlan) -> Result<FdbFormulaGroupIdentity, FdbBuildError> {
    if group.elements.is_empty() {
        return Err(FdbBuildError::AmbiguousGroup {
            group: group.formula.clone(),
            reason: "composition is empty".into(),
        });
    }
    if group.elements.len() > 7 {
        return Err(FdbBuildError::AmbiguousGroup {
            group: group.formula.clone(),
            reason: "native compound headers support at most seven element slots".into(),
        });
    }
    let mut elements = BTreeMap::new();
    for element in &group.elements {
        let symbol = element.symbol.as_bytes();
        if !(symbol.len() == 1 || symbol.len() == 2)
            || !symbol[0].is_ascii_uppercase()
            || (symbol.len() == 2 && !symbol[1].is_ascii_lowercase())
            || element.amount.numerator == 0
            || element.amount.denominator == 0
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
    // Stoichiometry is a ratio: Ni1S1 and Ni2S2 share a semantic key at the
    // same charge. Products of two u64 values fit u128; reduction stays exact.
    let scale = *elements.values().next().expect("nonempty checked above");
    let mut canonical = Vec::with_capacity(elements.len());
    for (symbol, amount) in elements {
        let numerator = u128::from(amount.numerator) * u128::from(scale.denominator);
        let denominator = u128::from(amount.denominator) * u128::from(scale.numerator);
        let divisor = gcd_u128(numerator, denominator);
        canonical.push((symbol, numerator / divisor, denominator / divisor));
    }
    Ok(FdbFormulaGroupIdentity {
        elements: canonical,
        charge: group.charge,
    })
}

fn gcd_u128(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

fn validate_identity(
    id: &FdbFunctionIdentity,
    profile: FdbConstructionProfile,
) -> Result<(), FdbBuildError> {
    if profile == FdbConstructionProfile::FreshModern {
        check_text(&id.target_name, "target_name", &id.target_name, 40, false)?;
        if !id.source_phase_id.is_empty() || !id.source_token.is_empty() || id.source_g_index != 0 {
            return Err(FdbBuildError::InvalidField {
                object: id.target_name.clone(),
                field: "source_provenance",
                reason: "fresh-modern identity must not claim Legacy FILE/G provenance".into(),
            });
        }
        return Ok(());
    }
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
        if field.trim().is_empty() {
            return Err(FdbBuildError::InvalidField {
                object: object.into(),
                field: "auxiliary.field",
                reason: "active auxiliary contribution needs a named field or capability".into(),
            });
        }
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
                let tolerance = crate::thermo::continuity_tolerance(left_value, right_value);
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

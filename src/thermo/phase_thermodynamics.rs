use std::fmt;

use crate::RawChunk;
use crate::domain::{CompoundView, DatabaseView, HeatCapacityRangeView, PhaseView, RawPhase};
use crate::raw::HeatCapacityKind;

use super::{
    EnergyUnit, OrdinaryFdbEffectiveGEligibility, UnitError,
    effective_g_eligibility::ordinary_effective_g_eligibility,
};

/// The reference temperature used by the established FDB ordinary-phase and
/// CP-range H/S constants.
pub const STANDARD_REFERENCE_TEMPERATURE_K: f64 = 298.15;

/// Absolute floor used by provider validation when comparing adjacent FDB H/S
/// values in the compound's native energy unit.
///
/// This is deliberately a provider-format validation tolerance, not a
/// cross-provider scientific comparison tolerance. The permitted residual is
/// this floor plus [`PROVIDER_RANGE_CONTINUITY_RELATIVE_TOLERANCE`] times the
/// larger magnitude of the two boundary values.
pub const PROVIDER_RANGE_CONTINUITY_ABSOLUTE_TOLERANCE: f64 = 1.0e-8;

/// Relative tolerance used only to validate adjacent FDB range H/S continuity.
pub const PROVIDER_RANGE_CONTINUITY_RELATIVE_TOLERANCE: f64 = 1.0e-8;

/// Thermodynamic quantity involved in provider range evaluation or validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdbThermodynamicQuantity {
    /// Heat capacity, `Cp(T)`.
    HeatCapacity,
    /// Enthalpy, `H(T)`.
    Enthalpy,
    /// Entropy, `S(T)`.
    Entropy,
    /// Gibbs energy, `G(T) = H(T) - T S(T)`.
    GibbsEnergy,
}

/// Errors raised while evaluating one already validated ordinary FDB range.
#[derive(Debug, Clone, PartialEq)]
pub enum FdbThermodynamicEvaluationError {
    /// Temperature was NaN or infinite.
    NonFiniteTemperature {
        /// Rejected temperature in kelvin.
        temperature_k: f64,
    },
    /// Absolute temperature must be positive for real powers and logarithms.
    NonPositiveTemperature {
        /// Rejected temperature in kelvin.
        temperature_k: f64,
    },
    /// Public evaluation never extrapolates outside the stored interval.
    TemperatureOutsideRange {
        /// Requested temperature in kelvin.
        temperature_k: f64,
        /// Stored lower bound in kelvin.
        temperature_min_k: f64,
        /// Stored upper bound in kelvin.
        temperature_max_k: f64,
    },
    /// A finite stored term produced a non-finite mathematical contribution.
    NonFiniteTermEvaluation {
        /// Physical source CP chunk index.
        source_chunk_index: usize,
        /// Zero-based coefficient/power term index.
        term_index: usize,
        /// Quantity whose term could not be evaluated finitely.
        quantity: FdbThermodynamicQuantity,
        /// Evaluation temperature in kelvin.
        temperature_k: f64,
    },
    /// Accumulation or the final H/S/G expression became non-finite.
    NonFiniteResult {
        /// Physical source CP chunk index.
        source_chunk_index: usize,
        /// Quantity whose result was non-finite.
        quantity: FdbThermodynamicQuantity,
        /// Evaluation temperature in kelvin.
        temperature_k: f64,
    },
    /// Conversion from the compound's native energy unit failed.
    Unit(UnitError),
}

impl fmt::Display for FdbThermodynamicEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteTemperature { temperature_k } => {
                write!(formatter, "non-finite temperature {temperature_k}")
            }
            Self::NonPositiveTemperature { temperature_k } => {
                write!(formatter, "temperature must be positive: {temperature_k}")
            }
            Self::TemperatureOutsideRange {
                temperature_k,
                temperature_min_k,
                temperature_max_k,
            } => write!(
                formatter,
                "temperature {temperature_k} K is outside {temperature_min_k}..{temperature_max_k} K"
            ),
            Self::NonFiniteTermEvaluation {
                source_chunk_index,
                term_index,
                quantity,
                temperature_k,
            } => write!(
                formatter,
                "non-finite {quantity:?} term {term_index} at CP chunk {source_chunk_index} and {temperature_k} K"
            ),
            Self::NonFiniteResult {
                source_chunk_index,
                quantity,
                temperature_k,
            } => write!(
                formatter,
                "non-finite {quantity:?} result at CP chunk {source_chunk_index} and {temperature_k} K"
            ),
            Self::Unit(error) => write!(formatter, "unit conversion error: {error}"),
        }
    }
}

impl std::error::Error for FdbThermodynamicEvaluationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unit(error) => Some(error),
            _ => None,
        }
    }
}

impl From<UnitError> for FdbThermodynamicEvaluationError {
    fn from(error: UnitError) -> Self {
        Self::Unit(error)
    }
}

/// Header evidence relevant when a CMPD-family database is assigned the
/// logical FactSage Function Database role.
///
/// This is intentionally a compatibility check, not an intrinsic CDB/FDB
/// classifier. Local evidence establishes that all examined FDB files use a
/// zero `read_flag`, but valid CDB files can also use that value. Callers must
/// retain their explicit logical bundle role and treat
/// [`Self::FunctionCompatible`] as necessary but not sufficient evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompoundDatabaseProfileEvidence {
    /// The observed FDB header guardrail is satisfied.
    FunctionCompatible,
    /// The header does not satisfy the observed FDB `read_flag` guardrail.
    FunctionGuardrailMismatch {
        /// The preserved raw `read_flag` value.
        read_flag: u8,
    },
}

impl CompoundDatabaseProfileEvidence {
    /// Classifies the header only as far as the established evidence permits.
    pub const fn from_read_flag(read_flag: u8) -> Self {
        if read_flag == 0 {
            Self::FunctionCompatible
        } else {
            Self::FunctionGuardrailMismatch { read_flag }
        }
    }
}

/// The provider H/S meaning exposed by a heat-capacity range.
///
/// The constants are not local lower- or upper-boundary anchors. They are
/// per-range values at [`STANDARD_REFERENCE_TEMPERATURE_K`], adjusted between
/// adjacent ranges so that H and S are continuous at a shared boundary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeatCapacityAnchorSemantics {
    /// Stored enthalpy and entropy are values at 298.15 K.
    StandardReferenceTemperature {
        /// The established reference temperature in kelvin.
        temperature_k: f64,
    },
}

/// A borrowed validated FDB/CMPD heat-capacity range.
///
/// The range retains its physical source chunk identity and the original CP
/// chunk kind. It deliberately exposes provider coefficients rather than a
/// cross-provider Gibbs representation.
#[derive(Debug, Clone, Copy)]
pub struct PhaseHeatCapacityRangeView<'a> {
    range: HeatCapacityRangeView<'a>,
    energy_unit: EnergyUnit,
}

impl<'a> PhaseHeatCapacityRangeView<'a> {
    const fn new(range: HeatCapacityRangeView<'a>, energy_unit: EnergyUnit) -> Self {
        Self { range, energy_unit }
    }

    /// Returns the physical source CP chunk index in the parsed file.
    pub const fn source_chunk_index(self) -> usize {
        self.range.chunk_index()
    }

    /// Returns the preserved original CP chunk kind.
    pub const fn kind(self) -> HeatCapacityKind {
        self.range.kind()
    }

    /// Returns the lower supported temperature in kelvin.
    pub const fn temperature_min_k(self) -> f64 {
        self.range.raw().temperature_min
    }

    /// Returns the upper supported temperature in kelvin.
    pub const fn temperature_max_k(self) -> f64 {
        self.range.raw().temperature_max
    }

    /// Returns the established interpretation of the stored H/S constants.
    pub const fn anchor_semantics(self) -> HeatCapacityAnchorSemantics {
        HeatCapacityAnchorSemantics::StandardReferenceTemperature {
            temperature_k: STANDARD_REFERENCE_TEMPERATURE_K,
        }
    }

    /// Returns the stored enthalpy at the standard reference temperature in
    /// the compound's native energy unit per formula unit.
    pub const fn reference_enthalpy_raw(self) -> f64 {
        self.range.raw().enthalpy
    }

    /// Converts the reference enthalpy to joules per formula unit.
    pub fn reference_enthalpy_j_per_mol(self) -> Result<f64, UnitError> {
        self.energy_unit.to_joules(self.reference_enthalpy_raw())
    }

    /// Returns the stored entropy at the standard reference temperature in
    /// the compound's native energy unit per formula unit kelvin.
    pub const fn reference_entropy_raw(self) -> f64 {
        self.range.raw().entropy
    }

    /// Converts the reference entropy to joules per formula unit kelvin.
    pub fn reference_entropy_j_per_mol_k(self) -> Result<f64, UnitError> {
        self.energy_unit.to_joules(self.reference_entropy_raw())
    }

    /// Returns the compound energy unit shared by this range's stored values.
    pub const fn energy_unit(self) -> EnergyUnit {
        self.energy_unit
    }

    /// Iterates the original coefficient/power pairs in stored order.
    pub fn heat_capacity_terms(self) -> impl Iterator<Item = (f64, f64)> + 'a {
        self.range
            .raw()
            .coefficients
            .iter()
            .copied()
            .zip(self.range.raw().powers.iter().copied())
    }

    /// Evaluates `Cp(T)` in the compound's native energy unit per formula unit
    /// kelvin without extrapolating outside this range.
    pub fn heat_capacity_raw_at(
        self,
        temperature_k: f64,
    ) -> Result<f64, FdbThermodynamicEvaluationError> {
        self.validate_public_temperature(temperature_k)?;
        self.evaluate_heat_capacity_raw(temperature_k)
    }

    /// Evaluates `Cp(T)` in joules per formula unit kelvin.
    pub fn heat_capacity_j_per_mol_k_at(
        self,
        temperature_k: f64,
    ) -> Result<f64, FdbThermodynamicEvaluationError> {
        Ok(self
            .energy_unit
            .to_joules(self.heat_capacity_raw_at(temperature_k)?)?)
    }

    /// Evaluates `H(T)` from the stored 298.15 K H anchor and the analytical
    /// integral of this range's `Cp(T)` expression.
    pub fn enthalpy_raw_at(
        self,
        temperature_k: f64,
    ) -> Result<f64, FdbThermodynamicEvaluationError> {
        self.validate_public_temperature(temperature_k)?;
        self.evaluate_enthalpy_raw_unbounded(temperature_k)
    }

    /// Evaluates `H(T)` in joules per formula unit.
    pub fn enthalpy_j_per_mol_at(
        self,
        temperature_k: f64,
    ) -> Result<f64, FdbThermodynamicEvaluationError> {
        Ok(self
            .energy_unit
            .to_joules(self.enthalpy_raw_at(temperature_k)?)?)
    }

    /// Evaluates `S(T)` from the stored 298.15 K S anchor and the analytical
    /// integral of `Cp(T) / T`.
    pub fn entropy_raw_at(
        self,
        temperature_k: f64,
    ) -> Result<f64, FdbThermodynamicEvaluationError> {
        self.validate_public_temperature(temperature_k)?;
        self.evaluate_entropy_raw_unbounded(temperature_k)
    }

    /// Evaluates `S(T)` in joules per formula unit kelvin.
    pub fn entropy_j_per_mol_k_at(
        self,
        temperature_k: f64,
    ) -> Result<f64, FdbThermodynamicEvaluationError> {
        Ok(self
            .energy_unit
            .to_joules(self.entropy_raw_at(temperature_k)?)?)
    }

    /// Evaluates the CP-backed provider expression `G(T) = H(T) - T S(T)` in
    /// the compound's native energy unit per formula unit.
    ///
    /// This mathematical range view does not by itself claim that magnetic,
    /// pressure-volume, transition, or ID-11 contributions are absent. Check
    /// [`OrdinaryPhaseThermodynamicView::effective_g_eligibility`] before
    /// treating it as a complete ordinary-phase Gibbs function.
    pub fn gibbs_energy_raw_at(
        self,
        temperature_k: f64,
    ) -> Result<f64, FdbThermodynamicEvaluationError> {
        self.validate_public_temperature(temperature_k)?;
        let value = self.evaluate_enthalpy_raw_unbounded(temperature_k)?
            - temperature_k * self.evaluate_entropy_raw_unbounded(temperature_k)?;
        self.ensure_finite_result(value, FdbThermodynamicQuantity::GibbsEnergy, temperature_k)
    }

    /// Evaluates the CP-backed `G(T)` expression in joules per formula unit.
    pub fn gibbs_energy_j_per_mol_at(
        self,
        temperature_k: f64,
    ) -> Result<f64, FdbThermodynamicEvaluationError> {
        Ok(self
            .energy_unit
            .to_joules(self.gibbs_energy_raw_at(temperature_k)?)?)
    }

    fn validate_public_temperature(
        self,
        temperature_k: f64,
    ) -> Result<(), FdbThermodynamicEvaluationError> {
        validate_evaluation_temperature(temperature_k)?;
        if temperature_k < self.temperature_min_k() || temperature_k > self.temperature_max_k() {
            return Err(FdbThermodynamicEvaluationError::TemperatureOutsideRange {
                temperature_k,
                temperature_min_k: self.temperature_min_k(),
                temperature_max_k: self.temperature_max_k(),
            });
        }
        Ok(())
    }

    fn evaluate_heat_capacity_raw(
        self,
        temperature_k: f64,
    ) -> Result<f64, FdbThermodynamicEvaluationError> {
        self.sum_terms(
            FdbThermodynamicQuantity::HeatCapacity,
            temperature_k,
            |c, p| c * temperature_k.powf(p),
        )
    }

    fn evaluate_enthalpy_raw_unbounded(
        self,
        temperature_k: f64,
    ) -> Result<f64, FdbThermodynamicEvaluationError> {
        validate_evaluation_temperature(temperature_k)?;
        let increment = self.sum_terms(
            FdbThermodynamicQuantity::Enthalpy,
            temperature_k,
            |coefficient, power| coefficient * integrate_power(temperature_k, power + 1.0),
        )?;
        self.ensure_finite_result(
            self.reference_enthalpy_raw() + increment,
            FdbThermodynamicQuantity::Enthalpy,
            temperature_k,
        )
    }

    fn evaluate_entropy_raw_unbounded(
        self,
        temperature_k: f64,
    ) -> Result<f64, FdbThermodynamicEvaluationError> {
        validate_evaluation_temperature(temperature_k)?;
        let increment = self.sum_terms(
            FdbThermodynamicQuantity::Entropy,
            temperature_k,
            |coefficient, power| coefficient * integrate_power(temperature_k, power),
        )?;
        self.ensure_finite_result(
            self.reference_entropy_raw() + increment,
            FdbThermodynamicQuantity::Entropy,
            temperature_k,
        )
    }

    fn sum_terms(
        self,
        quantity: FdbThermodynamicQuantity,
        temperature_k: f64,
        evaluate: impl Fn(f64, f64) -> f64,
    ) -> Result<f64, FdbThermodynamicEvaluationError> {
        let mut sum = 0.0;
        for (term_index, (coefficient, power)) in self.heat_capacity_terms().enumerate() {
            let term = evaluate(coefficient, power);
            if !term.is_finite() {
                return Err(FdbThermodynamicEvaluationError::NonFiniteTermEvaluation {
                    source_chunk_index: self.source_chunk_index(),
                    term_index,
                    quantity,
                    temperature_k,
                });
            }
            sum += term;
            if !sum.is_finite() {
                return Err(FdbThermodynamicEvaluationError::NonFiniteResult {
                    source_chunk_index: self.source_chunk_index(),
                    quantity,
                    temperature_k,
                });
            }
        }
        Ok(sum)
    }

    fn ensure_finite_result(
        self,
        value: f64,
        quantity: FdbThermodynamicQuantity,
        temperature_k: f64,
    ) -> Result<f64, FdbThermodynamicEvaluationError> {
        if value.is_finite() {
            Ok(value)
        } else {
            Err(FdbThermodynamicEvaluationError::NonFiniteResult {
                source_chunk_index: self.source_chunk_index(),
                quantity,
                temperature_k,
            })
        }
    }
}

fn validate_evaluation_temperature(
    temperature_k: f64,
) -> Result<(), FdbThermodynamicEvaluationError> {
    if !temperature_k.is_finite() {
        return Err(FdbThermodynamicEvaluationError::NonFiniteTemperature { temperature_k });
    }
    if temperature_k <= 0.0 {
        return Err(FdbThermodynamicEvaluationError::NonPositiveTemperature { temperature_k });
    }
    Ok(())
}

fn integrate_power(temperature_k: f64, integrated_power: f64) -> f64 {
    let log_ratio = (temperature_k / STANDARD_REFERENCE_TEMPERATURE_K).ln();
    if integrated_power == 0.0 {
        log_ratio
    } else {
        STANDARD_REFERENCE_TEMPERATURE_K.powf(integrated_power)
            * (integrated_power * log_ratio).exp_m1()
            / integrated_power
    }
}

/// A validated ordinary-phase provider thermodynamic definition.
///
/// It supports only finite, positive, contiguous source-order CP ranges of one
/// preserved CP kind. Gaps, overlaps, reordered ranges, mixed kinds, and
/// missing ranges remain typed failures rather than receiving invented
/// extrapolation or reordering.
#[derive(Debug, Clone)]
pub struct OrdinaryPhaseThermodynamicView<'a> {
    phase: PhaseView<'a>,
    energy_unit: EnergyUnit,
    heat_capacity_ranges: Vec<PhaseHeatCapacityRangeView<'a>>,
}

impl<'a> OrdinaryPhaseThermodynamicView<'a> {
    /// Returns the ordinary phase that owns this provider definition.
    pub fn phase(&self) -> PhaseView<'a> {
        self.phase
    }

    /// Returns the exact stored raw phase ID.
    pub fn phase_id_raw(&self) -> i32 {
        self.phase.phase_id_raw()
    }

    /// Returns the compound unit shared by all stored H/S and CP values.
    pub fn energy_unit(&self) -> EnergyUnit {
        self.energy_unit
    }

    /// Returns the established absolute reference temperature in kelvin.
    pub fn reference_temperature_k(&self) -> f64 {
        STANDARD_REFERENCE_TEMPERATURE_K
    }

    /// Returns the ordered, validated source ranges without reordering them.
    pub fn heat_capacity_ranges(&self) -> &[PhaseHeatCapacityRangeView<'a>] {
        &self.heat_capacity_ranges
    }

    /// Reports whether the CP-backed H/S/G view is complete for the physical
    /// contributions that this parser can currently identify.
    ///
    /// This is an eligibility guardrail, not an evaluator for magnetic,
    /// pressure-volume, or ID-11 equations. A result other than
    /// [`OrdinaryFdbEffectiveGEligibility::PureCpBacked`] means downstream code
    /// must not present the CP-only Gibbs expression as complete.
    pub fn effective_g_eligibility(&self) -> OrdinaryFdbEffectiveGEligibility {
        ordinary_effective_g_eligibility(self.phase)
    }
}

/// The established structural parent relation of an ID-8 transition record.
///
/// Parent linkage is established by raw phase ID. The thermodynamic effects of
/// the transition, including any entropy-jump rule, remain deliberately absent
/// until independently verified.
#[derive(Debug, Clone, Copy)]
pub enum TransitionParentRelation<'a> {
    /// Exactly one ordinary phase has the preserved parent ID.
    Ordinary(PhaseView<'a>),
    /// Exactly one transition phase has the preserved parent ID.
    Transition(PhaseView<'a>),
    /// No phase has the preserved parent ID.
    Missing {
        /// The parent phase ID stored by the ID-8 record.
        parent_phase_id_raw: i32,
    },
    /// Multiple phases have the preserved parent ID.
    Ambiguous {
        /// The parent phase ID stored by the ID-8 record.
        parent_phase_id_raw: i32,
        /// Number of matching phase records.
        matching_phase_count: usize,
    },
}

/// A provider structural view of an ID-8 transition record.
///
/// This value intentionally does not claim an effective Gibbs function.
#[derive(Debug, Clone, Copy)]
pub struct TransitionPhaseThermodynamicView<'a> {
    phase: PhaseView<'a>,
    energy_unit: EnergyUnit,
    parent: TransitionParentRelation<'a>,
}

impl<'a> TransitionPhaseThermodynamicView<'a> {
    /// Returns the transition phase record.
    pub const fn phase(self) -> PhaseView<'a> {
        self.phase
    }

    /// Returns the exact stored raw phase ID.
    pub const fn phase_id_raw(self) -> i32 {
        self.phase.phase_id_raw()
    }

    /// Returns the typed structural parent relation.
    pub const fn parent_relation(self) -> TransitionParentRelation<'a> {
        self.parent
    }

    /// Returns the preserved transition temperature in kelvin.
    pub fn transition_temperature_k(self) -> f64 {
        match self.phase.raw() {
            RawPhase::Transition(chunk) => chunk.transition_temperature,
            RawPhase::Ordinary(_) => unreachable!("transition view always contains ID-8"),
        }
    }

    /// Returns the transition enthalpy in the compound's native energy unit
    /// per formula unit.
    pub fn transition_enthalpy_raw(self) -> f64 {
        match self.phase.raw() {
            RawPhase::Transition(chunk) => chunk.transition_enthalpy,
            RawPhase::Ordinary(_) => unreachable!("transition view always contains ID-8"),
        }
    }

    /// Converts transition enthalpy to joules per formula unit.
    pub fn transition_enthalpy_j_per_mol(self) -> Result<f64, UnitError> {
        self.energy_unit.to_joules(self.transition_enthalpy_raw())
    }
}

/// A provider thermodynamic view of a phase.
#[derive(Debug, Clone)]
pub enum PhaseThermodynamicView<'a> {
    /// A validated CP-backed ordinary phase. Downstream normalization must also
    /// require [`OrdinaryPhaseThermodynamicView::effective_g_eligibility`] to be
    /// [`OrdinaryFdbEffectiveGEligibility::PureCpBacked`] before claiming a
    /// complete effective Gibbs function.
    Ordinary(OrdinaryPhaseThermodynamicView<'a>),
    /// An ID-8 structural transition relation whose effective-G semantics are
    /// not yet established.
    Transition(TransitionPhaseThermodynamicView<'a>),
}

/// Errors raised while constructing the evidence-backed provider view.
#[derive(Debug, Clone, PartialEq)]
pub enum PhaseThermodynamicViewError {
    /// The requested phase index was absent from the owning compound.
    InvalidPhaseIndex {
        /// The requested zero-based phase index.
        phase_index: usize,
    },
    /// An ordinary phase lacks CP records, so no H/S/Cp function is established.
    MissingHeatCapacityRanges {
        /// The ordinary phase's preserved raw ID.
        phase_id_raw: i32,
    },
    /// An FDB/CMPD phase uses multiple preserved CP chunk kinds.
    MixedHeatCapacityKinds {
        /// The phase's preserved raw ID.
        phase_id_raw: i32,
        /// The first kind encountered in source order.
        first_kind: HeatCapacityKind,
        /// The incompatible later kind.
        later_kind: HeatCapacityKind,
    },
    /// A range has non-finite or non-positive/inverted bounds.
    InvalidTemperatureRange {
        /// The physical CP chunk index.
        source_chunk_index: usize,
        /// The stored lower temperature.
        temperature_min_k: f64,
        /// The stored upper temperature.
        temperature_max_k: f64,
    },
    /// A coefficient or power is non-finite.
    NonFiniteHeatCapacityTerm {
        /// The physical CP chunk index.
        source_chunk_index: usize,
        /// The zero-based term index.
        term_index: usize,
        /// Whether the invalid value was a coefficient or power.
        field: &'static str,
        /// The invalid stored value.
        value: f64,
    },
    /// A stored H/S anchor is not finite.
    NonFiniteReferenceAnchor {
        /// Physical CP chunk index.
        source_chunk_index: usize,
        /// Whether the invalid value was enthalpy or entropy.
        quantity: FdbThermodynamicQuantity,
        /// Invalid stored value.
        value: f64,
    },
    /// Mathematical evaluation needed for provider validation became non-finite.
    RangeEvaluation {
        /// Underlying typed range-evaluation failure.
        error: FdbThermodynamicEvaluationError,
    },
    /// Source-order ranges leave an unsupported temperature gap.
    TemperatureGap {
        /// The preceding physical CP chunk index.
        previous_chunk_index: usize,
        /// The later physical CP chunk index.
        current_chunk_index: usize,
        /// The preceding range upper bound.
        previous_temperature_max_k: f64,
        /// The later range lower bound.
        current_temperature_min_k: f64,
    },
    /// Source-order ranges overlap, duplicate, or run backward.
    OverlappingOrUnorderedRanges {
        /// The preceding physical CP chunk index.
        previous_chunk_index: usize,
        /// The later physical CP chunk index.
        current_chunk_index: usize,
        /// The preceding range upper bound.
        previous_temperature_max_k: f64,
        /// The later range lower bound.
        current_temperature_min_k: f64,
    },
    /// Adjacent ranges produce materially different enthalpy at their shared boundary.
    EnthalpyDiscontinuity {
        /// Preceding physical CP chunk index.
        previous_chunk_index: usize,
        /// Following physical CP chunk index.
        next_chunk_index: usize,
        /// Shared boundary temperature in kelvin.
        boundary_temperature_k: f64,
        /// Enthalpy evaluated from the preceding range in native energy units.
        left_value: f64,
        /// Enthalpy evaluated from the following range in native energy units.
        right_value: f64,
        /// Absolute boundary difference in native energy units.
        difference: f64,
        /// Provider-validation allowance applied to this pair.
        tolerance: f64,
    },
    /// Adjacent ranges produce materially different entropy at their shared boundary.
    EntropyDiscontinuity {
        /// Preceding physical CP chunk index.
        previous_chunk_index: usize,
        /// Following physical CP chunk index.
        next_chunk_index: usize,
        /// Shared boundary temperature in kelvin.
        boundary_temperature_k: f64,
        /// Entropy evaluated from the preceding range in native energy units per kelvin.
        left_value: f64,
        /// Entropy evaluated from the following range in native energy units per kelvin.
        right_value: f64,
        /// Absolute boundary difference in native energy units per kelvin.
        difference: f64,
        /// Provider-validation allowance applied to this pair.
        tolerance: f64,
    },
}

impl fmt::Display for PhaseThermodynamicViewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPhaseIndex { phase_index } => {
                write!(formatter, "compound has no phase at index {phase_index}")
            }
            Self::MissingHeatCapacityRanges { phase_id_raw } => {
                write!(formatter, "ordinary phase {phase_id_raw} has no CP ranges")
            }
            Self::MixedHeatCapacityKinds {
                phase_id_raw,
                first_kind,
                later_kind,
            } => write!(
                formatter,
                "ordinary phase {phase_id_raw} mixes CP kinds {first_kind:?} and {later_kind:?}"
            ),
            Self::InvalidTemperatureRange {
                source_chunk_index,
                temperature_min_k,
                temperature_max_k,
            } => write!(
                formatter,
                "invalid CP range at chunk {source_chunk_index}: {temperature_min_k}..{temperature_max_k} K"
            ),
            Self::NonFiniteHeatCapacityTerm {
                source_chunk_index,
                term_index,
                field,
                value,
            } => write!(
                formatter,
                "non-finite CP {field} {term_index} at chunk {source_chunk_index}: {value}"
            ),
            Self::NonFiniteReferenceAnchor {
                source_chunk_index,
                quantity,
                value,
            } => write!(
                formatter,
                "non-finite {quantity:?} reference anchor at CP chunk {source_chunk_index}: {value}"
            ),
            Self::RangeEvaluation { error } => {
                write!(formatter, "FDB range validation evaluation failed: {error}")
            }
            Self::TemperatureGap {
                previous_chunk_index,
                current_chunk_index,
                previous_temperature_max_k,
                current_temperature_min_k,
            } => write!(
                formatter,
                "unsupported CP gap between chunks {previous_chunk_index} and {current_chunk_index}: {previous_temperature_max_k}..{current_temperature_min_k} K"
            ),
            Self::OverlappingOrUnorderedRanges {
                previous_chunk_index,
                current_chunk_index,
                previous_temperature_max_k,
                current_temperature_min_k,
            } => write!(
                formatter,
                "overlapping or unordered CP ranges at chunks {previous_chunk_index} and {current_chunk_index}: {previous_temperature_max_k} then {current_temperature_min_k} K"
            ),
            Self::EnthalpyDiscontinuity {
                previous_chunk_index,
                next_chunk_index,
                boundary_temperature_k,
                left_value,
                right_value,
                difference,
                tolerance,
            } => write!(
                formatter,
                "enthalpy discontinuity between CP chunks {previous_chunk_index} and {next_chunk_index} at {boundary_temperature_k} K: {left_value} versus {right_value}, difference {difference} exceeds {tolerance}"
            ),
            Self::EntropyDiscontinuity {
                previous_chunk_index,
                next_chunk_index,
                boundary_temperature_k,
                left_value,
                right_value,
                difference,
                tolerance,
            } => write!(
                formatter,
                "entropy discontinuity between CP chunks {previous_chunk_index} and {next_chunk_index} at {boundary_temperature_k} K: {left_value} versus {right_value}, difference {difference} exceeds {tolerance}"
            ),
        }
    }
}

impl std::error::Error for PhaseThermodynamicViewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RangeEvaluation { error } => Some(error),
            _ => None,
        }
    }
}

impl<'a> CompoundView<'a> {
    /// Returns typed CMPD header evidence for the Function Database role.
    ///
    /// A compatible result is deliberately not treated as proof that a file is
    /// an FDB: observed CDB files share the same header guardrail.
    pub fn database_profile_evidence(self) -> CompoundDatabaseProfileEvidence {
        let Some(RawChunk::DatabaseHeader(header)) = self.raw_chunks.first() else {
            unreachable!("DatabaseView always begins with a database header");
        };
        CompoundDatabaseProfileEvidence::from_read_flag(header.read_flag)
    }

    /// Builds a provider-level thermodynamic view for one phase in this
    /// compound. Ordinary phases require an unambiguous, contiguous CP-backed
    /// definition; transition phases return only their established structural
    /// parent relation.
    pub fn fdb_phase_thermodynamic_view(
        self,
        phase_index: usize,
    ) -> Result<PhaseThermodynamicView<'a>, PhaseThermodynamicViewError> {
        let phase = self
            .phases()
            .nth(phase_index)
            .ok_or(PhaseThermodynamicViewError::InvalidPhaseIndex { phase_index })?;
        let energy_unit = self.energy_unit();
        match phase.raw() {
            RawPhase::Ordinary(_) => {
                let ranges = phase
                    .heat_capacity_ranges()
                    .map(|range| PhaseHeatCapacityRangeView::new(range, energy_unit))
                    .collect::<Vec<_>>();
                validate_ordinary_ranges(phase.phase_id_raw(), &ranges)?;
                Ok(PhaseThermodynamicView::Ordinary(
                    OrdinaryPhaseThermodynamicView {
                        phase,
                        energy_unit,
                        heat_capacity_ranges: ranges,
                    },
                ))
            }
            RawPhase::Transition(chunk) => Ok(PhaseThermodynamicView::Transition(
                TransitionPhaseThermodynamicView {
                    phase,
                    energy_unit,
                    parent: transition_parent_relation(self, chunk.parent_phase_id_raw),
                },
            )),
        }
    }
}

impl DatabaseView<'_> {
    /// Returns typed header evidence for an explicitly assigned Function
    /// Database role. This remains compatibility evidence, not a role
    /// classifier.
    pub fn database_profile_evidence(self) -> CompoundDatabaseProfileEvidence {
        CompoundDatabaseProfileEvidence::from_read_flag(self.header().read_flag)
    }
}

fn transition_parent_relation(
    compound: CompoundView<'_>,
    parent_phase_id_raw: i32,
) -> TransitionParentRelation<'_> {
    let matches = compound
        .phases()
        .filter(|phase| phase.phase_id_raw() == parent_phase_id_raw)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => TransitionParentRelation::Missing {
            parent_phase_id_raw,
        },
        [phase] => match phase.raw() {
            RawPhase::Ordinary(_) => TransitionParentRelation::Ordinary(*phase),
            RawPhase::Transition(_) => TransitionParentRelation::Transition(*phase),
        },
        _ => TransitionParentRelation::Ambiguous {
            parent_phase_id_raw,
            matching_phase_count: matches.len(),
        },
    }
}

fn validate_ordinary_ranges(
    phase_id_raw: i32,
    ranges: &[PhaseHeatCapacityRangeView<'_>],
) -> Result<(), PhaseThermodynamicViewError> {
    let Some(first) = ranges.first().copied() else {
        return Err(PhaseThermodynamicViewError::MissingHeatCapacityRanges { phase_id_raw });
    };
    let first_kind = first.kind();
    let mut previous: Option<PhaseHeatCapacityRangeView<'_>> = None;

    for range in ranges {
        validate_range(*range)?;
        if range.kind() != first_kind {
            return Err(PhaseThermodynamicViewError::MixedHeatCapacityKinds {
                phase_id_raw,
                first_kind,
                later_kind: range.kind(),
            });
        }
        if let Some(previous_range) = previous {
            let previous_max = previous_range.temperature_max_k();
            let current_min = range.temperature_min_k();
            if previous_max.to_bits() != current_min.to_bits() {
                let error = if previous_max < current_min {
                    PhaseThermodynamicViewError::TemperatureGap {
                        previous_chunk_index: previous_range.source_chunk_index(),
                        current_chunk_index: range.source_chunk_index(),
                        previous_temperature_max_k: previous_max,
                        current_temperature_min_k: current_min,
                    }
                } else {
                    PhaseThermodynamicViewError::OverlappingOrUnorderedRanges {
                        previous_chunk_index: previous_range.source_chunk_index(),
                        current_chunk_index: range.source_chunk_index(),
                        previous_temperature_max_k: previous_max,
                        current_temperature_min_k: current_min,
                    }
                };
                return Err(error);
            }
            validate_boundary_continuity(previous_range, *range, previous_max)?;
        }
        previous = Some(*range);
    }
    Ok(())
}

fn validate_range(
    range: PhaseHeatCapacityRangeView<'_>,
) -> Result<(), PhaseThermodynamicViewError> {
    let temperature_min_k = range.temperature_min_k();
    let temperature_max_k = range.temperature_max_k();
    if !temperature_min_k.is_finite()
        || !temperature_max_k.is_finite()
        || temperature_min_k <= 0.0
        || temperature_min_k >= temperature_max_k
    {
        return Err(PhaseThermodynamicViewError::InvalidTemperatureRange {
            source_chunk_index: range.source_chunk_index(),
            temperature_min_k,
            temperature_max_k,
        });
    }
    for (term_index, (coefficient, power)) in range.heat_capacity_terms().enumerate() {
        if !coefficient.is_finite() {
            return Err(PhaseThermodynamicViewError::NonFiniteHeatCapacityTerm {
                source_chunk_index: range.source_chunk_index(),
                term_index,
                field: "coefficient",
                value: coefficient,
            });
        }
        if !power.is_finite() {
            return Err(PhaseThermodynamicViewError::NonFiniteHeatCapacityTerm {
                source_chunk_index: range.source_chunk_index(),
                term_index,
                field: "power",
                value: power,
            });
        }
    }
    for (quantity, value) in [
        (
            FdbThermodynamicQuantity::Enthalpy,
            range.reference_enthalpy_raw(),
        ),
        (
            FdbThermodynamicQuantity::Entropy,
            range.reference_entropy_raw(),
        ),
    ] {
        if !value.is_finite() {
            return Err(PhaseThermodynamicViewError::NonFiniteReferenceAnchor {
                source_chunk_index: range.source_chunk_index(),
                quantity,
                value,
            });
        }
    }
    Ok(())
}

fn validate_boundary_continuity(
    previous: PhaseHeatCapacityRangeView<'_>,
    next: PhaseHeatCapacityRangeView<'_>,
    boundary_temperature_k: f64,
) -> Result<(), PhaseThermodynamicViewError> {
    let left_enthalpy = previous
        .evaluate_enthalpy_raw_unbounded(boundary_temperature_k)
        .map_err(|error| PhaseThermodynamicViewError::RangeEvaluation { error })?;
    let right_enthalpy = next
        .evaluate_enthalpy_raw_unbounded(boundary_temperature_k)
        .map_err(|error| PhaseThermodynamicViewError::RangeEvaluation { error })?;
    let enthalpy_difference = (left_enthalpy - right_enthalpy).abs();
    let enthalpy_tolerance = continuity_tolerance(left_enthalpy, right_enthalpy);
    if enthalpy_difference > enthalpy_tolerance {
        return Err(PhaseThermodynamicViewError::EnthalpyDiscontinuity {
            previous_chunk_index: previous.source_chunk_index(),
            next_chunk_index: next.source_chunk_index(),
            boundary_temperature_k,
            left_value: left_enthalpy,
            right_value: right_enthalpy,
            difference: enthalpy_difference,
            tolerance: enthalpy_tolerance,
        });
    }

    let left_entropy = previous
        .evaluate_entropy_raw_unbounded(boundary_temperature_k)
        .map_err(|error| PhaseThermodynamicViewError::RangeEvaluation { error })?;
    let right_entropy = next
        .evaluate_entropy_raw_unbounded(boundary_temperature_k)
        .map_err(|error| PhaseThermodynamicViewError::RangeEvaluation { error })?;
    let entropy_difference = (left_entropy - right_entropy).abs();
    let entropy_tolerance = continuity_tolerance(left_entropy, right_entropy);
    if entropy_difference > entropy_tolerance {
        return Err(PhaseThermodynamicViewError::EntropyDiscontinuity {
            previous_chunk_index: previous.source_chunk_index(),
            next_chunk_index: next.source_chunk_index(),
            boundary_temperature_k,
            left_value: left_entropy,
            right_value: right_entropy,
            difference: entropy_difference,
            tolerance: entropy_tolerance,
        });
    }
    Ok(())
}

fn continuity_tolerance(left: f64, right: f64) -> f64 {
    PROVIDER_RANGE_CONTINUITY_ABSOLUTE_TOLERANCE
        + PROVIDER_RANGE_CONTINUITY_RELATIVE_TOLERANCE * left.abs().max(right.abs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Database;
    use crate::{CHUNK_SIZE, RawDatabaseHeaderChunk};

    fn put_f64(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: f64) {
        chunk[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    fn put_i32(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: i32) {
        chunk[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn put_f32(chunk: &mut [u8; CHUNK_SIZE], offset: usize, value: f32) {
        chunk[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn header() -> [u8; CHUNK_SIZE] {
        let mut chunk = [0_u8; CHUNK_SIZE];
        chunk[0] = 9;
        chunk[2..6].copy_from_slice(b"CMPD");
        chunk
    }

    fn compound() -> [u8; CHUNK_SIZE] {
        let mut chunk = [0_u8; CHUNK_SIZE];
        chunk[0] = 1;
        chunk[156..160].copy_from_slice(&1_u32.to_le_bytes());
        chunk[160..164].copy_from_slice(&1_u32.to_le_bytes());
        chunk
    }

    fn ordinary(phase_id_raw: i32) -> [u8; CHUNK_SIZE] {
        let mut chunk = [0_u8; CHUNK_SIZE];
        chunk[0] = 7;
        put_f64(&mut chunk, 32, -100.0);
        put_f64(&mut chunk, 40, 20.0);
        put_i32(&mut chunk, 48, -phase_id_raw);
        put_i32(&mut chunk, 52, phase_id_raw);
        chunk
    }

    fn kappa(phase_id_raw: i32, active_value: f64) -> [u8; CHUNK_SIZE] {
        let mut chunk = [0_u8; CHUNK_SIZE];
        chunk[0] = 11;
        put_f64(&mut chunk, 32, 298.15);
        put_f64(&mut chunk, 40, 1000.0);
        put_i32(&mut chunk, 48, phase_id_raw);
        put_f64(&mut chunk, 56, active_value);
        chunk
    }

    fn transition(phase_id_raw: i32, parent_phase_id_raw: i32) -> [u8; CHUNK_SIZE] {
        let mut chunk = [0_u8; CHUNK_SIZE];
        chunk[0] = 8;
        put_f64(&mut chunk, 32, 12.0);
        put_f64(&mut chunk, 40, 900.0);
        put_i32(&mut chunk, 48, parent_phase_id_raw);
        put_i32(&mut chunk, 52, phase_id_raw);
        chunk
    }

    fn cp(
        id: u8,
        phase_id_raw: i32,
        enthalpy: f64,
        entropy: f64,
        temperature_min: f64,
        temperature_max: f64,
        terms: &[(usize, f64, f64)],
    ) -> [u8; CHUNK_SIZE] {
        let mut chunk = [0_u8; CHUNK_SIZE];
        chunk[0] = id;
        put_f64(&mut chunk, 32, enthalpy);
        put_f64(&mut chunk, 40, entropy);
        put_i32(&mut chunk, 48, phase_id_raw);
        put_f64(&mut chunk, 56, temperature_min);
        put_f64(&mut chunk, 64, temperature_max);
        for (index, coefficient, power) in terms {
            put_f64(&mut chunk, 72 + index * 8, *coefficient);
            put_f64(&mut chunk, 136 + index * 8, *power);
        }
        chunk
    }

    fn database(chunks: Vec<[u8; CHUNK_SIZE]>) -> Database {
        let bytes = chunks.into_iter().flatten().collect::<Vec<_>>();
        Database::from_bytes(&bytes).unwrap()
    }

    fn assert_close(actual: f64, expected: f64) {
        let tolerance = 1.0e-10 * actual.abs().max(expected.abs()).max(1.0);
        assert!(
            (actual - expected).abs() <= tolerance,
            "{actual} != {expected} within {tolerance}"
        );
    }

    #[test]
    fn function_profile_guardrail_is_compatible_not_a_profile_proof() {
        assert_eq!(
            CompoundDatabaseProfileEvidence::from_read_flag(0),
            CompoundDatabaseProfileEvidence::FunctionCompatible
        );
        assert_eq!(
            CompoundDatabaseProfileEvidence::from_read_flag(1),
            CompoundDatabaseProfileEvidence::FunctionGuardrailMismatch { read_flag: 1 }
        );

        let raw_header = RawDatabaseHeaderChunk {
            padding_1: 0,
            magic: *b"CMPD",
            padding_2: [0; 2],
            date_ole: 0.0,
            read_flag: 0,
            unknown_1: [0; 11],
            comment: [0; 80],
            padding_3: [0; 136],
            unknown_2: [0; 12],
        };
        assert_eq!(
            CompoundDatabaseProfileEvidence::from_read_flag(raw_header.read_flag),
            CompoundDatabaseProfileEvidence::FunctionCompatible
        );

        let database = database(vec![header(), compound(), ordinary(101)]);
        let database_view = database.view().unwrap();
        assert_eq!(
            database_view.database_profile_evidence(),
            CompoundDatabaseProfileEvidence::FunctionCompatible
        );
        let compound = database_view.compounds().next().unwrap();
        assert_eq!(
            compound.database_profile_evidence(),
            CompoundDatabaseProfileEvidence::FunctionCompatible
        );
        assert!(matches!(
            compound.fdb_phase_thermodynamic_view(0),
            Err(PhaseThermodynamicViewError::MissingHeatCapacityRanges { phase_id_raw: 101 })
        ));
    }

    #[test]
    fn ordinary_view_exposes_298_15_anchors_and_source_order() {
        let boundary = 500.0;
        let reference = STANDARD_REFERENCE_TEMPERATURE_K;
        let first_h_at_boundary = -100.0 + 2.0 * (boundary - reference);
        let first_s_at_boundary = 20.0 + 2.0 * (boundary / reference).ln();
        let second_h_increment = 3.0 * (boundary / reference).ln()
            + (4.0 / 1.5) * (boundary.powf(1.5) - reference.powf(1.5));
        let second_s_increment = -3.0 * (boundary.powf(-1.0) - reference.powf(-1.0))
            + 8.0 * (boundary.sqrt() - reference.sqrt());
        let database = database(vec![
            header(),
            compound(),
            ordinary(101),
            cp(2, 101, -100.0, 20.0, 298.15, boundary, &[(0, 2.0, 0.0)]),
            cp(
                2,
                101,
                first_h_at_boundary - second_h_increment,
                first_s_at_boundary - second_s_increment,
                boundary,
                1000.0,
                &[(0, 3.0, -1.0), (1, 4.0, 0.5)],
            ),
        ]);
        let compound = database.view().unwrap().compounds().next().unwrap();
        let PhaseThermodynamicView::Ordinary(view) =
            compound.fdb_phase_thermodynamic_view(0).unwrap()
        else {
            panic!("expected an ordinary view");
        };

        assert_eq!(view.reference_temperature_k(), 298.15);
        assert_eq!(view.energy_unit(), EnergyUnit::Joules);
        assert_eq!(view.heat_capacity_ranges().len(), 2);
        let first = view.heat_capacity_ranges()[0];
        assert_eq!(
            first.anchor_semantics(),
            HeatCapacityAnchorSemantics::StandardReferenceTemperature {
                temperature_k: 298.15
            }
        );
        assert_eq!(first.reference_enthalpy_j_per_mol().unwrap(), -100.0);
        assert_eq!(first.reference_entropy_j_per_mol_k().unwrap(), 20.0);
        assert_eq!(
            view.heat_capacity_ranges()[1]
                .heat_capacity_terms()
                .collect::<Vec<_>>(),
            vec![
                (3.0, -1.0),
                (4.0, 0.5),
                (0.0, 0.0),
                (0.0, 0.0),
                (0.0, 0.0),
                (0.0, 0.0),
                (0.0, 0.0),
                (0.0, 0.0)
            ]
        );
    }

    #[test]
    fn analytical_integrals_cover_singular_and_finite_power_cases() {
        let powers = [-3.0, -2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0, 3.0];
        let temperature = 700.0;
        let reference = STANDARD_REFERENCE_TEMPERATURE_K;

        for power in powers {
            let database = database(vec![
                header(),
                compound(),
                ordinary(101),
                cp(2, 101, 11.0, 7.0, 200.0, 900.0, &[(0, 2.5, power)]),
            ]);
            let compound = database.view().unwrap().compounds().next().unwrap();
            let PhaseThermodynamicView::Ordinary(view) =
                compound.fdb_phase_thermodynamic_view(0).unwrap()
            else {
                unreachable!()
            };
            let range = view.heat_capacity_ranges()[0];
            let expected_h_increment = if power == -1.0 {
                2.5 * (temperature / reference).ln()
            } else {
                2.5 / (power + 1.0) * (temperature.powf(power + 1.0) - reference.powf(power + 1.0))
            };
            let expected_s_increment = if power == 0.0 {
                2.5 * (temperature / reference).ln()
            } else {
                2.5 / power * (temperature.powf(power) - reference.powf(power))
            };
            assert_close(
                range.enthalpy_raw_at(temperature).unwrap(),
                11.0 + expected_h_increment,
            );
            assert_close(
                range.entropy_raw_at(temperature).unwrap(),
                7.0 + expected_s_increment,
            );
            assert_close(
                range.heat_capacity_raw_at(temperature).unwrap(),
                2.5 * temperature.powf(power),
            );
            let expected_g =
                11.0 + expected_h_increment - temperature * (7.0 + expected_s_increment);
            assert_close(range.gibbs_energy_raw_at(temperature).unwrap(), expected_g);
        }
    }

    #[test]
    fn gibbs_derivatives_recover_source_entropy_and_heat_capacity() {
        let database = database(vec![
            header(),
            compound(),
            ordinary(101),
            cp(
                2,
                101,
                -125_000.0,
                45.0,
                250.0,
                1200.0,
                &[(0, 20.0, 0.0), (1, 0.01, 1.0), (2, 1500.0, -1.0)],
            ),
        ]);
        let compound_view = database.view().unwrap().compounds().next().unwrap();
        let PhaseThermodynamicView::Ordinary(view) =
            compound_view.fdb_phase_thermodynamic_view(0).unwrap()
        else {
            unreachable!()
        };
        let range = view.heat_capacity_ranges()[0];
        let temperature = 700.0;
        let step = 0.01;
        let lower = range.gibbs_energy_raw_at(temperature - step).unwrap();
        let center = range.gibbs_energy_raw_at(temperature).unwrap();
        let upper = range.gibbs_energy_raw_at(temperature + step).unwrap();
        let numerical_entropy = -(upper - lower) / (2.0 * step);
        let numerical_cp = -temperature * (upper - 2.0 * center + lower) / step.powi(2);
        assert!((numerical_entropy - range.entropy_raw_at(temperature).unwrap()).abs() < 1.0e-7);
        assert!((numerical_cp - range.heat_capacity_raw_at(temperature).unwrap()).abs() < 0.05);

        let log_limit = (temperature / STANDARD_REFERENCE_TEMPERATURE_K).ln();
        assert!((integrate_power(temperature, 1.0e-12) - log_limit).abs() < 1.0e-10);
    }

    #[test]
    fn validates_continuity_and_reports_h_and_s_discontinuities() {
        let boundary = 500.0;
        let reference = STANDARD_REFERENCE_TEMPERATURE_K;
        let next_h = 10.0 - (boundary - reference);
        let next_s = 5.0 - (boundary / reference).ln();
        let continuous = database(vec![
            header(),
            compound(),
            ordinary(101),
            cp(2, 101, 10.0, 5.0, 298.15, boundary, &[]),
            cp(2, 101, next_h, next_s, boundary, 1000.0, &[(0, 1.0, 0.0)]),
        ]);
        let compound_view = continuous.view().unwrap().compounds().next().unwrap();
        assert!(compound_view.fdb_phase_thermodynamic_view(0).is_ok());

        let discontinuous_h = database(vec![
            header(),
            compound(),
            ordinary(101),
            cp(2, 101, 10.0, 5.0, 298.15, boundary, &[]),
            cp(
                2,
                101,
                next_h + 1.0,
                next_s,
                boundary,
                1000.0,
                &[(0, 1.0, 0.0)],
            ),
        ]);
        let compound_view = discontinuous_h.view().unwrap().compounds().next().unwrap();
        assert!(matches!(
            compound_view.fdb_phase_thermodynamic_view(0),
            Err(PhaseThermodynamicViewError::EnthalpyDiscontinuity { .. })
        ));

        let discontinuous_s = database(vec![
            header(),
            compound(),
            ordinary(101),
            cp(2, 101, 10.0, 5.0, 298.15, boundary, &[]),
            cp(
                2,
                101,
                next_h,
                next_s + 1.0,
                boundary,
                1000.0,
                &[(0, 1.0, 0.0)],
            ),
        ]);
        let compound_view = discontinuous_s.view().unwrap().compounds().next().unwrap();
        assert!(matches!(
            compound_view.fdb_phase_thermodynamic_view(0),
            Err(PhaseThermodynamicViewError::EntropyDiscontinuity { .. })
        ));
    }

    #[test]
    fn ordinary_view_rejects_gaps_overlaps_mixed_kinds_and_nonfinite_terms() {
        let cases = [
            (
                vec![
                    header(),
                    compound(),
                    ordinary(101),
                    cp(2, 101, 0.0, 0.0, 298.15, 500.0, &[]),
                    cp(2, 101, 0.0, 0.0, 600.0, 1000.0, &[]),
                ],
                "gap",
            ),
            (
                vec![
                    header(),
                    compound(),
                    ordinary(101),
                    cp(2, 101, 0.0, 0.0, 298.15, 500.0, &[]),
                    cp(2, 101, 0.0, 0.0, 400.0, 1000.0, &[]),
                ],
                "overlap",
            ),
            (
                vec![
                    header(),
                    compound(),
                    ordinary(101),
                    cp(2, 101, 0.0, 0.0, 298.15, 500.0, &[]),
                    cp(4, 101, 0.0, 0.0, 500.0, 1000.0, &[]),
                ],
                "kind",
            ),
            (
                vec![
                    header(),
                    compound(),
                    ordinary(101),
                    cp(2, 101, 0.0, 0.0, 298.15, 500.0, &[(0, f64::NAN, 0.0)]),
                ],
                "term",
            ),
            (
                vec![
                    header(),
                    compound(),
                    ordinary(101),
                    cp(2, 101, 0.0, 0.0, 500.0, 400.0, &[]),
                ],
                "reversed",
            ),
        ];

        for (chunks, expected) in cases {
            let database = database(chunks);
            let compound = database.view().unwrap().compounds().next().unwrap();
            let error = compound.fdb_phase_thermodynamic_view(0).unwrap_err();
            match (expected, error) {
                ("gap", PhaseThermodynamicViewError::TemperatureGap { .. })
                | ("overlap", PhaseThermodynamicViewError::OverlappingOrUnorderedRanges { .. })
                | ("kind", PhaseThermodynamicViewError::MixedHeatCapacityKinds { .. })
                | ("term", PhaseThermodynamicViewError::NonFiniteHeatCapacityTerm { .. })
                | ("reversed", PhaseThermodynamicViewError::InvalidTemperatureRange { .. }) => {}
                (_, unexpected) => panic!("unexpected error {unexpected:?}"),
            }
        }
    }

    #[test]
    fn effective_g_eligibility_distinguishes_inactive_and_active_physics() {
        let pure = database(vec![
            header(),
            compound(),
            ordinary(101),
            cp(2, 101, 0.0, 0.0, 298.15, 1000.0, &[]),
        ]);
        let compound_view = pure.view().unwrap().compounds().next().unwrap();
        let PhaseThermodynamicView::Ordinary(view) =
            compound_view.fdb_phase_thermodynamic_view(0).unwrap()
        else {
            unreachable!()
        };
        assert_eq!(
            view.effective_g_eligibility(),
            OrdinaryFdbEffectiveGEligibility::PureCpBacked
        );

        let inactive_extended = database(vec![
            header(),
            compound(),
            ordinary(101),
            cp(2, 101, 0.0, 0.0, 298.15, 1000.0, &[]),
            kappa(101, 0.0),
        ]);
        let compound_view = inactive_extended
            .view()
            .unwrap()
            .compounds()
            .next()
            .unwrap();
        let PhaseThermodynamicView::Ordinary(view) =
            compound_view.fdb_phase_thermodynamic_view(0).unwrap()
        else {
            unreachable!()
        };
        assert_eq!(
            view.effective_g_eligibility(),
            OrdinaryFdbEffectiveGEligibility::PureCpBacked
        );

        let mut magnetic_phase = ordinary(101);
        put_f32(&mut magnetic_phase, 104, 1043.0);
        put_f32(&mut magnetic_phase, 108, 2.2);
        let magnetic = database(vec![
            header(),
            compound(),
            magnetic_phase,
            cp(2, 101, 0.0, 0.0, 298.15, 1000.0, &[]),
        ]);
        let compound_view = magnetic.view().unwrap().compounds().next().unwrap();
        let PhaseThermodynamicView::Ordinary(view) =
            compound_view.fdb_phase_thermodynamic_view(0).unwrap()
        else {
            unreachable!()
        };
        assert!(matches!(
            view.effective_g_eligibility(),
            OrdinaryFdbEffectiveGEligibility::RequiresMagneticSemantics {
                magnetic_temperature_active: true,
                magnetic_moment_active: true,
                p_factor_active: false
            }
        ));

        let mut pressure_phase = ordinary(101);
        put_f64(&mut pressure_phase, 56, 7.1);
        put_f32(&mut pressure_phase, 64, 1.0e-5);
        let pressure = database(vec![
            header(),
            compound(),
            pressure_phase,
            cp(2, 101, 0.0, 0.0, 298.15, 1000.0, &[]),
        ]);
        let compound_view = pressure.view().unwrap().compounds().next().unwrap();
        let PhaseThermodynamicView::Ordinary(view) =
            compound_view.fdb_phase_thermodynamic_view(0).unwrap()
        else {
            unreachable!()
        };
        assert!(matches!(
            view.effective_g_eligibility(),
            OrdinaryFdbEffectiveGEligibility::RequiresPressureSemantics {
                density_active: true,
                thermal_expansion_active: true,
                ..
            }
        ));

        let extended = database(vec![
            header(),
            compound(),
            ordinary(101),
            cp(2, 101, 0.0, 0.0, 298.15, 1000.0, &[]),
            kappa(101, 1.0),
        ]);
        let compound_view = extended.view().unwrap().compounds().next().unwrap();
        let PhaseThermodynamicView::Ordinary(view) =
            compound_view.fdb_phase_thermodynamic_view(0).unwrap()
        else {
            unreachable!()
        };
        assert_eq!(
            view.effective_g_eligibility(),
            OrdinaryFdbEffectiveGEligibility::RequiresExtendedPropertySemantics {
                linked_record_count: 1
            }
        );
    }

    #[test]
    fn transition_view_links_parent_without_claiming_effective_gibbs() {
        let database = database(vec![
            header(),
            compound(),
            ordinary(101),
            transition(102, 101),
        ]);
        let compound = database.view().unwrap().compounds().next().unwrap();
        let PhaseThermodynamicView::Transition(view) =
            compound.fdb_phase_thermodynamic_view(1).unwrap()
        else {
            panic!("expected a transition view");
        };
        assert_eq!(view.transition_temperature_k(), 900.0);
        assert_eq!(view.transition_enthalpy_j_per_mol().unwrap(), 12.0);
        assert!(matches!(
            view.parent_relation(),
            TransitionParentRelation::Ordinary(parent) if parent.phase_id_raw() == 101
        ));
    }
}

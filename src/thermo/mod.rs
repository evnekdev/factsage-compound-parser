//! Read-only thermodynamic decoding and stored-expression evaluation over views.
//!
//! These APIs borrow raw records through [`crate::domain::CompoundView`] and
//! [`crate::domain::PhaseView`]. They never alter the authoritative CDB stream
//! and do not infer unsupported FactSage semantics.

mod date;
mod density;
mod effective_g_eligibility;
mod error;
mod heat_capacity;
mod phase_thermodynamics;
mod units;

/// Re-export of OLE Automation date conversion.
pub use date::OleAutomationDate;
/// Re-export of the provisional density decoder.
pub use density::decode_density;
pub use effective_g_eligibility::OrdinaryFdbEffectiveGEligibility;
/// Re-exports for typed thermodynamic evaluation errors.
pub use error::{
    DateError, DensityError, HeatCapacityError, PhaseKind, PhaseProperty, PhaseThermoError,
    UnitError,
};
/// Re-exports for evidence-backed FDB/CMPD provider thermodynamic views.
pub use phase_thermodynamics::{
    CompoundDatabaseProfileEvidence, FdbThermodynamicEvaluationError, FdbThermodynamicQuantity,
    HeatCapacityAnchorSemantics, OrdinaryPhaseThermodynamicView,
    PROVIDER_RANGE_CONTINUITY_ABSOLUTE_TOLERANCE, PROVIDER_RANGE_CONTINUITY_RELATIVE_TOLERANCE,
    PhaseHeatCapacityRangeView, PhaseThermodynamicView, PhaseThermodynamicViewError,
    STANDARD_REFERENCE_TEMPERATURE_K, TransitionParentRelation, TransitionPhaseThermodynamicView,
};
/// Re-exports for stored compound unit codes.
pub use units::{EnergyUnit, PressureUnit};

use crate::domain::{CompoundView, PhaseView, RawPhase};

impl CompoundView<'_> {
    /// Returns the typed energy unit decoded from the raw compound code.
    ///
    /// Unknown codes are preserved as [`EnergyUnit::Unknown`] and only fail when
    /// a conversion is requested.
    pub fn energy_unit(self) -> EnergyUnit {
        EnergyUnit::from_raw(self.raw().unit_energy)
    }

    /// Returns the typed pressure unit decoded from the raw compound code.
    ///
    /// Unknown codes remain losslessly represented as [`PressureUnit::Unknown`].
    pub fn pressure_unit(self) -> PressureUnit {
        PressureUnit::from_raw(self.raw().unit_pressure)
    }

    /// Converts an ordinary phase enthalpy using this compound's energy unit.
    pub fn enthalpy_298_j_per_mol(self, phase_index: usize) -> Result<f64, PhaseThermoError> {
        let phase = self
            .phases()
            .nth(phase_index)
            .ok_or(PhaseThermoError::InvalidPhaseIndex { phase_index })?;
        phase.enthalpy_298_j_per_mol(self.energy_unit())
    }

    /// Converts an ordinary phase entropy using this compound's energy unit.
    pub fn entropy_298_j_per_mol_k(self, phase_index: usize) -> Result<f64, PhaseThermoError> {
        let phase = self
            .phases()
            .nth(phase_index)
            .ok_or(PhaseThermoError::InvalidPhaseIndex { phase_index })?;
        phase.entropy_298_j_per_mol_k(self.energy_unit())
    }

    /// Converts a transition enthalpy using this compound's energy unit.
    pub fn transition_enthalpy_j_per_mol(
        self,
        phase_index: usize,
    ) -> Result<f64, PhaseThermoError> {
        let phase = self
            .phases()
            .nth(phase_index)
            .ok_or(PhaseThermoError::InvalidPhaseIndex { phase_index })?;
        phase.transition_enthalpy_j_per_mol(self.energy_unit())
    }
}

impl PhaseView<'_> {
    /// Returns the ordinary-phase stored enthalpy at its 298 K anchor.
    pub fn enthalpy_298_raw(self) -> Result<f64, PhaseThermoError> {
        match self.raw() {
            RawPhase::Ordinary(chunk) => Ok(chunk.enthalpy),
            RawPhase::Transition(_) => Err(PhaseThermoError::WrongPhaseType {
                property: PhaseProperty::Enthalpy298,
                phase_kind: PhaseKind::Transition,
            }),
        }
    }

    /// Converts ordinary-phase stored enthalpy to joules per mole.
    pub fn enthalpy_298_j_per_mol(self, energy_unit: EnergyUnit) -> Result<f64, PhaseThermoError> {
        energy_unit
            .to_joules(self.enthalpy_298_raw()?)
            .map_err(PhaseThermoError::from)
    }

    /// Returns the ordinary-phase stored entropy at its 298 K anchor.
    pub fn entropy_298_raw(self) -> Result<f64, PhaseThermoError> {
        match self.raw() {
            RawPhase::Ordinary(chunk) => Ok(chunk.entropy),
            RawPhase::Transition(_) => Err(PhaseThermoError::WrongPhaseType {
                property: PhaseProperty::Entropy298,
                phase_kind: PhaseKind::Transition,
            }),
        }
    }

    /// Converts ordinary-phase stored entropy to joules per mole kelvin.
    pub fn entropy_298_j_per_mol_k(self, energy_unit: EnergyUnit) -> Result<f64, PhaseThermoError> {
        energy_unit
            .to_joules(self.entropy_298_raw()?)
            .map_err(PhaseThermoError::from)
    }

    /// Returns the transition enthalpy in its original stored unit.
    pub fn transition_enthalpy_raw(self) -> Result<f64, PhaseThermoError> {
        match self.raw() {
            RawPhase::Transition(chunk) => Ok(chunk.transition_enthalpy),
            RawPhase::Ordinary(_) => Err(PhaseThermoError::WrongPhaseType {
                property: PhaseProperty::TransitionEnthalpy,
                phase_kind: PhaseKind::Ordinary,
            }),
        }
    }

    /// Converts transition enthalpy to joules per mole.
    pub fn transition_enthalpy_j_per_mol(
        self,
        energy_unit: EnergyUnit,
    ) -> Result<f64, PhaseThermoError> {
        energy_unit
            .to_joules(self.transition_enthalpy_raw()?)
            .map_err(PhaseThermoError::from)
    }

    /// Returns the transition temperature in kelvin without unit conversion.
    pub fn transition_temperature_k(self) -> Result<f64, PhaseThermoError> {
        match self.raw() {
            RawPhase::Transition(chunk) => Ok(chunk.transition_temperature),
            RawPhase::Ordinary(_) => Err(PhaseThermoError::WrongPhaseType {
                property: PhaseProperty::TransitionTemperature,
                phase_kind: PhaseKind::Ordinary,
            }),
        }
    }

    /// Returns the transition's preserved parent phase ID.
    pub fn parent_phase_id_raw(self) -> Result<i32, PhaseThermoError> {
        match self.raw() {
            RawPhase::Transition(chunk) => Ok(chunk.parent_phase_id_raw),
            RawPhase::Ordinary(_) => Err(PhaseThermoError::WrongPhaseType {
                property: PhaseProperty::ParentPhaseId,
                phase_kind: PhaseKind::Ordinary,
            }),
        }
    }
}

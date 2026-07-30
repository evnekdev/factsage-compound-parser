//! Read-only thermodynamic decoding and evaluation over the domain model.

mod date;
mod density;
mod error;
mod heat_capacity;
mod units;

pub use date::OleAutomationDate;
pub use density::decode_density;
pub use error::{
    DateError, DensityError, HeatCapacityError, PhaseKind, PhaseProperty, PhaseThermoError,
    UnitError,
};
pub use units::{EnergyUnit, PressureUnit};

use crate::domain::{Compound, Phase, PhaseDefinition};

impl Compound {
    /// Returns the typed energy unit decoded from the raw compound code.
    pub fn energy_unit(&self) -> EnergyUnit {
        EnergyUnit::from_raw(self.raw.unit_energy)
    }

    /// Returns the typed pressure unit decoded from the raw compound code.
    pub fn pressure_unit(&self) -> PressureUnit {
        PressureUnit::from_raw(self.raw.unit_pressure)
    }

    /// Converts an ordinary phase enthalpy using this compound's energy unit.
    pub fn enthalpy_298_j_per_mol(&self, phase_index: usize) -> Result<f64, PhaseThermoError> {
        let phase = self
            .phases
            .get(phase_index)
            .ok_or(PhaseThermoError::InvalidPhaseIndex { phase_index })?;
        phase.enthalpy_298_j_per_mol(self.energy_unit())
    }

    /// Converts an ordinary phase entropy using this compound's energy unit.
    pub fn entropy_298_j_per_mol_k(&self, phase_index: usize) -> Result<f64, PhaseThermoError> {
        let phase = self
            .phases
            .get(phase_index)
            .ok_or(PhaseThermoError::InvalidPhaseIndex { phase_index })?;
        phase.entropy_298_j_per_mol_k(self.energy_unit())
    }

    /// Converts a transition enthalpy using this compound's energy unit.
    pub fn transition_enthalpy_j_per_mol(
        &self,
        phase_index: usize,
    ) -> Result<f64, PhaseThermoError> {
        let phase = self
            .phases
            .get(phase_index)
            .ok_or(PhaseThermoError::InvalidPhaseIndex { phase_index })?;
        phase.transition_enthalpy_j_per_mol(self.energy_unit())
    }
}

impl Phase {
    /// Returns the ordinary-phase stored enthalpy at its 298 K anchor.
    pub fn enthalpy_298_raw(&self) -> Result<f64, PhaseThermoError> {
        match self.definition {
            PhaseDefinition::Ordinary {
                enthalpy_298_raw, ..
            } => Ok(enthalpy_298_raw),
            PhaseDefinition::Transition { .. } => Err(PhaseThermoError::WrongPhaseType {
                property: PhaseProperty::Enthalpy298,
                phase_kind: PhaseKind::Transition,
            }),
        }
    }

    /// Converts ordinary-phase stored enthalpy to joules per mole.
    pub fn enthalpy_298_j_per_mol(&self, energy_unit: EnergyUnit) -> Result<f64, PhaseThermoError> {
        energy_unit
            .to_joules(self.enthalpy_298_raw()?)
            .map_err(PhaseThermoError::from)
    }

    /// Returns the ordinary-phase stored entropy at its 298 K anchor.
    pub fn entropy_298_raw(&self) -> Result<f64, PhaseThermoError> {
        match self.definition {
            PhaseDefinition::Ordinary {
                entropy_298_raw, ..
            } => Ok(entropy_298_raw),
            PhaseDefinition::Transition { .. } => Err(PhaseThermoError::WrongPhaseType {
                property: PhaseProperty::Entropy298,
                phase_kind: PhaseKind::Transition,
            }),
        }
    }

    /// Converts ordinary-phase stored entropy to joules per mole kelvin.
    pub fn entropy_298_j_per_mol_k(
        &self,
        energy_unit: EnergyUnit,
    ) -> Result<f64, PhaseThermoError> {
        energy_unit
            .to_joules(self.entropy_298_raw()?)
            .map_err(PhaseThermoError::from)
    }

    /// Returns the transition enthalpy in its original stored unit.
    pub fn transition_enthalpy_raw(&self) -> Result<f64, PhaseThermoError> {
        match self.definition {
            PhaseDefinition::Transition {
                transition_enthalpy_raw,
                ..
            } => Ok(transition_enthalpy_raw),
            PhaseDefinition::Ordinary { .. } => Err(PhaseThermoError::WrongPhaseType {
                property: PhaseProperty::TransitionEnthalpy,
                phase_kind: PhaseKind::Ordinary,
            }),
        }
    }

    /// Converts transition enthalpy to joules per mole.
    pub fn transition_enthalpy_j_per_mol(
        &self,
        energy_unit: EnergyUnit,
    ) -> Result<f64, PhaseThermoError> {
        energy_unit
            .to_joules(self.transition_enthalpy_raw()?)
            .map_err(PhaseThermoError::from)
    }

    /// Returns the transition temperature in kelvin without unit conversion.
    pub fn transition_temperature_k(&self) -> Result<f64, PhaseThermoError> {
        match self.definition {
            PhaseDefinition::Transition {
                transition_temperature,
                ..
            } => Ok(transition_temperature),
            PhaseDefinition::Ordinary { .. } => Err(PhaseThermoError::WrongPhaseType {
                property: PhaseProperty::TransitionTemperature,
                phase_kind: PhaseKind::Ordinary,
            }),
        }
    }

    /// Returns the transition's preserved parent phase ID.
    pub fn parent_phase_id_raw(&self) -> Result<i32, PhaseThermoError> {
        match self.definition {
            PhaseDefinition::Transition {
                parent_phase_id_raw,
                ..
            } => Ok(parent_phase_id_raw),
            PhaseDefinition::Ordinary { .. } => Err(PhaseThermoError::WrongPhaseType {
                property: PhaseProperty::ParentPhaseId,
                phase_kind: PhaseKind::Ordinary,
            }),
        }
    }
}

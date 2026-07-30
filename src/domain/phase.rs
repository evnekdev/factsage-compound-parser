use std::borrow::Cow;

use crate::raw::{RawOrdinaryPhaseChunk, RawTransitionPhaseChunk};

use super::{
    range::{HeatCapacityRange, PhysicalPropertyRange},
    text::decode_ascii,
};

/// The phase state inferred from a raw FactSage phase ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseState {
    /// A solid phase.
    Solid,
    /// A liquid phase.
    Liquid,
    /// A gas phase.
    Gas,
    /// An aqueous phase.
    Aqueous,
}

impl PhaseState {
    pub(crate) fn from_raw_id(phase_id_raw: i32) -> Self {
        if phase_id_raw > 990 {
            Self::Aqueous
        } else if phase_id_raw > 900 {
            Self::Gas
        } else if phase_id_raw > 800 {
            Self::Liquid
        } else {
            Self::Solid
        }
    }

    fn compact_prefix(self) -> &'static str {
        match self {
            Self::Solid => "S",
            Self::Liquid => "L",
            Self::Gas => "G",
            Self::Aqueous => "AQ",
        }
    }

    fn chemapp_prefix(self) -> &'static str {
        match self {
            Self::Solid => "s",
            Self::Liquid => "l",
            Self::Gas => "g",
            Self::Aqueous => "aq",
        }
    }
}

/// The raw phase record variant retained by a semantic phase.
#[derive(Debug, Clone, PartialEq)]
pub enum RawPhase {
    /// An ordinary phase record, ID 7.
    Ordinary(RawOrdinaryPhaseChunk),
    /// A transition phase record, ID 8.
    Transition(RawTransitionPhaseChunk),
}

impl RawPhase {
    /// Returns the original phase chunk ID.
    pub const fn id(&self) -> u8 {
        match self {
            Self::Ordinary(_) => 7,
            Self::Transition(_) => 8,
        }
    }

    /// Returns the raw phase identifier.
    pub const fn phase_id_raw(&self) -> i32 {
        match self {
            Self::Ordinary(chunk) => chunk.phase_id_raw,
            Self::Transition(chunk) => chunk.phase_id_raw,
        }
    }
}

/// The raw thermodynamic definition associated with a phase variant.
#[derive(Debug, Clone, PartialEq)]
pub enum PhaseDefinition {
    /// The ordinary phase anchor values.
    Ordinary {
        /// Raw enthalpy at the 298 K anchor.
        enthalpy_298_raw: f64,
        /// Raw entropy at the 298 K anchor.
        entropy_298_raw: f64,
    },
    /// The transition phase values.
    Transition {
        /// Raw transition enthalpy.
        transition_enthalpy_raw: f64,
        /// Raw transition temperature.
        transition_temperature: f64,
        /// Raw parent phase ID.
        parent_phase_id_raw: i32,
    },
}

/// A semantic phase with attached raw range records.
#[derive(Debug, Clone, PartialEq)]
pub struct Phase {
    /// The complete original phase record.
    pub raw: RawPhase,
    /// The variant-specific raw definition values.
    pub definition: PhaseDefinition,
    /// Heat-capacity ranges linked by exact raw phase ID.
    pub heat_capacity_ranges: Vec<HeatCapacityRange>,
    /// Kappa ranges linked by exact raw phase ID.
    pub physical_property_ranges: Vec<PhysicalPropertyRange>,
}

impl Phase {
    pub(crate) fn from_raw(raw: RawPhase) -> Self {
        let definition = match &raw {
            RawPhase::Ordinary(chunk) => PhaseDefinition::Ordinary {
                enthalpy_298_raw: chunk.enthalpy,
                entropy_298_raw: chunk.entropy,
            },
            RawPhase::Transition(chunk) => PhaseDefinition::Transition {
                transition_enthalpy_raw: chunk.transition_enthalpy,
                transition_temperature: chunk.transition_temperature,
                parent_phase_id_raw: chunk.parent_phase_id_raw,
            },
        };

        Self {
            raw,
            definition,
            heat_capacity_ranges: Vec::new(),
            physical_property_ranges: Vec::new(),
        }
    }

    /// Returns the original raw phase ID.
    pub fn phase_id_raw(&self) -> i32 {
        self.raw.phase_id_raw()
    }

    /// Decodes the fixed-width phase name as strict ASCII-compatible text.
    pub fn name(&self) -> Result<Cow<'_, str>, super::TextDecodeError> {
        let bytes = match &self.raw {
            RawPhase::Ordinary(chunk) => &chunk.physical.phase_name,
            RawPhase::Transition(chunk) => &chunk.physical.phase_name,
        };
        decode_ascii(bytes, "phase_name")
    }

    /// Infers the phase state using the FactSage raw-ID thresholds.
    pub fn state(&self) -> PhaseState {
        PhaseState::from_raw_id(self.phase_id_raw())
    }

    /// Calculates the state-local phase index from the raw ID.
    pub fn index(&self) -> i32 {
        let raw_id = self.phase_id_raw();
        match self.state() {
            PhaseState::Aqueous => raw_id - 990,
            PhaseState::Gas => raw_id - 900,
            PhaseState::Liquid => raw_id - 800,
            PhaseState::Solid => raw_id - 100,
        }
    }

    /// Returns the compact uppercase label, such as S1 or L2.
    pub fn compact_label(&self) -> String {
        format!("{}{}", self.state().compact_prefix(), self.index())
    }

    /// Returns the ChemApp-style lowercase label, such as s or aq2.
    pub fn chemapp_label(&self) -> String {
        let suffix = if self.index() == 1 {
            String::new()
        } else {
            self.index().to_string()
        };
        format!("{}{}", self.state().chemapp_prefix(), suffix)
    }

    /// Returns true for a transition phase record.
    pub const fn is_transition(&self) -> bool {
        matches!(self.raw, RawPhase::Transition(_))
    }
}

use super::error::UnitError;

/// The established energy unit codes stored in a compound record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyUnit {
    /// Raw code 0: calorie-based stored values.
    Calories,
    /// Raw code 1: joule-based stored values.
    Joules,
    /// Any raw code not yet established by the format sources.
    Unknown(u32),
}

impl EnergyUnit {
    /// Converts a raw compound code without rejecting unknown values.
    pub const fn from_raw(raw: u32) -> Self {
        match raw {
            0 => Self::Calories,
            1 => Self::Joules,
            other => Self::Unknown(other),
        }
    }

    /// Returns the original raw compound code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Calories => 0,
            Self::Joules => 1,
            Self::Unknown(raw) => raw,
        }
    }

    /// Converts a stored energy quantity to joules using the established 4.184 factor.
    pub fn to_joules(self, value: f64) -> Result<f64, UnitError> {
        if !value.is_finite() {
            return Err(UnitError::NonFiniteValue { value });
        }
        match self {
            Self::Calories => Ok(value * 4.184),
            Self::Joules => Ok(value),
            Self::Unknown(raw) => Err(UnitError::UnknownEnergyUnit { raw }),
        }
    }

    /// Converts a joule quantity into the stored unit.
    pub fn from_joules(self, value: f64) -> Result<f64, UnitError> {
        if !value.is_finite() {
            return Err(UnitError::NonFiniteValue { value });
        }
        match self {
            Self::Calories => Ok(value / 4.184),
            Self::Joules => Ok(value),
            Self::Unknown(raw) => Err(UnitError::UnknownEnergyUnit { raw }),
        }
    }
}

/// The established pressure unit codes observed in compound records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureUnit {
    /// Raw code 0: atmospheres.
    Atmospheres,
    /// Raw code 1: bars.
    Bars,
    /// Any raw code not yet established by the format sources.
    Unknown(u32),
}

impl PressureUnit {
    /// Converts a raw compound code without rejecting unknown values.
    pub const fn from_raw(raw: u32) -> Self {
        match raw {
            0 => Self::Atmospheres,
            1 => Self::Bars,
            other => Self::Unknown(other),
        }
    }

    /// Returns the original raw compound code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Atmospheres => 0,
            Self::Bars => 1,
            Self::Unknown(raw) => raw,
        }
    }

    /// Returns an error for an unknown pressure code when a future conversion requests one.
    pub fn require_known(self) -> Result<Self, UnitError> {
        match self {
            Self::Unknown(raw) => Err(UnitError::UnknownPressureUnit { raw }),
            known => Ok(known),
        }
    }
}

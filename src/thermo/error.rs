use std::fmt;

/// Errors raised while converting stored compound units.
#[derive(Debug, Clone, PartialEq)]
pub enum UnitError {
    /// The raw energy code is not one of the established values.
    UnknownEnergyUnit { raw: u32 },
    /// The raw pressure code is not one of the established values.
    UnknownPressureUnit { raw: u32 },
    /// A conversion input was not finite.
    NonFiniteValue { value: f64 },
}

impl fmt::Display for UnitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownEnergyUnit { raw } => {
                write!(formatter, "unknown energy unit code {raw}")
            }
            Self::UnknownPressureUnit { raw } => {
                write!(formatter, "unknown pressure unit code {raw}")
            }
            Self::NonFiniteValue { value } => write!(formatter, "non-finite unit value {value}"),
        }
    }
}

impl std::error::Error for UnitError {}

/// The phase kind relevant to a thermodynamic accessor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseKind {
    /// An ordinary phase record.
    Ordinary,
    /// A transition phase record.
    Transition,
}

/// A phase property that is restricted to one phase kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseProperty {
    /// Ordinary-phase enthalpy at the stored 298 K anchor.
    Enthalpy298,
    /// Ordinary-phase entropy at the stored 298 K anchor.
    Entropy298,
    /// Transition enthalpy.
    TransitionEnthalpy,
    /// Transition temperature.
    TransitionTemperature,
    /// Transition parent phase ID.
    ParentPhaseId,
}

/// Errors raised by ordinary- or transition-phase accessors.
#[derive(Debug, Clone, PartialEq)]
pub enum PhaseThermoError {
    /// The requested property does not exist for this phase kind.
    WrongPhaseType {
        /// Property requested by the caller.
        property: PhaseProperty,
        /// Actual phase kind.
        phase_kind: PhaseKind,
    },
    /// Unit conversion failed.
    Unit(UnitError),
    /// A compound phase index was not present.
    InvalidPhaseIndex { phase_index: usize },
}

impl fmt::Display for PhaseThermoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongPhaseType {
                property,
                phase_kind,
            } => write!(
                formatter,
                "{property:?} is not available for {phase_kind:?} phase"
            ),
            Self::Unit(error) => write!(formatter, "unit conversion error: {error}"),
            Self::InvalidPhaseIndex { phase_index } => {
                write!(formatter, "compound has no phase at index {phase_index}")
            }
        }
    }
}

impl std::error::Error for PhaseThermoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unit(error) => Some(error),
            _ => None,
        }
    }
}

impl From<UnitError> for PhaseThermoError {
    fn from(error: UnitError) -> Self {
        Self::Unit(error)
    }
}

/// Errors raised while converting an OLE Automation day count.
#[derive(Debug, Clone, PartialEq)]
pub enum DateError {
    /// The raw value is NaN or infinite.
    NonFinite { raw_days: f64 },
    /// The value is outside the established OLE Automation date range.
    OutOfRange { raw_days: f64 },
    /// The target clock representation could not be constructed.
    SystemTimeOverflow { raw_days: f64 },
}

impl fmt::Display for DateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite { raw_days } => write!(formatter, "non-finite OLE date {raw_days}"),
            Self::OutOfRange { raw_days } => write!(formatter, "OLE date out of range: {raw_days}"),
            Self::SystemTimeOverflow { raw_days } => {
                write!(
                    formatter,
                    "OLE date cannot be represented as SystemTime: {raw_days}"
                )
            }
        }
    }
}

impl std::error::Error for DateError {}

/// Errors raised while evaluating a heat-capacity expression.
#[derive(Debug, Clone, PartialEq)]
pub enum HeatCapacityError {
    /// Temperature was NaN or infinite.
    NonFiniteTemperature { temperature_k: f64 },
    /// Evaluation requires a positive absolute temperature.
    NonPositiveTemperature { temperature_k: f64 },
    /// A range's bounds are not finite and strictly ordered.
    InvalidRangeBounds {
        /// Range index when selected from a phase, if known.
        range_index: Option<usize>,
        /// Lower bound.
        t_min: f64,
        /// Upper bound.
        t_max: f64,
    },
    /// The requested temperature is outside one range.
    TemperatureOutsideRange {
        /// Requested temperature.
        temperature_k: f64,
        /// Range lower bound.
        t_min: f64,
        /// Range upper bound.
        t_max: f64,
    },
    /// No phase range contains the requested temperature.
    TemperatureOutsideAllRanges {
        /// Requested temperature.
        temperature_k: f64,
        /// Available intervals in stream order.
        available_ranges: Vec<(f64, f64)>,
    },
    /// More than one phase range contains the requested temperature.
    OverlappingRanges {
        /// Requested temperature.
        temperature_k: f64,
        /// Candidate range indexes in stream order.
        candidate_range_indexes: Vec<usize>,
    },
    /// A coefficient is not finite.
    NonFiniteCoefficient {
        /// Coefficient index.
        coefficient_index: usize,
        /// Raw coefficient.
        value: f64,
    },
    /// A power is not finite.
    NonFinitePower {
        /// Power index.
        power_index: usize,
        /// Raw power.
        value: f64,
    },
    /// A power operation or term multiplication was invalid.
    InvalidPowerEvaluation {
        /// Term index.
        term_index: usize,
        /// Evaluation temperature.
        temperature_k: f64,
        /// Raw exponent.
        power: f64,
    },
    /// An individual term became non-finite.
    NonFiniteTerm {
        /// Term index.
        term_index: usize,
        /// Term value.
        value: f64,
    },
    /// The final raw sum became non-finite.
    NonFiniteResult { value: f64 },
    /// Energy-unit conversion failed.
    Unit(UnitError),
    /// A phase index was not present.
    InvalidPhaseIndex { phase_index: usize },
}

impl fmt::Display for HeatCapacityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteTemperature { temperature_k } => {
                write!(formatter, "non-finite temperature {temperature_k}")
            }
            Self::NonPositiveTemperature { temperature_k } => {
                write!(formatter, "temperature must be positive: {temperature_k}")
            }
            Self::InvalidRangeBounds {
                range_index,
                t_min,
                t_max,
            } => write!(
                formatter,
                "invalid heat-capacity range {range_index:?}: {t_min}..{t_max}"
            ),
            Self::TemperatureOutsideRange {
                temperature_k,
                t_min,
                t_max,
            } => write!(
                formatter,
                "temperature {temperature_k} is outside {t_min}..{t_max}"
            ),
            Self::TemperatureOutsideAllRanges {
                temperature_k,
                available_ranges,
            } => write!(
                formatter,
                "temperature {temperature_k} is outside all available ranges {available_ranges:?}"
            ),
            Self::OverlappingRanges {
                temperature_k,
                candidate_range_indexes,
            } => write!(
                formatter,
                "temperature {temperature_k} is covered by overlapping ranges {candidate_range_indexes:?}"
            ),
            Self::NonFiniteCoefficient {
                coefficient_index,
                value,
            } => write!(
                formatter,
                "non-finite CP coefficient {coefficient_index}: {value}"
            ),
            Self::NonFinitePower { power_index, value } => {
                write!(formatter, "non-finite CP power {power_index}: {value}")
            }
            Self::InvalidPowerEvaluation {
                term_index,
                temperature_k,
                power,
            } => write!(
                formatter,
                "invalid CP power evaluation for term {term_index}: {temperature_k}^{power}"
            ),
            Self::NonFiniteTerm { term_index, value } => {
                write!(formatter, "non-finite CP term {term_index}: {value}")
            }
            Self::NonFiniteResult { value } => write!(formatter, "non-finite CP result {value}"),
            Self::Unit(error) => write!(formatter, "unit conversion error: {error}"),
            Self::InvalidPhaseIndex { phase_index } => {
                write!(formatter, "compound has no phase at index {phase_index}")
            }
        }
    }
}

impl std::error::Error for HeatCapacityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unit(error) => Some(error),
            _ => None,
        }
    }
}

impl From<UnitError> for HeatCapacityError {
    fn from(error: UnitError) -> Self {
        Self::Unit(error)
    }
}

/// Errors raised while decoding the packed phase density field.
#[derive(Debug, Clone, PartialEq)]
pub enum DensityError {
    /// The raw density is NaN or infinite.
    NonFinite { raw: f64 },
}

impl fmt::Display for DensityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite { raw } => write!(formatter, "non-finite density value {raw}"),
        }
    }
}

impl std::error::Error for DensityError {}

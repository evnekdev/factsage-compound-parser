use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::domain::{Compound, HeatCapacityRange, Phase, PhysicalPropertyRange, RawPhase};
use crate::raw::{
    RawCommentChunk, RawCommonHeader, RawCompoundChunk, RawDatabaseHeaderChunk,
    RawHeatCapacityChunk, RawKappaChunk, RawOrdinaryPhaseChunk, RawTransitionPhaseChunk,
};

use super::error::DateError;

const MILLIS_PER_DAY: i128 = 86_400_000;
const UNIX_EPOCH_OLE_DAYS: i128 = 25_569;
const MIN_OLE_DAYS: f64 = -657_435.0;
const MAX_OLE_DAYS: f64 = 2_958_466.0;

/// A finite OLE Automation day count retained without replacing the raw value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OleAutomationDate {
    /// The original OLE day count.
    pub raw_days: f64,
}

impl OleAutomationDate {
    /// Creates a date from a finite, representable OLE Automation day count.
    pub fn from_raw(raw_days: f64) -> Result<Self, DateError> {
        if !raw_days.is_finite() {
            return Err(DateError::NonFinite { raw_days });
        }
        if !(MIN_OLE_DAYS < raw_days && raw_days < MAX_OLE_DAYS) {
            return Err(DateError::OutOfRange { raw_days });
        }
        Ok(Self { raw_days })
    }

    /// Converts using OLE Automation's 1899-12-30 epoch and negative-date rule.
    ///
    /// As in the established Windows/OLE implementation, negative fractional
    /// values mirror the time portion around midnight. For example, -0.25 is
    /// 06:00 on 1899-12-30 rather than 18:00 on that date.
    pub fn to_system_time(self) -> Result<SystemTime, DateError> {
        let raw_days = self.raw_days;
        if !raw_days.is_finite() {
            return Err(DateError::NonFinite { raw_days });
        }
        if !(MIN_OLE_DAYS < raw_days && raw_days < MAX_OLE_DAYS) {
            return Err(DateError::OutOfRange { raw_days });
        }

        // Match `System.DateTime.FromOADate`: Automation dates are rounded to
        // the nearest millisecond before applying the negative-fraction rule.
        let rounding = if raw_days >= 0.0 { 0.5 } else { -0.5 };
        let mut ole_millis = (raw_days * MILLIS_PER_DAY as f64 + rounding) as i128;
        if ole_millis < 0 {
            ole_millis -= (ole_millis % MILLIS_PER_DAY) * 2;
        }
        let unix_millis = ole_millis - UNIX_EPOCH_OLE_DAYS * MILLIS_PER_DAY;

        if unix_millis >= 0 {
            let millis = u64::try_from(unix_millis)
                .map_err(|_| DateError::SystemTimeOverflow { raw_days })?;
            UNIX_EPOCH
                .checked_add(Duration::from_millis(millis))
                .ok_or(DateError::SystemTimeOverflow { raw_days })
        } else {
            let millis = u64::try_from(-unix_millis)
                .map_err(|_| DateError::SystemTimeOverflow { raw_days })?;
            UNIX_EPOCH
                .checked_sub(Duration::from_millis(millis))
                .ok_or(DateError::SystemTimeOverflow { raw_days })
        }
    }
}

impl RawDatabaseHeaderChunk {
    /// Converts the raw database-header OLE date.
    pub fn ole_date(&self) -> Result<OleAutomationDate, DateError> {
        OleAutomationDate::from_raw(self.date_ole)
    }
}

impl RawCommonHeader {
    /// Converts the raw shared-entry OLE timestamp.
    pub fn ole_timestamp(&self) -> Result<OleAutomationDate, DateError> {
        OleAutomationDate::from_raw(self.timestamp_ole)
    }
}

impl RawCompoundChunk {
    /// Converts the compound's shared-entry OLE timestamp.
    pub fn ole_timestamp(&self) -> Result<OleAutomationDate, DateError> {
        self.header.ole_timestamp()
    }
}

impl RawOrdinaryPhaseChunk {
    /// Converts the ordinary phase's shared-entry OLE timestamp.
    pub fn ole_timestamp(&self) -> Result<OleAutomationDate, DateError> {
        self.header.ole_timestamp()
    }
}

impl RawTransitionPhaseChunk {
    /// Converts the transition phase's shared-entry OLE timestamp.
    pub fn ole_timestamp(&self) -> Result<OleAutomationDate, DateError> {
        self.header.ole_timestamp()
    }
}

impl RawHeatCapacityChunk {
    /// Converts the CP range's shared-entry OLE timestamp.
    pub fn ole_timestamp(&self) -> Result<OleAutomationDate, DateError> {
        self.header.ole_timestamp()
    }
}

impl RawCommentChunk {
    /// Converts the comment's shared-entry OLE timestamp.
    pub fn ole_timestamp(&self) -> Result<OleAutomationDate, DateError> {
        self.header.ole_timestamp()
    }
}

impl RawKappaChunk {
    /// Converts the kappa record's shared-entry OLE timestamp.
    pub fn ole_timestamp(&self) -> Result<OleAutomationDate, DateError> {
        self.header.ole_timestamp()
    }
}

impl Compound {
    /// Converts the compound's shared-entry OLE timestamp.
    pub fn ole_timestamp(&self) -> Result<OleAutomationDate, DateError> {
        self.raw.ole_timestamp()
    }
}

impl Phase {
    /// Converts the phase's shared-entry OLE timestamp.
    pub fn ole_timestamp(&self) -> Result<OleAutomationDate, DateError> {
        match &self.raw {
            RawPhase::Ordinary(chunk) => chunk.ole_timestamp(),
            RawPhase::Transition(chunk) => chunk.ole_timestamp(),
        }
    }
}

impl HeatCapacityRange {
    /// Converts the CP range's shared-entry OLE timestamp.
    pub fn ole_timestamp(&self) -> Result<OleAutomationDate, DateError> {
        self.raw.ole_timestamp()
    }
}

impl PhysicalPropertyRange {
    /// Converts the kappa record's shared-entry OLE timestamp.
    pub fn ole_timestamp(&self) -> Result<OleAutomationDate, DateError> {
        self.raw.ole_timestamp()
    }
}

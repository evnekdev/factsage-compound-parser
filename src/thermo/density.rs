use crate::domain::Phase;
use crate::domain::RawPhase;

use super::error::DensityError;

const DENSITY_REMAINDER_MODULUS: f64 = 1_000_000.0;

/// Decodes the established provisional density remainder rule.
pub fn decode_density(raw: f64) -> Result<f64, DensityError> {
    if !raw.is_finite() {
        return Err(DensityError::NonFinite { raw });
    }
    Ok(raw % DENSITY_REMAINDER_MODULUS)
}

impl Phase {
    /// Returns the original packed density value.
    pub fn density_raw(&self) -> f64 {
        match &self.raw {
            RawPhase::Ordinary(chunk) => chunk.physical.density_raw,
            RawPhase::Transition(chunk) => chunk.physical.density_raw,
        }
    }

    /// Returns the provisional Python-compatible density remainder.
    ///
    /// The result has no asserted physical unit. The upper encoded portion
    /// and the meaning of the packed field remain reverse-engineered.
    pub fn density(&self) -> Result<f64, DensityError> {
        decode_density(self.density_raw())
    }
}

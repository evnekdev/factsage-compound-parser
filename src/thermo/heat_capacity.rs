use crate::domain::{Compound, HeatCapacityRange, Phase};

use super::error::HeatCapacityError;
use super::units::EnergyUnit;

impl HeatCapacityRange {
    /// Returns the raw lower temperature bound.
    pub const fn temperature_min(&self) -> f64 {
        self.raw.temperature_min
    }

    /// Returns the raw upper temperature bound.
    pub const fn temperature_max(&self) -> f64 {
        self.raw.temperature_max
    }

    /// Returns the original eight CP coefficients.
    pub const fn coefficients(&self) -> &[f64; 8] {
        &self.raw.coefficients
    }

    /// Returns the original eight CP powers.
    pub const fn powers(&self) -> &[f64; 8] {
        &self.raw.powers
    }

    /// Returns whether a finite temperature is within inclusive range bounds.
    pub fn contains_temperature(&self, temperature_k: f64) -> bool {
        temperature_k.is_finite()
            && self.temperature_min() <= temperature_k
            && temperature_k <= self.temperature_max()
    }

    /// Returns the stored CP-range enthalpy without reinterpretation.
    pub const fn stored_enthalpy_raw(&self) -> f64 {
        self.raw.enthalpy
    }

    /// Returns the stored CP-range entropy without reinterpretation.
    pub const fn stored_entropy_raw(&self) -> f64 {
        self.raw.entropy
    }

    /// Converts the stored CP-range enthalpy to joules per mole.
    pub fn stored_enthalpy_j_per_mol(
        &self,
        energy_unit: EnergyUnit,
    ) -> Result<f64, super::UnitError> {
        energy_unit.to_joules(self.stored_enthalpy_raw())
    }

    /// Converts the stored CP-range entropy to joules per mole kelvin.
    pub fn stored_entropy_j_per_mol_k(
        &self,
        energy_unit: EnergyUnit,
    ) -> Result<f64, super::UnitError> {
        energy_unit.to_joules(self.stored_entropy_raw())
    }

    /// Evaluates the eight-term stored CP expression at a temperature in kelvin.
    pub fn heat_capacity_raw(&self, temperature_k: f64) -> Result<f64, HeatCapacityError> {
        validate_temperature(temperature_k)?;
        validate_bounds(self, None)?;
        if !self.contains_temperature(temperature_k) {
            return Err(HeatCapacityError::TemperatureOutsideRange {
                temperature_k,
                t_min: self.temperature_min(),
                t_max: self.temperature_max(),
            });
        }
        evaluate_expression(self, temperature_k)
    }

    /// Evaluates CP and converts the result to joules per mole kelvin.
    pub fn heat_capacity_j_per_mol_k(
        &self,
        temperature_k: f64,
        energy_unit: EnergyUnit,
    ) -> Result<f64, HeatCapacityError> {
        let raw = self.heat_capacity_raw(temperature_k)?;
        energy_unit.to_joules(raw).map_err(HeatCapacityError::from)
    }
}

impl Phase {
    /// Selects and evaluates the applicable CP range at a temperature.
    ///
    /// Individual range bounds are closed. At an exact endpoint shared by two
    /// adjacent ranges, the lower-temperature range wins. This matches the
    /// observed CDB topology while still reporting genuine interval overlaps.
    /// Ranges are searched in preserved stream order and need not be sorted.
    pub fn heat_capacity_at(
        &self,
        temperature_k: f64,
        energy_unit: EnergyUnit,
    ) -> Result<f64, HeatCapacityError> {
        validate_temperature(temperature_k)?;

        let mut available_ranges = Vec::with_capacity(self.heat_capacity_ranges.len());
        let mut candidates = Vec::new();
        for (range_index, range) in self.heat_capacity_ranges.iter().enumerate() {
            validate_bounds(range, Some(range_index))?;
            available_ranges.push((range.temperature_min(), range.temperature_max()));
            if range.contains_temperature(temperature_k) {
                candidates.push(range_index);
            }
        }

        let range_index = match candidates.as_slice() {
            [] => {
                return Err(HeatCapacityError::TemperatureOutsideAllRanges {
                    temperature_k,
                    available_ranges,
                });
            }
            [range_index] => *range_index,
            _ => select_shared_endpoint(&self.heat_capacity_ranges, temperature_k, &candidates)
                .ok_or(HeatCapacityError::OverlappingRanges {
                    temperature_k,
                    candidate_range_indexes: candidates,
                })?,
        };

        self.heat_capacity_ranges[range_index].heat_capacity_j_per_mol_k(temperature_k, energy_unit)
    }
}

impl Compound {
    /// Evaluates CP for a phase index using this compound's energy unit.
    pub fn heat_capacity_at(
        &self,
        phase_index: usize,
        temperature_k: f64,
    ) -> Result<f64, HeatCapacityError> {
        let phase = self
            .phases
            .get(phase_index)
            .ok_or(HeatCapacityError::InvalidPhaseIndex { phase_index })?;
        phase.heat_capacity_at(temperature_k, self.energy_unit())
    }
}

fn select_shared_endpoint(
    ranges: &[HeatCapacityRange],
    temperature_k: f64,
    candidates: &[usize],
) -> Option<usize> {
    if candidates.len() != 2 {
        return None;
    }

    let first = candidates[0];
    let second = candidates[1];
    let first_range = &ranges[first];
    let second_range = &ranges[second];

    let first_ends = first_range.temperature_max() == temperature_k
        && first_range.temperature_min() < temperature_k;
    let second_starts = second_range.temperature_min() == temperature_k
        && temperature_k < second_range.temperature_max();
    if first_ends && second_starts {
        return Some(first);
    }

    let second_ends = second_range.temperature_max() == temperature_k
        && second_range.temperature_min() < temperature_k;
    let first_starts = first_range.temperature_min() == temperature_k
        && temperature_k < first_range.temperature_max();
    if second_ends && first_starts {
        return Some(second);
    }

    None
}

fn validate_temperature(temperature_k: f64) -> Result<(), HeatCapacityError> {
    if !temperature_k.is_finite() {
        return Err(HeatCapacityError::NonFiniteTemperature { temperature_k });
    }
    if temperature_k <= 0.0 {
        return Err(HeatCapacityError::NonPositiveTemperature { temperature_k });
    }
    Ok(())
}

fn validate_bounds(
    range: &HeatCapacityRange,
    range_index: Option<usize>,
) -> Result<(), HeatCapacityError> {
    if !range.temperature_min().is_finite()
        || !range.temperature_max().is_finite()
        || range.temperature_min() > range.temperature_max()
    {
        return Err(HeatCapacityError::InvalidRangeBounds {
            range_index,
            t_min: range.temperature_min(),
            t_max: range.temperature_max(),
        });
    }
    Ok(())
}

fn evaluate_expression(
    range: &HeatCapacityRange,
    temperature_k: f64,
) -> Result<f64, HeatCapacityError> {
    let mut sum = 0.0;
    for index in 0..8 {
        let coefficient = range.coefficients()[index];
        let power = range.powers()[index];
        if !coefficient.is_finite() {
            return Err(HeatCapacityError::NonFiniteCoefficient {
                coefficient_index: index,
                value: coefficient,
            });
        }
        if !power.is_finite() {
            return Err(HeatCapacityError::NonFinitePower {
                power_index: index,
                value: power,
            });
        }

        let powered = temperature_k.powf(power);
        if !powered.is_finite() {
            return Err(HeatCapacityError::InvalidPowerEvaluation {
                term_index: index,
                temperature_k,
                power,
            });
        }
        let term = coefficient * powered;
        if !term.is_finite() {
            return Err(HeatCapacityError::NonFiniteTerm {
                term_index: index,
                value: term,
            });
        }
        sum += term;
        if !sum.is_finite() {
            return Err(HeatCapacityError::NonFiniteResult { value: sum });
        }
    }
    Ok(sum)
}

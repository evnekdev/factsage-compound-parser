//! Completeness guardrails for CP-backed ordinary FDB Gibbs functions.
//!
//! The binary layout always contains fixed physical-tail slots. This module
//! distinguishes the established all-zero inactive representation from
//! materially nonzero data without inventing magnetic, pressure-volume, or
//! ID-11 equations.

use crate::domain::{PhaseView, PhysicalPropertyRangeView, RawPhase};

/// Why an ordinary FDB phase cannot yet be represented completely by its
/// validated H/S/Cp ranges.
///
/// Exact numeric zero (including negative zero) is the established inactive
/// pattern for fixed-width physical-tail slots. Nonzero data is never ignored.
/// Linked ID-11 bounds and powers are structural metadata, not contributions by
/// themselves; a record is active only when one of its coefficient arrays is
/// nonzero. When multiple blockers exist, the view reports non-finite/unknown
/// evidence first, then magnetic, ID-11, and pressure-volume data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdinaryFdbEffectiveGEligibility {
    /// All established extra-contribution fields are inactive, so H/S/Cp is the
    /// complete ordinary-phase model currently established by this parser.
    PureCpBacked,
    /// At least one magnetic field is materially active; no magnetic equation
    /// is inferred by this crate.
    RequiresMagneticSemantics {
        /// Whether the magnetic-temperature slot is nonzero.
        magnetic_temperature_active: bool,
        /// Whether the magnetic-moment slot is nonzero.
        magnetic_moment_active: bool,
        /// Whether the p-factor slot is nonzero.
        p_factor_active: bool,
    },
    /// One or more linked ID-11 records carry a nonzero preserved coefficient;
    /// their physical equation remains unsupported.
    RequiresExtendedPropertySemantics {
        /// Number of linked ID-11 records.
        linked_record_count: usize,
    },
    /// Pressure-volume data is materially active, but this temperature-only API
    /// does not invent a pressure or volume correction.
    RequiresPressureSemantics {
        /// Whether the packed density/volume-related slot is nonzero.
        density_active: bool,
        /// Whether any thermal-expansion coefficient is nonzero.
        thermal_expansion_active: bool,
        /// Whether any compressibility coefficient is nonzero.
        compressibility_active: bool,
        /// Whether any bulk-modulus derivative coefficient is nonzero.
        bulk_modulus_derivative_active: bool,
    },
    /// A structurally present value cannot safely be classified as active or
    /// inactive with current provider evidence.
    PendingProviderEvidence {
        /// Stable field/category name; diagnostic only, never scientific identity.
        field: &'static str,
    },
}

pub(super) fn ordinary_effective_g_eligibility(
    phase: PhaseView<'_>,
) -> OrdinaryFdbEffectiveGEligibility {
    let RawPhase::Ordinary(chunk) = phase.raw() else {
        unreachable!("ordinary thermodynamic view always contains ID-7")
    };
    let physical = &chunk.physical;

    let scalar_fields = [
        ("density_raw", physical.density_raw),
        (
            "magnetic_temperature",
            f64::from(physical.magnetic_temperature),
        ),
        ("magnetic_moment", f64::from(physical.magnetic_moment)),
        ("p_factor", f64::from(physical.p_factor)),
    ];
    for (field, value) in scalar_fields {
        if !value.is_finite() {
            return OrdinaryFdbEffectiveGEligibility::PendingProviderEvidence { field };
        }
    }
    for (field, values) in [
        (
            "thermal_expansion_coefficients",
            physical.thermal_expansion_coefficients.as_slice(),
        ),
        (
            "compressibility_coefficients",
            physical.compressibility_coefficients.as_slice(),
        ),
        (
            "bulk_modulus_derivative_coefficients",
            physical.bulk_modulus_derivative_coefficients.as_slice(),
        ),
    ] {
        if values.iter().any(|value| !value.is_finite()) {
            return OrdinaryFdbEffectiveGEligibility::PendingProviderEvidence { field };
        }
    }

    let magnetic_temperature_active = physical.magnetic_temperature != 0.0;
    let magnetic_moment_active = physical.magnetic_moment != 0.0;
    let p_factor_active = physical.p_factor != 0.0;
    if magnetic_temperature_active || magnetic_moment_active || p_factor_active {
        return OrdinaryFdbEffectiveGEligibility::RequiresMagneticSemantics {
            magnetic_temperature_active,
            magnetic_moment_active,
            p_factor_active,
        };
    }

    let mut linked_record_count = 0;
    let mut extended_active = false;
    for range in phase.physical_property_ranges() {
        linked_record_count += 1;
        match kappa_data_status(range) {
            KappaDataStatus::NonFinite => {
                return OrdinaryFdbEffectiveGEligibility::PendingProviderEvidence {
                    field: "non-finite ID-11 field",
                };
            }
            KappaDataStatus::Active => extended_active = true,
            KappaDataStatus::Inactive => {}
        }
    }
    if extended_active {
        return OrdinaryFdbEffectiveGEligibility::RequiresExtendedPropertySemantics {
            linked_record_count,
        };
    }
    let density_active = physical.density_raw != 0.0;
    let thermal_expansion_active = physical
        .thermal_expansion_coefficients
        .iter()
        .any(|value| *value != 0.0);
    let compressibility_active = physical
        .compressibility_coefficients
        .iter()
        .any(|value| *value != 0.0);
    let bulk_modulus_derivative_active = physical
        .bulk_modulus_derivative_coefficients
        .iter()
        .any(|value| *value != 0.0);
    if density_active
        || thermal_expansion_active
        || compressibility_active
        || bulk_modulus_derivative_active
    {
        return OrdinaryFdbEffectiveGEligibility::RequiresPressureSemantics {
            density_active,
            thermal_expansion_active,
            compressibility_active,
            bulk_modulus_derivative_active,
        };
    }

    OrdinaryFdbEffectiveGEligibility::PureCpBacked
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KappaDataStatus {
    Inactive,
    Active,
    NonFinite,
}

fn kappa_data_status(range: PhysicalPropertyRangeView<'_>) -> KappaDataStatus {
    let raw = range.raw();
    for value in [raw.temperature_min, raw.temperature_max]
        .into_iter()
        .chain(raw.f1_temperature_powers.map(f64::from))
        .chain(raw.f2_pressure_powers.map(f64::from))
        .chain(raw.f3_temperature_powers.map(f64::from))
    {
        if !value.is_finite() {
            return KappaDataStatus::NonFinite;
        }
    }

    let any_active_coefficient = raw
        .f1_temperature_coefficients
        .iter()
        .chain(&raw.f2_pressure_coefficients)
        .chain(&raw.f3_temperature_coefficients)
        .any(|value| !value.is_finite() || *value != 0.0);
    if any_active_coefficient {
        if raw
            .f1_temperature_coefficients
            .iter()
            .chain(&raw.f2_pressure_coefficients)
            .chain(&raw.f3_temperature_coefficients)
            .any(|value| !value.is_finite())
        {
            return KappaDataStatus::NonFinite;
        }
        KappaDataStatus::Active
    } else {
        KappaDataStatus::Inactive
    }
}

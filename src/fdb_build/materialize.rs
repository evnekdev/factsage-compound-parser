//! Bounded FreshModern FDB construction from caller supplied, versioned native templates.

use std::collections::BTreeMap;
use std::str::FromStr;

use chemformula::Formula;

use super::{
    FdbAddedContribution, FdbBuildError, FdbBuildPlan, FdbConstructionProfile, FdbFormulaGroupPlan,
    FdbFunctionIdentity, FdbFunctionPlan, FdbThermoRangePlan,
};
use crate::raw::{
    HeatCapacityKind, RawChunk, RawCommonHeader, RawCompoundChunk, RawDatabaseHeaderChunk,
    RawHeatCapacityChunk, RawOrdinaryPhaseChunk,
};
use crate::{
    DatabaseView, DomainIndex, OleAutomationDate, PhaseThermodynamicView, RawDatabase, RawPhase,
};

/// Versioned opaque defaults extracted from a controlled empty FDB and a fresh
/// single-function, one-range FDB of the same FactSage version.
///
/// The caller owns template provenance. Private native records are never embedded
/// in this crate; only established fields are overwritten during construction.
#[derive(Debug, Clone)]
pub struct FdbNativeTemplates {
    database: RawDatabaseHeaderChunk,
    compound: RawCompoundChunk,
    phase: RawOrdinaryPhaseChunk,
    cp: RawHeatCapacityChunk,
    legacy_zero_cp: Option<RawHeatCapacityChunk>,
    group_start_entry: u8,
}

/// Caller-selected build timestamp and optional ordinary densities.
#[derive(Debug, Clone, PartialEq)]
pub struct FdbFreshMaterialization {
    /// OLE Automation timestamp copied into every record of each new group.
    pub timestamp_ole: f64,
    /// Optional plain density keyed by exact `(formula label, function name)`.
    pub ordinary_density: BTreeMap<(String, String), f64>,
}

impl FdbFreshMaterialization {
    /// Creates options for a new build, with no ordinary density overrides.
    pub fn new(timestamp_ole: f64) -> Self {
        Self {
            timestamp_ole,
            ordinary_density: BTreeMap::new(),
        }
    }
}

/// A bounded writer rejection; failures leave no output file behind.
#[derive(Debug)]
pub enum FdbMaterializeError {
    /// The semantic plan is invalid.
    Plan(FdbBuildError),
    /// The supplied local templates do not satisfy the controlled profile.
    Template(&'static str),
    /// The plan requests a feature outside the bounded writer.
    Unsupported {
        /// Object outside the bounded profile.
        object: String,
        /// Unsupported native capability.
        reason: &'static str,
    },
    /// The formula label and exact declared composition or charge disagree.
    Formula {
        /// Exact caller label that failed validation.
        label: String,
        /// Mismatch or unsupported composition shape.
        reason: String,
    },
    /// Internal serialization or domain verification failed.
    Verification(String),
}

impl std::fmt::Display for FdbMaterializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for FdbMaterializeError {}

fn has_nonzero_cp(range: &FdbThermoRangePlan) -> bool {
    let mut by_power = BTreeMap::<u64, f64>::new();
    for term in &range.cp_terms {
        let power = if term.power == 0.0 { 0.0 } else { term.power };
        *by_power.entry(power.to_bits()).or_default() += term.coefficient;
    }
    by_power.values().any(|coefficient| *coefficient != 0.0)
}

impl FdbNativeTemplates {
    /// Extracts only the four needed record kinds from controlled native examples.
    /// The empty source must contain only ID-9; the exemplar must contain
    /// ID-9, ID-1, ID-7, ID-2 in that order and have consecutive entry numbers.
    pub fn from_examples(
        empty: &RawDatabase,
        one_range: &RawDatabase,
    ) -> Result<Self, FdbMaterializeError> {
        let [RawChunk::DatabaseHeader(database)] = empty.chunks() else {
            return Err(FdbMaterializeError::Template(
                "empty template must contain only ID-9",
            ));
        };
        let [
            RawChunk::DatabaseHeader(exemplar_database),
            RawChunk::Compound(compound),
            RawChunk::PhaseOrdinary(phase),
            RawChunk::HeatCapacity {
                kind: HeatCapacityKind::Id2,
                chunk: cp,
            },
        ] = one_range.chunks()
        else {
            return Err(FdbMaterializeError::Template(
                "range exemplar must contain ID-9, ID-1, ID-7, ID-2",
            ));
        };
        if exemplar_database != database || database.comment.iter().any(|byte| *byte != 0) {
            return Err(FdbMaterializeError::Template(
                "bounded fresh templates must share an unchanged, empty-comment ID-9 header",
            ));
        }
        let start = compound.header.entry_number;
        if start > u8::MAX - 2
            || phase.header.entry_number != start + 1
            || cp.header.entry_number != start + 2
            || phase.phase_id_raw != 101
            || phase.phase_id_raw_neg != -101
            || cp.phase_id_raw != 101
        {
            return Err(FdbMaterializeError::Template(
                "exemplar does not establish the bounded entry/phase sequence",
            ));
        }
        if compound.compound_name != [b' '; 40]
            || compound.formula_name[39] != b' '
            || phase.physical.phase_name[39] != b' '
            || database.comment[79] != 0
        {
            return Err(FdbMaterializeError::Template(
                "template text padding differs from the bounded fresh profile",
            ));
        }
        Ok(Self {
            database: database.clone(),
            compound: compound.clone(),
            phase: phase.clone(),
            cp: cp.clone(),
            legacy_zero_cp: None,
            group_start_entry: start,
        })
    }

    /// Adds the native ID-5 shape observed in a translated FDB of the same
    /// FactSage version. Only its zero-Cp power slots and opaque defaults are
    /// used; identity, bounds, H/S, and timestamps are filled from the plan.
    pub fn with_legacy_id5_example(
        mut self,
        example: &RawDatabase,
    ) -> Result<Self, FdbMaterializeError> {
        let mut found = None::<RawHeatCapacityChunk>;
        for chunk in example.chunks() {
            let RawChunk::HeatCapacity {
                kind: HeatCapacityKind::Id5,
                chunk,
            } = chunk
            else {
                continue;
            };
            if chunk.coefficients.iter().any(|value| *value != 0.0)
                || chunk.unknown_1 != [0; 4]
                || chunk.padding_remaining != [0; 56]
            {
                return Err(FdbMaterializeError::Template(
                    "legacy ID-5 example has unexpected coefficients or opaque bytes",
                ));
            }
            if let Some(previous) = &found {
                if chunk.powers.map(f64::to_bits) != previous.powers.map(f64::to_bits) {
                    return Err(FdbMaterializeError::Template(
                        "legacy ID-5 example has multiple power-slot patterns",
                    ));
                }
            } else {
                found = Some(chunk.clone());
            }
        }
        self.legacy_zero_cp = Some(found.ok_or(FdbMaterializeError::Template(
            "legacy ID-5 example contains no ID-5 record",
        ))?);
        Ok(self)
    }

    /// Returns the observed group-start entry number without revealing template bytes.
    pub const fn group_start_entry(&self) -> u8 {
        self.group_start_entry
    }

    /// Returns the creation date carried by the controlled empty FDB header.
    pub const fn database_date_ole(&self) -> f64 {
        self.database.date_ole
    }
}

impl FdbBuildPlan {
    /// Emits the evidenced translated subset with ordinary ID-2 ranges and
    /// physically omitted, explicitly zero A companions.
    ///
    /// This accepts an already reduced target plan. It does not translate raw
    /// Legacy G fields or construct matching SLN references.
    pub fn materialize_legacy_zero_added(
        &self,
        templates: &FdbNativeTemplates,
        options: &FdbFreshMaterialization,
    ) -> Result<RawDatabase, FdbMaterializeError> {
        self.legacy_zero_added_physical_plan()?
            .materialize_bounded(templates, options, true)
    }

    pub(super) fn legacy_zero_added_physical_plan(
        &self,
    ) -> Result<FdbBuildPlan, FdbMaterializeError> {
        self.validate().map_err(FdbMaterializeError::Plan)?;
        if self.profile() != FdbConstructionProfile::LegacyTranslation {
            return Err(FdbMaterializeError::Unsupported {
                object: "database".into(),
                reason: "zero-added translated emitter requires LegacyTranslation",
            });
        }
        let mut physical_groups = Vec::with_capacity(self.groups().len());
        for group in self.groups() {
            let mut physical = group.clone();
            physical.functions.clear();
            for function in &group.functions {
                match function {
                    FdbFunctionPlan::Ordinary(ordinary) => {
                        if ordinary.ranges.is_empty() {
                            return Err(FdbMaterializeError::Unsupported {
                                object: ordinary.identity.target_name.clone(),
                                reason: "translated empty Cp needs a separate omitted-base policy",
                            });
                        }
                        let first_has_cp = has_nonzero_cp(&ordinary.ranges[0]);
                        if ordinary
                            .ranges
                            .iter()
                            .any(|range| has_nonzero_cp(range) != first_has_cp)
                        {
                            return Err(FdbMaterializeError::Unsupported {
                                object: ordinary.identity.target_name.clone(),
                                reason: "mixed ID-2/ID-5 ranges need a separate thermodynamic-view policy",
                            });
                        }
                        let mut target = ordinary.clone();
                        for range in &mut target.ranges {
                            if !has_nonzero_cp(range) {
                                // The ID-5 exemplar supplies its canonical
                                // zero-Cp power slots, not source padding.
                                range.cp_terms.clear();
                            }
                        }
                        target.identity = FdbFunctionIdentity::fresh_modern(
                            &ordinary.identity.target_name,
                            ordinary.identity.target_state,
                        );
                        physical.functions.push(FdbFunctionPlan::Ordinary(target));
                    }
                    FdbFunctionPlan::Added(added)
                        if added.contribution == FdbAddedContribution::ExplicitZero => {}
                    FdbFunctionPlan::Added(added) => {
                        return Err(FdbMaterializeError::Unsupported {
                            object: added.identity.target_name.clone(),
                            reason: "active translated A needs a native CP-kind and phase policy",
                        });
                    }
                    FdbFunctionPlan::ExplicitZeroOrdinary(identity) => {
                        return Err(FdbMaterializeError::Unsupported {
                            object: identity.target_name.clone(),
                            reason: "zero translated base needs a native omission policy",
                        });
                    }
                }
            }
            physical_groups.push(physical);
        }
        FdbBuildPlan::new_fresh_modern(self.metadata().clone(), physical_groups)
            .map_err(FdbMaterializeError::Plan)
    }

    /// Builds the bounded fresh profile and verifies serialization, strict reparse,
    /// and domain indexing before returning its raw stream.
    pub fn materialize_fresh(
        &self,
        templates: &FdbNativeTemplates,
        options: &FdbFreshMaterialization,
    ) -> Result<RawDatabase, FdbMaterializeError> {
        self.materialize_bounded(templates, options, false)
    }

    fn materialize_bounded(
        &self,
        templates: &FdbNativeTemplates,
        options: &FdbFreshMaterialization,
        legacy_zero_cp: bool,
    ) -> Result<RawDatabase, FdbMaterializeError> {
        self.validate().map_err(FdbMaterializeError::Plan)?;
        if self.profile() != FdbConstructionProfile::FreshModern {
            return Err(FdbMaterializeError::Unsupported {
                object: "database".into(),
                reason: "LegacyTranslation needs its own native omission and A policy",
            });
        }
        OleAutomationDate::from_raw(options.timestamp_ole)
            .map_err(|error| FdbMaterializeError::Verification(error.to_string()))?;
        if !self.metadata().comment.is_empty()
            || self.metadata().date_ole.to_bits() != templates.database.date_ole.to_bits()
        {
            return Err(FdbMaterializeError::Unsupported {
                object: "database header".into(),
                reason: "bounded fresh FDB construction preserves the controlled ID-9 date and empty comment",
            });
        }
        for ((formula, function), density) in &options.ordinary_density {
            if !density.is_finite() || *density <= 0.0 || *density >= 1_000_000.0 {
                return Err(FdbMaterializeError::Unsupported {
                    object: format!("{formula}/{function}"),
                    reason: "plain density must be finite, positive and below the packed-family scale",
                });
            }
            if !self.groups().iter().any(|group| {
                group.formula == *formula
                    && group
                        .functions
                        .iter()
                        .any(|item| item.identity().target_name == *function)
            }) {
                return Err(FdbMaterializeError::Unsupported {
                    object: format!("{formula}/{function}"),
                    reason: "density names no function in this plan",
                });
            }
        }
        let mut chunks = vec![RawChunk::DatabaseHeader(templates.database.clone())];
        for group in self.groups() {
            write_group(&mut chunks, group, templates, options, legacy_zero_cp)?;
        }
        let raw = RawDatabase::from_chunks(chunks);
        let bytes = raw
            .to_bytes()
            .map_err(|error| FdbMaterializeError::Verification(error.to_string()))?;
        let reparsed = RawDatabase::from_bytes(&bytes)
            .map_err(|error| FdbMaterializeError::Verification(error.to_string()))?;
        let index = DomainIndex::build(&reparsed)
            .map_err(|error| FdbMaterializeError::Verification(error.to_string()))?;
        if !index.diagnostics().is_empty() || reparsed != raw {
            return Err(FdbMaterializeError::Verification(
                "constructed stream has domain diagnostics or changed on strict reparse".into(),
            ));
        }
        verify_thermodynamics(self, &reparsed, &index, legacy_zero_cp)?;
        Ok(raw)
    }
}

fn verify_thermodynamics(
    plan: &FdbBuildPlan,
    raw: &RawDatabase,
    index: &DomainIndex,
    legacy_zero_cp: bool,
) -> Result<(), FdbMaterializeError> {
    let view = DatabaseView::new(raw, index)
        .map_err(|error| FdbMaterializeError::Verification(error.to_string()))?;
    if view.compound_count() != plan.groups().len() {
        return Err(FdbMaterializeError::Verification(
            "compound count changed on reparse".into(),
        ));
    }
    for (group, compound) in plan.groups().iter().zip(view.compounds()) {
        let (element_ids, coefficients) = parse_composition(group)?;
        let charge = group.charge.fdb_raw_byte().expect("validated charge") as i8;
        let header = &compound.raw().header;
        if compound
            .formula()
            .map_err(|error| FdbMaterializeError::Verification(error.to_string()))?
            != group.formula
            || header.charge_raw != charge
            || header.element_ids != element_ids
            || header.element_coefficients != coefficients
            || compound.raw().real_stoichiometric_coefficients != coefficients.map(f64::from)
        {
            return Err(FdbMaterializeError::Verification(
                "formula group identity changed on reparse".into(),
            ));
        }
        if compound.phases().count() != group.functions.len() {
            return Err(FdbMaterializeError::Verification(
                "function count changed on reparse".into(),
            ));
        }
        for (phase_index, function) in group.functions.iter().enumerate() {
            let phase_view = compound.phases().nth(phase_index).ok_or_else(|| {
                FdbMaterializeError::Verification("function disappeared on reparse".into())
            })?;
            let RawPhase::Ordinary(parsed_phase) = phase_view.raw() else {
                return Err(FdbMaterializeError::Verification(
                    "function became a transition on reparse".into(),
                ));
            };
            let expected_id = 101 + phase_index as i32;
            let (expected_h, expected_s) = match function {
                FdbFunctionPlan::Ordinary(ordinary) => {
                    (ordinary.phase_enthalpy, ordinary.phase_entropy)
                }
                FdbFunctionPlan::ExplicitZeroOrdinary(_) => (0.0, 0.0),
                FdbFunctionPlan::Added(_) => unreachable!("fresh plan validation excludes A"),
            };
            if phase_view
                .name()
                .map_err(|error| FdbMaterializeError::Verification(error.to_string()))?
                != function.identity().target_name
                || parsed_phase.header.charge_raw != charge
                || parsed_phase.header.element_ids != element_ids
                || parsed_phase.header.element_coefficients != coefficients
                || parsed_phase.phase_id_raw != expected_id
                || parsed_phase.phase_id_raw_neg != -expected_id
                || parsed_phase.enthalpy.to_bits() != expected_h.to_bits()
                || parsed_phase.entropy.to_bits() != expected_s.to_bits()
            {
                return Err(FdbMaterializeError::Verification(
                    "function identity or phase anchors changed on reparse".into(),
                ));
            }
            let FdbFunctionPlan::Ordinary(ordinary) = function else {
                continue;
            };
            let PhaseThermodynamicView::Ordinary(phase) = compound
                .fdb_phase_thermodynamic_view(phase_index)
                .map_err(|error| FdbMaterializeError::Verification(error.to_string()))?
            else {
                return Err(FdbMaterializeError::Verification(
                    "ordinary function became a transition".into(),
                ));
            };
            if phase.heat_capacity_ranges().len() != ordinary.ranges.len() {
                return Err(FdbMaterializeError::Verification(
                    "Cp range count changed on reparse".into(),
                ));
            }
            for (source, parsed_range) in ordinary
                .ranges
                .iter()
                .zip(phase_view.heat_capacity_ranges())
            {
                let cp = parsed_range.raw();
                let expected_kind = if legacy_zero_cp && !has_nonzero_cp(source) {
                    HeatCapacityKind::Id5
                } else {
                    HeatCapacityKind::Id2
                };
                if parsed_range.kind() != expected_kind
                    || cp.phase_id_raw != expected_id
                    || cp.header.charge_raw != charge
                    || cp.header.element_ids != element_ids
                    || cp.header.element_coefficients != coefficients
                    || cp.temperature_min.to_bits() != source.temperature_min_k.to_bits()
                    || cp.temperature_max.to_bits() != source.temperature_max_k.to_bits()
                    || cp.enthalpy.to_bits() != source.reference_enthalpy.to_bits()
                    || cp.entropy.to_bits() != source.reference_entropy.to_bits()
                {
                    return Err(FdbMaterializeError::Verification(
                        "Cp identity or anchors changed on reparse".into(),
                    ));
                }
            }
            for (source, parsed) in ordinary.ranges.iter().zip(phase.heat_capacity_ranges()) {
                for temperature in [
                    source.temperature_min_k,
                    source.temperature_min_k / 2.0 + source.temperature_max_k / 2.0,
                    source.temperature_max_k,
                ] {
                    let expected_cp = source
                        .cp_terms
                        .iter()
                        .map(|term| term.coefficient * temperature.powf(term.power))
                        .sum::<f64>();
                    let expected_h = source.reference_enthalpy
                        + super::integrated_at(source, temperature, false)
                            .map_err(FdbMaterializeError::Verification)?;
                    let expected_s = source.reference_entropy
                        + super::integrated_at(source, temperature, true)
                            .map_err(FdbMaterializeError::Verification)?;
                    let expected_g = expected_h - temperature * expected_s;
                    for (actual, expected) in [
                        (parsed.heat_capacity_raw_at(temperature), expected_cp),
                        (parsed.enthalpy_raw_at(temperature), expected_h),
                        (parsed.entropy_raw_at(temperature), expected_s),
                        (parsed.gibbs_energy_raw_at(temperature), expected_g),
                    ] {
                        let actual = actual.map_err(|error| {
                            FdbMaterializeError::Verification(error.to_string())
                        })?;
                        if !actual.is_finite()
                            || !expected.is_finite()
                            || (actual - expected).abs()
                                > crate::thermo::continuity_tolerance(actual, expected)
                        {
                            return Err(FdbMaterializeError::Verification(
                                "thermodynamic function changed on reparse".into(),
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn write_group(
    chunks: &mut Vec<RawChunk>,
    group: &FdbFormulaGroupPlan,
    templates: &FdbNativeTemplates,
    options: &FdbFreshMaterialization,
    legacy_zero_cp: bool,
) -> Result<(), FdbMaterializeError> {
    let (element_ids, coefficients) = parse_composition(group)?;
    let charge = group.charge.fdb_raw_byte().expect("validated charge") as i8;
    let mut entry = templates.group_start_entry as usize;
    let total_records = 1
        + group.functions.len()
        + group
            .functions
            .iter()
            .map(|function| match function {
                FdbFunctionPlan::Ordinary(ordinary) => ordinary.ranges.len(),
                _ => 0,
            })
            .sum::<usize>();
    if entry + total_records - 1 > u8::MAX as usize {
        return Err(FdbMaterializeError::Unsupported {
            object: group.formula.clone(),
            reason: "entry numbers would roll over",
        });
    }
    let mut compound = templates.compound.clone();
    fill_header(
        &mut compound.header,
        element_ids,
        coefficients,
        charge,
        entry as u8,
        options.timestamp_ole,
    );
    compound.compound_name = [b' '; 40];
    write_ascii(&mut compound.formula_name, &group.formula, b' ');
    compound.unit_energy = group.energy_unit.raw();
    compound.unit_pressure = group.pressure_unit.raw();
    compound.real_stoichiometric_coefficients = coefficients.map(f64::from);
    chunks.push(RawChunk::Compound(compound));
    entry += 1;

    // The parser and native format require every ID-7 before any CP record.
    let mut cp_chunks = Vec::new();
    for (function_index, function) in group.functions.iter().enumerate() {
        let (enthalpy, entropy, ranges) = match function {
            FdbFunctionPlan::Ordinary(value) => (
                value.phase_enthalpy,
                value.phase_entropy,
                value.ranges.as_slice(),
            ),
            FdbFunctionPlan::ExplicitZeroOrdinary(_) => (0.0, 0.0, &[][..]),
            FdbFunctionPlan::Added(_) => unreachable!("fresh plan validation excludes A"),
        };
        let range_limit = if legacy_zero_cp { 9 } else { 3 };
        if ranges.len() > range_limit {
            return Err(FdbMaterializeError::Unsupported {
                object: function.identity().target_name.clone(),
                reason: if legacy_zero_cp {
                    "paired Legacy translation evidence permits at most nine Cp ranges"
                } else {
                    "fresh profile permits at most three Cp ranges"
                },
            });
        }
        let phase_id = 101 + function_index as i32;
        let mut phase = templates.phase.clone();
        fill_header(
            &mut phase.header,
            element_ids,
            coefficients,
            charge,
            entry as u8,
            options.timestamp_ole,
        );
        phase.enthalpy = enthalpy;
        phase.entropy = entropy;
        phase.phase_id_raw = phase_id;
        phase.phase_id_raw_neg = -phase_id;
        write_ascii(
            &mut phase.physical.phase_name,
            &function.identity().target_name,
            b' ',
        );
        phase.physical.density_raw = options
            .ordinary_density
            .get(&(
                group.formula.clone(),
                function.identity().target_name.clone(),
            ))
            .copied()
            .unwrap_or(0.0);
        phase.physical.thermal_expansion_coefficients = [0.0; 4];
        phase.physical.compressibility_coefficients = [0.0; 4];
        phase.physical.bulk_modulus_derivative_coefficients = [0.0; 2];
        phase.physical.magnetic_temperature = 0.0;
        phase.physical.magnetic_moment = 0.0;
        phase.physical.p_factor = 0.0;
        chunks.push(RawChunk::PhaseOrdinary(phase));
        entry += 1;
        for range in ranges {
            let kind = if legacy_zero_cp && !has_nonzero_cp(range) {
                HeatCapacityKind::Id5
            } else {
                HeatCapacityKind::Id2
            };
            let mut cp = match kind {
                HeatCapacityKind::Id5 => {
                    templates
                        .legacy_zero_cp
                        .clone()
                        .ok_or(FdbMaterializeError::Template(
                            "legacy zero-Cp output needs an ID-5 example",
                        ))?
                }
                _ => templates.cp.clone(),
            };
            fill_header(
                &mut cp.header,
                element_ids,
                coefficients,
                charge,
                entry as u8,
                options.timestamp_ole,
            );
            cp.phase_id_raw = phase_id;
            cp.enthalpy = range.reference_enthalpy;
            cp.entropy = range.reference_entropy;
            cp.temperature_min = range.temperature_min_k;
            cp.temperature_max = range.temperature_max_k;
            if kind == HeatCapacityKind::Id2 {
                cp.coefficients = [0.0; 8];
                cp.powers = [0.0; 8];
                for (slot, term) in range.cp_terms.iter().enumerate() {
                    cp.coefficients[slot] = term.coefficient;
                    cp.powers[slot] = term.power;
                }
            }
            cp_chunks.push(RawChunk::HeatCapacity { kind, chunk: cp });
        }
    }
    // Entry numbers follow physical stream order, not the phase/range walk above.
    let cp_start = templates.group_start_entry as usize + 1 + group.functions.len();
    for (offset, chunk) in cp_chunks.iter_mut().enumerate() {
        if let RawChunk::HeatCapacity { chunk, .. } = chunk {
            chunk.header.entry_number = (cp_start + offset) as u8;
        }
    }
    chunks.extend(cp_chunks);
    Ok(())
}

fn parse_composition(
    group: &FdbFormulaGroupPlan,
) -> Result<([u8; 7], [u8; 7]), FdbMaterializeError> {
    let parsed =
        Formula::from_str(&group.formula).map_err(|reason| FdbMaterializeError::Formula {
            label: group.formula.clone(),
            reason,
        })?;
    if parsed.charge != f64::from(group.charge.value())
        || parsed.pairs.len() != group.elements.len()
    {
        return Err(FdbMaterializeError::Formula {
            label: group.formula.clone(),
            reason: "parsed charge or element set differs from the plan".into(),
        });
    }
    let mut ids = [0; 7];
    let mut coefficients = [0; 7];
    // In controlled FactSage 7.3 probes, hydrogen-first FDB Function groups
    // opened without a visible function. Hydrogen-last groups were visible;
    // the admitted native FDB hydrogen groups also put hydrogen last.
    let ordered = parsed
        .pairs
        .iter()
        .filter(|(element, _)| element.index() != 1)
        .chain(
            parsed
                .pairs
                .iter()
                .filter(|(element, _)| element.index() == 1),
        );
    for (slot, (element, amount)) in ordered.enumerate() {
        let symbol = format!("{element:?}");
        let declared = group
            .elements
            .iter()
            .find(|item| item.symbol == symbol)
            .ok_or_else(|| FdbMaterializeError::Formula {
                label: group.formula.clone(),
                reason: "parsed element differs from declared composition".into(),
            })?;
        let id = element.index();
        if id == 0
            || id > u8::MAX as usize
            || !amount.is_finite()
            || *amount < 1.0
            || *amount > u8::MAX as f64
            || amount.fract() != 0.0
            || (declared.amount.numerator as u128)
                != (*amount as u128) * u128::from(declared.amount.denominator)
        {
            return Err(FdbMaterializeError::Formula {
                label: group.formula.clone(),
                reason:
                    "only real elements with exact positive u8 integer coefficients are admitted"
                        .into(),
            });
        }
        ids[slot] = id as u8;
        coefficients[slot] = *amount as u8;
    }
    Ok((ids, coefficients))
}

fn fill_header(
    header: &mut RawCommonHeader,
    ids: [u8; 7],
    coefficients: [u8; 7],
    charge: i8,
    entry: u8,
    timestamp_ole: f64,
) {
    header.element_ids = ids;
    header.element_coefficients = coefficients;
    header.charge_raw = charge;
    header.entry_number = entry;
    header.reference = [0; 2];
    header.timestamp_ole = timestamp_ole;
}

fn write_ascii<const N: usize>(target: &mut [u8; N], value: &str, padding: u8) {
    target.fill(padding);
    target[..value.len()].copy_from_slice(value.as_bytes());
}

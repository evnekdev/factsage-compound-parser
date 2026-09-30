//! Actionable FDB-C3 blockers. These are not semantic-plan validation errors.

use super::{FdbAddedContribution, FdbBuildPlan, FdbFunctionPlan};

/// Provider-neutral blocker category, translatable to application reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdbBlockerClass {
    /// The rule is known; implementation remains.
    Engineering,
    /// An owning provider API is absent.
    ProviderApi,
    /// Native field encoding or fresh-output convention is unproved.
    NativeFormat,
    /// A required scientific transform is unproved.
    ScientificSemantics,
    /// More than one valid encoding needs a declared product policy.
    UserPolicyDecision,
    /// A believed rule still lacks a controlled fixture.
    MissingTestEvidence,
    /// Independent output verification is unavailable.
    Verification,
}

/// One actionable native-construction or verification blocker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FdbBuildBlocker {
    /// Exact plan object or database-level capability.
    pub object: String,
    /// Native field or provider capability.
    pub field: &'static str,
    /// Blocker category.
    pub class: FdbBlockerClass,
    /// Why the missing rule prevents construction or verification.
    pub reason: &'static str,
    /// Established fact that narrows the question.
    pub known: &'static str,
    /// Exact missing rule, encoding, or implementation.
    pub missing: &'static str,
    /// Smallest controlled artifact or action that could settle the issue.
    pub exact_evidence: Option<&'static str>,
    /// Whether a user/domain expert could supply that evidence.
    pub user_evidence_can_unblock: bool,
}

#[derive(Clone, Copy)]
struct Gap {
    field: &'static str,
    class: FdbBlockerClass,
    reason: &'static str,
    known: &'static str,
    missing: &'static str,
    exact_evidence: Option<&'static str>,
    user_evidence_can_unblock: bool,
}

impl Gap {
    fn on(self, object: &str) -> FdbBuildBlocker {
        FdbBuildBlocker {
            object: object.into(),
            field: self.field,
            class: self.class,
            reason: self.reason,
            known: self.known,
            missing: self.missing,
            exact_evidence: self.exact_evidence,
            user_evidence_can_unblock: self.user_evidence_can_unblock,
        }
    }
}

const HEADER_GAPS: &[Gap] = &[
    Gap {
        field: "ID-9.read_flag",
        class: FdbBlockerClass::MissingTestEvidence,
        reason: "the observed zero guardrail has not been confirmed for freshly created FDBs",
        known: "zero is compatible with every locally examined FDB but also occurs in CDBs",
        missing: "the fresh FDB read-flag rule under an identified FactSage version",
        exact_evidence: Some(
            "one controlled newly created FDB header with FactSage version and creation action recorded",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "ID-9.padding_1/padding_2/padding_3",
        class: FdbBlockerClass::NativeFormat,
        reason: "fresh header bytes cannot be chosen from the parser's preservation behavior",
        known: "the parser and serializer preserve all three padding fields",
        missing: "the native fresh-output byte pattern for each padding field",
        exact_evidence: Some(
            "one controlled newly created FDB header with FactSage version and creation action recorded",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "ID-9.unknown_1/unknown_2",
        class: FdbBlockerClass::NativeFormat,
        reason: "uninterpreted header fields cannot receive an invented default",
        known: "the parser preserves eleven and twelve bytes respectively",
        missing: "whether these fields are fixed, versioned, or derived for new FDBs",
        exact_evidence: Some(
            "two controlled fresh FDB headers created under the same FactSage version with one changed input",
        ),
        user_evidence_can_unblock: true,
    },
];

const COMMON_GAPS: &[Gap] = &[
    Gap {
        field: "RawCommonHeader.element_ids[7]/element_coefficients[7]",
        class: FdbBlockerClass::NativeFormat,
        reason: "source symbols and exact ratios are not yet mapped to repeated native element slots",
        known: "seven ID and integer-coefficient slots are parsed and preserved in every entry header",
        missing: "element ID dictionary/order, coefficient scaling and unused-slot convention per record kind",
        exact_evidence: Some(
            "paired fresh FDBs with one neutral formula and one changed elemental ratio, inspecting ID-1/ID-7/CP headers",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.coefficient_padding/unknown[2]",
        class: FdbBlockerClass::NativeFormat,
        reason: "uninterpreted shared-header bytes have no evidenced fresh default",
        known: "the parser preserves all three bytes in each entry record",
        missing: "fresh values and whether they vary by record kind or FactSage version",
        exact_evidence: Some(
            "controlled fresh FDB with one group, one ordinary function and one Cp interval",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.charge_raw",
        class: FdbBlockerClass::NativeFormat,
        reason: "semantic group charge is not yet proved to map directly to every repeated signed-byte header",
        known: "the provider parses and serializes a signed i8 charge in every non-database record",
        missing: "charge value/repetition rule for ID-1, ID-7 and CP records, including neutral zero",
        exact_evidence: Some(
            "controlled neutral and charged fresh FDB groups with equal element ratios, comparing shared headers",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.entry_number/reference[2]",
        class: FdbBlockerClass::NativeFormat,
        reason: "entry/reference counters cannot be generated from source order alone",
        known: "one u8 entry number and two u16 references are preserved in each entry header",
        missing: "fresh allocation, linkage, rollover and per-record repetition rules",
        exact_evidence: Some(
            "fresh FDBs with one then two functions in a single group, retaining creation order",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.timestamp_ole",
        class: FdbBlockerClass::NativeFormat,
        reason: "the plan's database date does not establish entry timestamp behavior",
        known: "OLE timestamps are parsed and convertible independently of the database date",
        missing: "whether entries copy the header date, source timestamp or creation time",
        exact_evidence: Some(
            "two controlled fresh FDB creations at distinct times with unchanged function intent",
        ),
        user_evidence_can_unblock: true,
    },
];

const GROUP_GAPS: &[Gap] = &[
    Gap {
        field: "ID-1.formula_name semantic correspondence",
        class: FdbBlockerClass::NativeFormat,
        reason: "the opaque target label is not parsed into or proved equivalent to exact composition plus charge",
        known: "the plan keeps label, canonical ratio and semantic charge separately",
        missing: "provider-accepted formula-label grammar and consistency rule for the admitted profile",
        exact_evidence: Some(
            "controlled fresh neutral and charged groups with recorded formula labels, composition and charge",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "ID-1.compound_name",
        class: FdbBlockerClass::NativeFormat,
        reason: "the formula label alone does not establish the compound-name field",
        known: "ID-1 stores separate 40-byte compound and formula names",
        missing: "fresh FDB compound-name rule for an arbitrary formula group",
        exact_evidence: Some(
            "fresh FDB with a deliberately different display name and formula label",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "ID-1.real_stoichiometric_coefficients[7]",
        class: FdbBlockerClass::NativeFormat,
        reason: "exact semantic ratios do not determine native coefficient scale or slots",
        known: "ID-1 stores seven f64 real coefficients in addition to shared-header integer coefficients",
        missing: "native scale, order and integer-versus-real selection for fresh groups",
        exact_evidence: Some(
            "fresh FDB groups with equivalent proportional formulas and one fractional stoichiometry",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "ID-1.reserved_string_1/reserved_string_2/unknown[4]/padding_final[24]",
        class: FdbBlockerClass::NativeFormat,
        reason: "preserved native fields have no verified constructor values",
        known: "all fields have distinct parsed byte ranges and round-trip exactly",
        missing: "fresh values and whether any depend on formula, charge, version or creation order",
        exact_evidence: Some("controlled fresh FDB with one neutral and one charged formula group"),
        user_evidence_can_unblock: true,
    },
];

const PHASE_GAPS: &[Gap] = &[
    Gap {
        field: "ID-7.phase_id_raw/phase_id_raw_neg",
        class: FdbBlockerClass::NativeFormat,
        reason: "CP links require exact IDs but fresh ID allocation and negative-ID meaning are unproved",
        known: "domain indexing links CP by exact phase_id_raw within the compound group",
        missing: "fresh ID sequence, state convention and phase_id_raw_neg rule",
        exact_evidence: Some(
            "fresh FDB with two base/A pairs under one formula group, inspecting ID-7 and CP links",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "ID-7.physical.padding_1/padding_2",
        class: FdbBlockerClass::NativeFormat,
        reason: "inactive physical coefficients do not establish the adjacent padding bytes",
        known: "the pure H/S/Cp profile rejects active physical tails; raw padding is preserved",
        missing: "fresh padding bytes for ordinary and added ID-7 records",
        exact_evidence: Some("controlled fresh FDB with one ordinary function and one A companion"),
        user_evidence_can_unblock: true,
    },
];

const CP_GAPS: &[Gap] = &[Gap {
    field: "CP.unknown_1[4]/padding_remaining[56]",
    class: FdbBlockerClass::NativeFormat,
    reason: "the parser preserves these bytes but has no fresh construction rule",
    known: "ID-2 and ID-5 share the parsed CP body layout",
    missing: "fresh unknown/padding bytes for each admitted CP kind",
    exact_evidence: Some("controlled fresh FDB with one ID-2 and one ID-5 interval"),
    user_evidence_can_unblock: true,
}];

fn append_gaps(blockers: &mut Vec<FdbBuildBlocker>, object: &str, gaps: &[Gap]) {
    blockers.extend(gaps.iter().map(|gap| gap.on(object)));
}

impl FdbBuildPlan {
    /// Reports remaining native construction and verification work in plan order.
    /// Resolved caller fields and evidenced profile defaults are not blockers.
    pub fn native_blockers(&self) -> Vec<FdbBuildBlocker> {
        let mut blockers = vec![FdbBuildBlocker {
            object: "FDB materializer".into(),
            field: "provider-owned RawDatabase construction",
            class: FdbBlockerClass::Engineering,
            reason: "no sealed plan-to-raw builder exists yet",
            known: "raw serialization, strict reparse, domain indexing and ordinary FDB validation already exist",
            missing: "implement the provider builder after required native field rules are established",
            exact_evidence: None,
            user_evidence_can_unblock: false,
        }];
        append_gaps(&mut blockers, "database header", HEADER_GAPS);
        for group in &self.groups {
            let group_object = group.formula.as_str();
            append_gaps(&mut blockers, group_object, COMMON_GAPS);
            append_gaps(&mut blockers, group_object, GROUP_GAPS);
            if i8::try_from(group.charge.value()).is_err() {
                blockers.push(FdbBuildBlocker {
                    object: group.formula.clone(),
                    field: "RawCommonHeader.charge_raw range",
                    class: FdbBlockerClass::NativeFormat,
                    reason: "the semantic charge exceeds the only parsed native signed-byte slot",
                    known: "charge_raw is i8, while the solution source charge is i32",
                    missing: "an alternative native encoding or explicit rejection policy for this charge",
                    exact_evidence: Some("provider documentation or a controlled FDB containing a charge outside -128..127"),
                    user_evidence_can_unblock: true,
                });
            }
            for function in &group.functions {
                let object = function.identity().target_name.as_str();
                append_gaps(&mut blockers, object, PHASE_GAPS);
                match function {
                    FdbFunctionPlan::ExplicitZeroOrdinary(_) => blockers.push(FdbBuildBlocker {
                        object: object.into(),
                        field: "zero-base physical encoding",
                        class: FdbBlockerClass::NativeFormat,
                        reason: "the rigorous base identity cannot be silently omitted",
                        known: "paired exports may omit physically zero base blocks",
                        missing: "accepted explicit zero-base ID-7/CP representation or versioned omission rule",
                        exact_evidence: Some("controlled paired Legacy import with a zero-range G entry and generated FDB/SLN references"),
                        user_evidence_can_unblock: true,
                    }),
                    FdbFunctionPlan::Ordinary(base) => {
                        for (index, range) in base.ranges.iter().enumerate() {
                            let range_object = format!("{object} range {index}");
                            append_gaps(&mut blockers, &range_object, CP_GAPS);
                            append_unused_cp_gap(&mut blockers, &range_object, range.cp_terms.len());
                        }
                    }
                    FdbFunctionPlan::Added(added) => match &added.contribution {
                        FdbAddedContribution::ExplicitZero => blockers.push(FdbBuildBlocker {
                            object: object.into(),
                            field: "zero-A physical encoding",
                            class: FdbBlockerClass::NativeFormat,
                            reason: "the rigorous A identity cannot be silently omitted",
                            known: "paired exports may omit physically zero A blocks",
                            missing: "accepted explicit zero-A ID-7/ID-5 representation or versioned omission rule",
                            exact_evidence: Some("controlled paired Legacy import with an explicit zero A and generated FDB/SLN references"),
                            user_evidence_can_unblock: true,
                        }),
                        FdbAddedContribution::Thermodynamic { ranges, .. } => {
                            for (index, range) in ranges.iter().enumerate() {
                                let range_object = format!("{object} range {index}");
                                append_gaps(&mut blockers, &range_object, CP_GAPS);
                                append_unused_cp_gap(&mut blockers, &range_object, range.cp_terms.len());
                            }
                        }
                    },
                }
            }
        }
        blockers.push(FdbBuildBlocker {
            object: "constructed FDB".into(),
            field: "independent FactSage acceptance",
            class: FdbBlockerClass::Verification,
            reason: "self-reparse alone cannot prove FactSage accepts a newly constructed FDB",
            known: "provider parser/domain/thermo can verify internal consistency after serialization",
            missing: "a controlled FactSage open/import and target-reference resolution check",
            exact_evidence: Some("FactSage acceptance of one tiny provider-built FDB, with version and action recorded; keep proprietary bytes local"),
            user_evidence_can_unblock: true,
        });
        blockers
    }
}

fn append_unused_cp_gap(blockers: &mut Vec<FdbBuildBlocker>, object: &str, term_count: usize) {
    if term_count != 7 && term_count != 8 {
        blockers.push(FdbBuildBlocker {
            object: object.into(),
            field: "CP unused coefficient/power slots",
            class: FdbBlockerClass::NativeFormat,
            reason: "a shorter semantic term list needs native slot filling without inventing defaults",
            known: "paired seven-term Legacy ranges have a zero eighth slot",
            missing: "fill rule for a source list with fewer than seven meaningful terms",
            exact_evidence: Some("controlled fresh FDB with one Cp expression containing fewer than seven supplied terms"),
            user_evidence_can_unblock: true,
        });
    }
}

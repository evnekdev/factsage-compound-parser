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
        known: "zero occurs in the paired FDB and both local CDBs, so it is a compatibility guardrail rather than an FDB classifier",
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
        known: "all three fields are zero in the paired FDB and both local CDBs; the parser preserves them",
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
        known: "both fields differ between the paired FDB and CDBs; the parser preserves eleven and twelve bytes respectively",
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
        known: "the paired FDB copies both arrays unchanged within each formula group; strict simple integer formula labels match atomic-number IDs, written coefficients and zero unused slots",
        missing: "a construction rule for fractional, charged-label and pseudocomponent groups, plus independent confirmation of the simple-integer profile",
        exact_evidence: Some(
            "paired fresh FDBs with one neutral formula and one changed elemental ratio, inspecting ID-1/ID-7/CP headers",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.coefficient_padding/unknown[2]",
        class: FdbBlockerClass::MissingTestEvidence,
        reason: "paired repeated values do not prove a fresh-construction rule",
        known: "coefficient padding is zero throughout the paired FDB; the two unknown bytes vary by record kind but are uniform within each observed kind",
        missing: "fresh values by record kind under an identified FactSage version",
        exact_evidence: Some(
            "controlled fresh FDB with one group, one ordinary function and one Cp interval",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.charge_raw",
        class: FdbBlockerClass::NativeFormat,
        reason: "semantic group charge is not yet proved to map directly to every repeated signed-byte header",
        known: "the signed i8 field varies between paired FDB groups and repeats unchanged on ID-1, ID-7 and CP within each group; this corpus has no zero-charge group and some unmarked labels have nonzero raw charge",
        missing: "source semantic charge to native signed-byte mapping and neutral/opposite-charge behavior",
        exact_evidence: Some(
            "controlled neutral and charged fresh FDB groups with equal element ratios, comparing shared headers",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.entry_number",
        class: FdbBlockerClass::MissingTestEvidence,
        reason: "the paired FDB record-order counter is not established for fresh files or rollover",
        known: "every paired FDB group increments entry_number once per ID-1/ID-7/CP record in stream order from a uniform group start; comparison CDB groups do not all follow this pattern",
        missing: "fresh FDB start, increment and rollover behavior under a named version",
        exact_evidence: Some(
            "controlled fresh FDB with multiple functions and Cp records in one group, preserving creation order",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.reference[2]",
        class: FdbBlockerClass::NativeFormat,
        reason: "the observed zero references are not a proven fresh FDB default",
        known: "both references are zero across the paired FDB but comparison CDBs contain nonzero references; CP references match the owning phase in all local members",
        missing: "fresh reference initialization, parent-copy and nonzero/rollover conditions",
        exact_evidence: Some(
            "controlled fresh FDB with multiple functions and Cp records, inspecting both references",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.timestamp_ole",
        class: FdbBlockerClass::NativeFormat,
        reason: "the plan's database date does not establish entry timestamp behavior",
        known: "CP timestamps copy the ID-1 group timestamp in the paired FDB and both comparison CDBs; ID-7 timestamps can differ from ID-1 and CP, and the group timestamp need not equal the ID-9 date",
        missing: "fresh ID-1 and ID-7 timestamp sources and assignment rule",
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
        known: "the plan keeps label, canonical ratio and semantic charge separately; the paired FDB has charged labels and unmarked labels with nonzero raw charge",
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
        known: "ID-1 stores separate 40-byte names; they differ for some paired groups, and modern references can use either stored name",
        missing: "fresh compound-name selection and reference-alias rule for a requested group",
        exact_evidence: Some(
            "fresh FDB with a deliberately different display name and formula label",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "ID-1.real_stoichiometric_coefficients[7]",
        class: FdbBlockerClass::NativeFormat,
        reason: "exact semantic ratios do not determine native coefficient scale or slots",
        known: "strict simple integer labels in the paired FDB carry matching real and integer coefficients in label order with zero unused slots; other labels remain unproved",
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
        known: "reserved strings are uniform but nonzero across paired FDB groups; adjacent unknown and final padding fields are zero in that translation and round-trip exactly",
        missing: "fresh values and whether any depend on formula, charge, version or creation order",
        exact_evidence: Some("controlled fresh FDB with one neutral and one charged formula group"),
        user_evidence_can_unblock: true,
    },
];

const PHASE_GAPS: &[Gap] = &[
    Gap {
        field: "ID-7.phase_id_raw allocation",
        class: FdbBlockerClass::MissingTestEvidence,
        reason: "the paired group-local solid allocation pattern has not been confirmed for a newly created FDB",
        known: "paired solid groups allocate consecutive IDs from the first solid index; IDs repeat across groups, CP links are group-local, and phase_id_raw_neg equals the negative ID in the paired FDB and CDBs",
        missing: "independent fresh-FDB confirmation of solid allocation and a rule for other target states",
        exact_evidence: Some(
            "fresh FDB with two solid functions in one group and one non-solid function in another, inspecting ID-7 and CP IDs",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "ID-7.physical.padding_1/padding_2",
        class: FdbBlockerClass::NativeFormat,
        reason: "inactive physical coefficients do not establish the adjacent padding bytes",
        known: "the pure H/S/Cp profile rejects active physical tails; both padding regions are zero throughout the paired FDB but remain raw-preserved",
        missing: "fresh padding bytes for ordinary and added ID-7 records",
        exact_evidence: Some("controlled fresh FDB with one ordinary function and one A companion"),
        user_evidence_can_unblock: true,
    },
];

const CP_BASE_KIND_GAPS: &[Gap] = &[Gap {
    field: "CP.kind selection for counted base range",
    class: FdbBlockerClass::MissingTestEvidence,
    reason: "the paired zero-Cp/nonzero-Cp split has not been confirmed as a fresh writer rule",
    known: "every identity-and-order-linked counted base range with exact-zero source Cp uses ID-5 and every nonzero one uses ID-2 across the represented model families; the paired FDB and comparison CDBs preserve the same ID-2/ID-5 output split",
    missing: "controlled fresh/imported FDB confirmation that exact semantic Cp zero status selects the native kind",
    exact_evidence: Some(
        "one controlled same-version Legacy import with otherwise equivalent zero-Cp and nonzero-Cp counted base ranges",
    ),
    user_evidence_can_unblock: true,
}];

const CP_ADDED_KIND_GAPS: &[Gap] = &[Gap {
    field: "CP.kind selection for added range",
    class: FdbBlockerClass::ScientificSemantics,
    reason: "added/generated source records do not map directly to the retained target Cp terms",
    known: "the paired FDB ID-2/ID-5 zero-Cp split holds under A names too, but source leading terms can be nonzero when the emitted ID-5 Cp coefficients are zero",
    missing: "source-to-target reduction that determines added-range Cp content and then native kind, plus fresh confirmation",
    exact_evidence: Some(
        "controlled same-version Legacy import varying the added source Cp contribution while retaining generated FDB and SLN",
    ),
    user_evidence_can_unblock: true,
}];

const CP_SHARED_GAPS: &[Gap] = &[Gap {
    field: "CP.unknown_1[4]/padding_remaining[56]",
    class: FdbBlockerClass::MissingTestEvidence,
    reason: "observed zero bytes do not prove a fresh construction rule",
    known: "both regions are zero in every observed paired ID-2 and ID-5 record; the parser preserves them",
    missing: "fresh unknown/padding bytes for each admitted CP kind and named FactSage version",
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
                            append_gaps(&mut blockers, &range_object, CP_BASE_KIND_GAPS);
                            append_gaps(&mut blockers, &range_object, CP_SHARED_GAPS);
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
                                append_gaps(&mut blockers, &range_object, CP_ADDED_KIND_GAPS);
                                append_gaps(&mut blockers, &range_object, CP_SHARED_GAPS);
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

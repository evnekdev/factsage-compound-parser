//! Actionable FDB-C3 blockers. These are not semantic-plan validation errors.

use super::{FdbAddedContribution, FdbBuildPlan, FdbConstructionProfile, FdbFunctionPlan};
use crate::PhaseState;

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
        reason: "fresh snapshots agree, but their creation version and read-flag policy are not recorded",
        known: "the directly authored FDB snapshots and the translated FDB share the same read flag",
        missing: "a version-scoped fresh initialization rule, independently accepted after provider construction",
        exact_evidence: Some(
            "one controlled newly created FDB header with FactSage version and creation action recorded",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "ID-9.padding_1/padding_2/padding_3",
        class: FdbBlockerClass::NativeFormat,
        reason: "matching fresh and translated padding is observed but not a universal constructor policy",
        known: "all three padding regions are zero in every receipt-admitted fresh FDB and the translated FDB",
        missing: "version-scoped initialization and independent acceptance of provider-written padding",
        exact_evidence: Some(
            "one controlled newly created FDB header with FactSage version and creation action recorded",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "ID-9.unknown_1/unknown_2",
        class: FdbBlockerClass::NativeFormat,
        reason: "uninterpreted header fields cannot receive an invented default",
        known: "both fields are stable across the fresh edit sequence and unchanged when a charged group is added; the first matches the translated FDB, while the second differs",
        missing: "the recorded FactSage version and a rule for assigning both fields in a new file",
        exact_evidence: Some(
            "one independently created fresh FDB with version, action and complete semantic input recorded",
        ),
        user_evidence_can_unblock: true,
    },
];

const COMMON_GAPS: &[Gap] = &[
    Gap {
        field: "RawCommonHeader.element_ids[7]/element_coefficients[7]",
        class: FdbBlockerClass::NativeFormat,
        reason: "source symbols and exact ratios are not yet mapped to repeated native element slots",
        known: "fresh ID-1/ID-7/CP records copy both arrays within the group; the declared neutral and charged groups have equal native arrays; simple translated integer labels matched native IDs and coefficients",
        missing: "an input-linked element order, coefficient scale and charge-aware mapping for even a bounded fresh composition profile",
        exact_evidence: Some(
            "paired fresh FDBs with one neutral formula and one changed elemental ratio, inspecting ID-1/ID-7/CP headers",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.coefficient_padding/unknown[2]",
        class: FdbBlockerClass::MissingTestEvidence,
        reason: "observed record-kind constants lack versioned constructor provenance",
        known: "fresh coefficient padding and per-kind unknown bytes are stable through the controlled edits and match translated FDB counterparts",
        missing: "version-scoped fresh assignment and independent acceptance of provider-written bytes",
        exact_evidence: Some(
            "named-version FactSage acceptance of a provider-built one-function fresh FDB",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.charge_raw",
        class: FdbBlockerClass::NativeFormat,
        reason: "the fresh-modern offset rule is profile-specific and does not establish Legacy-translation charge encoding",
        known: "the bounded fresh-modern rule adds the neutral offset to signed charge and copies the result to ID-1/ID-7/CP; direct cast and complement rules fail on the fresh probe",
        missing: "a source-to-native charge rule for Legacy translation",
        exact_evidence: Some(
            "controlled Legacy charge input matched to translated FDB shared headers",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.entry_number",
        class: FdbBlockerClass::MissingTestEvidence,
        reason: "the small fresh sequence confirms the counter pattern, but larger groups and rollover remain outside evidence",
        known: "fresh and translated FDBs start each group alike and increment entry_number once per ID-1/ID-7/CP record in stream order; two fresh singleton groups restart independently",
        missing: "a bounded maximum or overflow/restart rule for larger groups, plus provider-written acceptance",
        exact_evidence: Some(
            "controlled fresh FDB with multiple functions and Cp records in one group, preserving creation order",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.reference[2]",
        class: FdbBlockerClass::NativeFormat,
        reason: "the fresh reference edit changes only the phase header's reference slots; input-to-slot mapping is unproved",
        known: "fresh group and CP references remain zero in the controlled edit, while phase references change; both groups in the charged probe have zero reference slots",
        missing: "input-linked phase reference encoding for active references or versioned zero-reference initialization for a bounded profile",
        exact_evidence: Some(
            "controlled fresh FDB with multiple functions and Cp records, inspecting both references",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "RawCommonHeader.timestamp_ole",
        class: FdbBlockerClass::NativeFormat,
        reason: "fresh entry timestamps change on edit while the ID-9 date stays fixed, without a source-time policy",
        known: "fresh ID-1, ID-7 and CP timestamps agree within each group; two groups in one file differ while ID-9 date is unchanged; translated CP copies ID-1 but ID-7 can differ",
        missing: "fresh timestamp source, rounding and deterministic assignment policy",
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
        known: "the fresh neutral/charged groups have different formula labels despite equal native element arrays and a reused function name; compound names are empty; label spelling is not a charge oracle",
        missing: "provider-accepted formula-label grammar and consistency rule for the admitted profile",
        exact_evidence: Some(
            "record exact formula, composition and signed charge inputs for the existing fresh pair, then check provider reference resolution",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "ID-1.compound_name",
        class: FdbBlockerClass::NativeFormat,
        reason: "the formula label alone does not establish the compound-name policy across construction profiles",
        known: "fresh snapshots keep the compound-name field empty while the formula label is populated; translated groups can have distinct populated names",
        missing: "bounded fresh policy for omitted or supplied compound names and reference resolution",
        exact_evidence: Some(
            "fresh FDB with a deliberately different display name and formula label",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "ID-1.real_stoichiometric_coefficients[7]",
        class: FdbBlockerClass::NativeFormat,
        reason: "exact semantic ratios do not determine native coefficient scale or slots",
        known: "fresh coefficients stay stable through edits and are equal across the declared neutral/charged groups; simple translated integer labels matched native real and integer coefficients in label order",
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
        known: "fresh reserved strings are stable and nonzero through edits and the charged contrast, while adjacent unknown and final padding are zero; translated strings may follow another profile",
        missing: "version-scoped fresh values and provider acceptance; dependence on formula or charge was not observed in this pair",
        exact_evidence: Some(
            "record the version of the existing fresh pair and independently accept a provider-built bounded FDB",
        ),
        user_evidence_can_unblock: true,
    },
];

const PHASE_GAPS: &[Gap] = &[
    Gap {
        field: "ID-7.phase_id_raw allocation",
        class: FdbBlockerClass::MissingTestEvidence,
        reason: "fresh singleton groups reuse a solid phase ID, but allocation of a second function within one group is unproved",
        known: "the fresh neutral/charged singleton groups reuse a solid phase ID with correct negatives and CP links; translated solid groups allocate consecutive local IDs",
        missing: "fresh allocation for a second function in one group and any admitted non-solid state",
        exact_evidence: Some(
            "fresh FDB with a second solid function in the same group; test non-solid allocation only if admitted",
        ),
        user_evidence_can_unblock: true,
    },
    Gap {
        field: "ID-7.physical.padding_1/padding_2",
        class: FdbBlockerClass::NativeFormat,
        reason: "inactive physical coefficients do not establish the adjacent padding bytes",
        known: "both padding regions are zero in fresh and translated FDB phases; the fresh density edit changes the density field without changing other physical-tail fields",
        missing: "version-scoped writer acceptance for ordinary ID-7 padding; added A requires separate translation evidence",
        exact_evidence: Some(
            "FactSage acceptance of a provider-built fresh ordinary function with the observed padding",
        ),
        user_evidence_can_unblock: true,
    },
];

const CP_BASE_KIND_GAPS: &[Gap] = &[Gap {
    field: "CP.kind selection for counted base range",
    class: FdbBlockerClass::MissingTestEvidence,
    reason: "the translated zero/nonzero split is not yet established as the direct-modern writer rule",
    known: "all observed fresh nonzero-Cp ranges use ID-2; identity-linked translated zero-Cp counted bases use ID-5 and nonzero ones ID-2",
    missing: "controlled fresh zero-Cp output or independent acceptance of a candidate canonical encoding",
    exact_evidence: Some(
        "one controlled fresh-modern zero-Cp range under the same recorded version as a nonzero-Cp range",
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

const CP_FRESH_ZERO_KIND_GAPS: &[Gap] = &[Gap {
    field: "CP.kind selection for fresh zero-Cp range",
    class: FdbBlockerClass::UserPolicyDecision,
    reason: "the controlled fresh ranges all have nonzero Cp and use ID-2; no direct-modern zero-Cp range was admitted",
    known: "ID-2 is the observed fresh nonzero-Cp form; the translated output uses ID-5 for exact-zero Cp, but that is not a direct-modern constructor rule",
    missing: "a fresh zero-Cp selector, or a canonical ID-2 fresh-write policy validated by strict reparse, independent FactSage acceptance and load/open/save stability",
    exact_evidence: Some(
        "one same-version fresh zero-Cp range with recorded UI inputs and a FactSage load/open/save check",
    ),
    user_evidence_can_unblock: true,
}];

const CP_SHARED_GAPS: &[Gap] = &[Gap {
    field: "CP.unknown_1[4]/padding_remaining[56]",
    class: FdbBlockerClass::MissingTestEvidence,
    reason: "matching observed zero bytes still require version-scoped provider-written acceptance",
    known: "both regions are zero in every observed fresh ID-2 and translated ID-2/ID-5 record",
    missing: "fresh rule for each admitted CP kind and independent acceptance of provider-written bytes",
    exact_evidence: Some(
        "FactSage acceptance of provider-built fresh ID-2; test ID-5 only if admitted",
    ),
    user_evidence_can_unblock: true,
}];

fn append_gaps(blockers: &mut Vec<FdbBuildBlocker>, object: &str, gaps: &[Gap]) {
    blockers.extend(gaps.iter().map(|gap| gap.on(object)));
}

impl FdbBuildPlan {
    /// Reports remaining native construction and verification work in plan order.
    ///
    /// Resolved FDB serialization rules (template-derived opaque bytes, charge
    /// encoding, composition slots, counters, timestamps, references, Function
    /// IDs, fresh ID-2 zero-Cp, and unused-slot zero filling) are deliberately
    /// not emitted as blockers.
    pub fn native_blockers(&self) -> Vec<FdbBuildBlocker> {
        let mut blockers = vec![FdbBuildBlocker {
            object: "FDB materializer".into(),
            field: "provider-owned RawDatabase construction",
            class: FdbBlockerClass::Engineering,
            reason: "no sealed plan-to-raw builder exists yet",
            known: "raw serialization, strict reparse, domain indexing and ordinary FDB validation already exist",
            missing: "implement the provider builder using the resolved native policies",
            exact_evidence: None,
            user_evidence_can_unblock: false,
        }];

        for group in &self.groups {
            for function in &group.functions {
                let object = function.identity().target_name.as_str();
                match function {
                    FdbFunctionPlan::ExplicitZeroOrdinary(_)
                        if self.profile == FdbConstructionProfile::LegacyTranslation =>
                    {
                        // Confirmed native serialization policy: physically zero
                        // Legacy base objects may be omitted while semantic identity
                        // remains in the rigorous graph.
                    }
                    FdbFunctionPlan::ExplicitZeroOrdinary(_) => {
                        blockers.push(FdbBuildBlocker {
                            object: object.into(),
                            field: "fresh empty-function verification",
                            class: FdbBlockerClass::Engineering,
                            reason: "the ordinary thermodynamic view requires at least one CP range",
                            known: "a directly authored fresh empty function has an ID-7 and no CP record",
                            missing: "verify an empty fresh function by strict reparse and domain structure without invoking ordinary H/S/Cp evaluation",
                            exact_evidence: None,
                            user_evidence_can_unblock: false,
                        });
                    }
                    FdbFunctionPlan::Ordinary(base) => {
                        if self.profile == FdbConstructionProfile::LegacyTranslation {
                            for (index, _range) in base.ranges.iter().enumerate() {
                                let range_object = format!("{object} range {index}");
                                append_gaps(&mut blockers, &range_object, CP_BASE_KIND_GAPS);
                            }
                        }
                    }
                    FdbFunctionPlan::Added(added) => match &added.contribution {
                        FdbAddedContribution::ExplicitZero => {
                            // Confirmed native serialization policy: physically
                            // zero Legacy A objects may be omitted.
                        }
                        FdbAddedContribution::Thermodynamic { ranges, .. } => {
                            for (index, _range) in ranges.iter().enumerate() {
                                let range_object = format!("{object} range {index}");
                                append_gaps(&mut blockers, &range_object, CP_ADDED_KIND_GAPS);
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

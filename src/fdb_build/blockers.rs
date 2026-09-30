//! Actionable FDB-C3 blockers. These are not semantic-plan validation errors.

use super::{FdbBuildPlan, FdbConstructionProfile, FdbFunctionPlan};

/// Provider-neutral blocker category, translatable to application reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdbBlockerClass {
    /// The rule is known; implementation remains.
    Engineering,
    /// An owning provider API is absent.
    ProviderApi,
    /// Native field encoding or output convention is unproved.
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

impl FdbBuildPlan {
    /// Reports remaining construction and verification work in plan order.
    ///
    /// Resolved FDB serialization rules are deliberately not emitted as
    /// blockers. Legacy source-to-target scientific conversion uncertainties
    /// belong to the conversion adapter; an already explicit target plan is not
    /// blocked from native materialization by those upstream questions.
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

        if self.profile == FdbConstructionProfile::FreshModern {
            for group in &self.groups {
                for function in &group.functions {
                    if matches!(function, FdbFunctionPlan::ExplicitZeroOrdinary(_)) {
                        blockers.push(FdbBuildBlocker {
                            object: function.identity().target_name.clone(),
                            field: "fresh empty-function verification",
                            class: FdbBlockerClass::Engineering,
                            reason: "the ordinary thermodynamic view requires at least one CP range",
                            known: "a directly authored fresh empty function has an ID-7 and no CP record",
                            missing: "verify an empty fresh function by strict reparse and domain structure without invoking ordinary H/S/Cp evaluation",
                            exact_evidence: None,
                            user_evidence_can_unblock: false,
                        });
                    }
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

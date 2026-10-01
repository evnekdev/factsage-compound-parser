//! Actionable FDB-C3 blockers. These are not semantic-plan validation errors.

use super::{FdbBuildPlan, FdbConstructionProfile};

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
        let mut blockers = Vec::new();
        if self.profile == FdbConstructionProfile::LegacyTranslation
            && self.legacy_zero_added_physical_plan().is_err()
        {
            blockers.push(FdbBuildBlocker {
                object: "LegacyTranslation FDB materializer".into(),
                field: "explicit target A and zero-object native emission",
                class: FdbBlockerClass::Engineering,
                reason: "the bounded translated writer accepts only nonzero-Cp ID-2 bases with explicitly zero, physically omitted A companions",
                known: "the shared FDB charge, composition, phase ID and entry policies are resolved; the zero-added translated subset has an emitter",
                missing: "add native ID-5/active-A and zero-base policies for this plan's unsupported objects",
                exact_evidence: None,
                user_evidence_can_unblock: false,
            });
        }

        blockers.push(FdbBuildBlocker {
            object: "constructed FDB".into(),
            field: "independent FactSage acceptance",
            class: FdbBlockerClass::Verification,
            reason: "a corrected one-function FDB opened and saved byte-identically, and a hydrogen-only Function opened in FactSage 7.3; other admitted shapes lack native coverage",
            known: "the bounded writer preserves ID-9, places hydrogen last, self-verifies, and has native one-function and hydrogen-only acceptance",
            missing: "native checks for additional function and charge-group shapes and target-reference resolution",
            exact_evidence: Some("controlled FactSage acceptance of an unverified shape; keep proprietary bytes local"),
            user_evidence_can_unblock: true,
        });

        blockers
    }
}

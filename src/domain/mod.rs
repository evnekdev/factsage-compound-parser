//! Zero-duplication semantic indexes and borrowed views over raw CDB chunks.
//!
//! [`DomainIndex`] contains only physical chunk indexes, relationships, and
//! diagnostics. [`DatabaseView`] and its child views borrow the authoritative
//! [`crate::RawDatabase`], so grouping does not clone complete raw records.

mod compound;
mod database;
mod error;
mod phase;
mod range;
mod text;

/// Re-export of borrowed compound semantic views.
pub use compound::CompoundView;
/// Re-exports for owned database handles, indexes, and borrowed database views.
pub use database::{Database, DatabaseView, DomainIndex};
/// Re-exports for domain construction errors, diagnostics, and text errors.
pub use error::{
    DatabaseError, Diagnostic, DiagnosticKind, DomainError, ExpectedCategory, TextDecodeError,
};
/// Re-exports for borrowed phase views and inferred raw-ID states.
pub use phase::{PhaseState, PhaseView, RawPhase};
/// Re-exports for borrowed range and orphan-range views.
pub use range::{
    HeatCapacityRangeView, OrphanRangeView, OrphanReason, PhysicalPropertyRangeView, RangeView,
};

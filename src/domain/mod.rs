//! Read-only semantic grouping for parsed FactSage CDB chunks.

mod compound;
mod database;
mod error;
mod phase;
mod range;
mod text;

pub use compound::Compound;
pub use database::Database;
pub use error::{
    DatabaseError, Diagnostic, DiagnosticKind, DomainError, ExpectedCategory, TextDecodeError,
};
pub use phase::{Phase, PhaseDefinition, PhaseState, RawPhase};
pub use range::{HeatCapacityRange, OrphanRange, OrphanReason, PhysicalPropertyRange, Range};

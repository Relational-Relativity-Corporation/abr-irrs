/// irrs-eval: pre-build declaration evaluator.
///
/// Public API for use by domain builders who want to embed evaluation
/// into their own binaries or test suites.

pub mod record;
pub mod evaluator;

pub use record::{
    EvaluationRecord, EvaluationStatus, InventoryItem, InventoryKind,
    PrerequisiteReport, MappingChainEntry, OriginAuthorization,
};
pub use evaluator::{
    DeclarationSpec, Evaluator,
    ObservableSpec, PrerequisiteSpec, MappingSpec,
};

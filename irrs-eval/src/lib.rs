/// irrs-eval: domain-independent pre-build declaration evaluator.
///
/// Public API for use by domain builders who want to embed evaluation
/// into their own binaries or test suites.
///
/// Supply a `Declaration` loaded from a JSON file; receive an `EvaluationRecord`.
/// No domain-specific code lives in this crate.

pub mod declaration;
pub mod provenance;
pub mod evidence;
pub mod authorization;
pub mod record;
pub mod evaluator;

pub use declaration::{
    Declaration, InputDeclaration, InputClassification, ProvenanceStatus,
    EvidenceRef, RegionalPrerequisiteDecl, MappingDeclaration,
};
pub use record::{
    EvaluationRecord, EvaluationStatus, InventoryItem, InventoryKind,
    PrerequisiteReport, MappingChainEntry, OriginAuthorization,
};
pub use provenance::ProvenanceSummary;
pub use evaluator::Evaluator;

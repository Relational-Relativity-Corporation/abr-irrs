/// `relational-gate` — General-Purpose Relational Admission Enforcement
///
/// # What this crate does
///
/// It makes a single guarantee: no value of type `AdmittedObservation<T>`
/// exists in your program unless it passed through a declared, verified
/// admission gate.
///
/// # What this crate does NOT do
///
/// It does not enforce what you do with an admitted value after admission.
/// It does not implement your domain mathematics.
/// It cannot establish physical observational provenance from a type alone.
///
/// # How to use it
///
/// 1. Implement `Observable` for your raw source type.
/// 2. Declare your region (`RegionBuilder`) and topology (`TopologySpec`).
/// 3. Register your authorized `AuthorizedMapping`s.
/// 4. Build an `AdmissionGate`.
/// 5. Submit `DeclarationPackage`s. Receive `AdmissionOutcome`s.
/// 6. Only `AdmittedObservation<T>` values may enter your relational treatment.
///
/// The type system enforces step 6. All other steps are verified at the gate.

pub mod observable;
pub mod declaration;
pub mod admitted;
pub mod mapping;
pub mod region;
pub mod topology;
pub mod gate;
pub mod trace;

pub use observable::Observable;
pub use declaration::{DeclarationPackage, ProvenanceChain, ProvenanceLink};
pub use admitted::{AdmittedObservation, ProvenanceRecord};
pub use mapping::{AuthorizedMapping, MappingError};
pub use region::{
    RegionBuilder, EstablishedRegion, ObservableExtent,
    CrossFamilyOrientation, RegionalAdmissionRule, RegionError,
};
pub use topology::{TopologySpec, DeclaredEdge, TopologyError};
pub use gate::{AdmissionGate, AdmissionOutcome, AdmissionError};
pub use trace::{TraceRecord, OperatorTrace, Divergence, ConformanceSummary};

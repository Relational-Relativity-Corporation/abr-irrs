/// AdmissionGate<O, V> — the general-purpose relational enforcement gate.
///
/// `O`: observable source type (implements `Observable`)
/// `V`: admitted value type (produced by `AuthorizedMapping<V>`)
///
/// The gate is the only code path that produces `AdmittedObservation<V>`.
/// Application code cannot construct `AdmittedObservation<V>` directly.
/// The compiler enforces this unconditionally.

use crate::observable::Observable;
use crate::declaration::DeclarationPackage;
use crate::admitted::{AdmittedObservation, ProvenanceRecord};
use crate::mapping::AuthorizedMapping;
use crate::topology::TopologySpec;
use crate::trace::{TraceRecord, Divergence};

/// The outcome of one submission to the gate.
///
/// `Executed`  — admitted, mapping executed, trace produced.
/// `Rejected`  — not admitted for evaluation. Structural/mathematical rejection.
///               Does not mean a physical relation failed.
/// `Divergent` — admitted and executed, but result diverged from a declared
///               expected value. Preserved, not suppressed.
pub enum AdmissionOutcome<V: Clone> {
    Executed {
        observation: AdmittedObservation<V>,
        trace:       TraceRecord<V>,
    },
    Rejected {
        error:        AdmissionError,
        topology_id:  String,
        mapping_name: String,
    },
    Divergent {
        observation: AdmittedObservation<V>,
        trace:       TraceRecord<V>,
        divergence:  Divergence,
    },
}

/// The general-purpose admission gate.
pub struct AdmissionGate<O: Observable, V: Clone> {
    topology: TopologySpec,
    mappings: Vec<AuthorizedMapping<V>>,
    _phantom: std::marker::PhantomData<O>,
}

impl<O: Observable, V: Clone> AdmissionGate<O, V> {
    pub fn new(topology: TopologySpec, mappings: Vec<AuthorizedMapping<V>>) -> Self {
        AdmissionGate { topology, mappings, _phantom: std::marker::PhantomData }
    }

    /// Submit a declaration package for admission.
    ///
    /// The gate:
    ///   1. Validates the declaration structure.
    ///   2. Verifies the topology ID.
    ///   3. Locates the named authorized mapping.
    ///   4. Independently executes the mapping from raw inputs.
    ///   5. Verifies the result scalar is interior to declared topology extents.
    ///   6. Constructs `AdmittedObservation<V>` (private constructor).
    ///   7. Returns the trace record.
    pub fn submit(&self, package: DeclarationPackage<O>) -> AdmissionOutcome<V> {
        let topology_id  = package.topology_id.to_string();
        let mapping_name = package.mapping_name.to_string();

        // ── 1. Declaration structure ─────────────────────────────────────────
        if package.topology_id.is_empty() {
            return self.reject(
                AdmissionError::MissingDeclaration { field: "topology_id" },
                topology_id, mapping_name,
            );
        }
        if package.mapping_name.is_empty() {
            return self.reject(
                AdmissionError::MissingDeclaration { field: "mapping_name" },
                topology_id, mapping_name,
            );
        }
        if package.provenance.is_empty() {
            return self.reject(
                AdmissionError::MissingDeclaration { field: "provenance_chain" },
                topology_id, mapping_name,
            );
        }
        if let Err(e) = package.provenance.verify_no_silent_links() {
            return self.reject(
                AdmissionError::SilentTransformation { detail: format!("{}", e) },
                topology_id, mapping_name,
            );
        }

        // ── 2. Topology ID ───────────────────────────────────────────────────
        if let Err(e) = self.topology.verify_id(package.topology_id) {
            return self.reject(
                AdmissionError::TopologyMismatch { detail: format!("{}", e) },
                topology_id, mapping_name,
            );
        }

        // ── 3. Locate mapping ────────────────────────────────────────────────
        let mapping = match self.mappings.iter().find(|m| m.name == package.mapping_name) {
            Some(m) => m,
            None => return self.reject(
                AdmissionError::UnknownMapping { name: mapping_name.clone() },
                topology_id, mapping_name,
            ),
        };

        // ── 4. Gate independently executes the mapping ───────────────────────
        let primary   = package.source.primary_value();
        let reference = package.source.reference_value();

        let admitted_value = match (mapping.execute)(primary, reference) {
            Ok(v)  => v,
            Err(e) => return self.reject(
                AdmissionError::MappingFailed { detail: format!("{}", e) },
                topology_id, mapping_name,
            ),
        };

        // ── 5. Interior-locus condition ──────────────────────────────────────
        let scalar = (mapping.to_scalar)(&admitted_value);
        if let Err(e) = self.topology.verify_interior(scalar) {
            return self.reject(
                AdmissionError::TopologyViolation { detail: format!("{}", e) },
                topology_id, mapping_name,
            );
        }

        // ── 6. Construct AdmittedObservation (private constructor) ───────────
        let provenance = ProvenanceRecord {
            source_id:            package.source.source_id().to_string(),
            topology_id:          package.topology_id.to_string(),
            mapping_applied:      mapping.name.to_string(),
            primary_input:        primary,
            reference_input:      reference,
            gate_computed_result: scalar,
        };
        let observation = AdmittedObservation::new_admitted(admitted_value.clone(), provenance);

        // ── 7. Trace ─────────────────────────────────────────────────────────
        let trace = TraceRecord {
            provenance:     observation.provenance().clone(),
            admitted_value,
            divergence:     None,
        };

        AdmissionOutcome::Executed { observation, trace }
    }

    fn reject(
        &self,
        error:        AdmissionError,
        topology_id:  String,
        mapping_name: String,
    ) -> AdmissionOutcome<V> {
        AdmissionOutcome::Rejected { error, topology_id, mapping_name }
    }
}

#[derive(Debug)]
pub enum AdmissionError {
    MissingDeclaration   { field: &'static str },
    SilentTransformation { detail: String },
    TopologyMismatch     { detail: String },
    TopologyViolation    { detail: String },
    UnknownMapping       { name: String },
    MappingFailed        { detail: String },
}

impl std::fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdmissionError::MissingDeclaration { field } =>
                write!(f, "rejected: '{}' not declared (not defaulted)", field),
            AdmissionError::SilentTransformation { detail } =>
                write!(f, "rejected: silent transformation — {}", detail),
            AdmissionError::TopologyMismatch { detail } =>
                write!(f, "rejected: topology mismatch — {}", detail),
            AdmissionError::TopologyViolation { detail } =>
                write!(f, "rejected: topology violation — {}", detail),
            AdmissionError::UnknownMapping { name } =>
                write!(f, "rejected: mapping '{}' not declared in this topology", name),
            AdmissionError::MappingFailed { detail } =>
                write!(f, "rejected: mapping execution failed — {}", detail),
        }
    }
}

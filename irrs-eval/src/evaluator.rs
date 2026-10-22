/// Pre-build evaluator.
///
/// Takes a `DeclarationSpec` describing a proposed experiment — its observables,
/// regional prerequisites, and authorized mappings — and produces an
/// `EvaluationRecord` that documents what was found.
///
/// The evaluator runs before any `DeclarationPackage` is submitted to the gate.
/// It does not execute mappings. It identifies what is observable versus derived
/// or declared, verifies that the Regional M prerequisite is fully stated, and
/// traces the authorized mapping chain.

use crate::record::{
    EvaluationRecord, InventoryItem, InventoryKind,
    PrerequisiteReport, MappingChainEntry, EvaluationStatus,
};

// ── Input spec types ─────────────────────────────────────────────────────────

/// One entry in the observable inventory.
pub struct ObservableSpec {
    pub name:           &'static str,
    pub source:         &'static str,
    pub physical_basis: &'static str,
    pub kind:           InventoryKind,
    /// True if the Generator submits this value in its DeclarationPackage.
    pub submitted_to_gate: bool,
}

/// The Regional M prerequisite declaration.
pub struct PrerequisiteSpec {
    pub region_name:              &'static str,
    pub extents_lower:            f64,
    pub extents_upper:            f64,
    pub extents_label:            &'static str,
    pub orientation_preserved:    &'static str,
    pub orientation_discarded:    &'static str,
    pub admission_rule_name:      &'static str,
    pub admission_rule_description: &'static str,
    /// Set true when the domain builder has called `establish(locus)` and
    /// the locus was verified interior. The evaluator records this; it does
    /// not re-verify at evaluation time.
    pub interior_locus_verified:  bool,
}

/// One authorized mapping.
pub struct MappingSpec {
    pub name:                &'static str,
    pub input_description:   &'static str,
    pub output_description:  &'static str,
    pub preserves:           &'static str,
    pub discards:            &'static str,
}

/// Complete description of a proposed experiment.
pub struct DeclarationSpec {
    pub domain:      &'static str,
    pub topology_id: &'static str,
    pub version:     &'static str,
    pub observables: Vec<ObservableSpec>,
    pub prerequisite: PrerequisiteSpec,
    pub mappings:    Vec<MappingSpec>,
}

// ── Evaluator ────────────────────────────────────────────────────────────────

pub struct Evaluator;

impl Evaluator {
    /// Evaluate a declaration spec and produce a record.
    ///
    /// Does not execute mappings. Does not submit values to the gate.
    /// Does not require the gate to be constructed.
    pub fn evaluate(spec: DeclarationSpec) -> EvaluationRecord {
        let record_id = format!("{}-{}-{}", spec.domain, spec.topology_id, spec.version);

        // ── Observable inventory ─────────────────────────────────────────────
        let observable_inventory: Vec<InventoryItem> = spec.observables
            .into_iter()
            .map(|o| InventoryItem {
                name:             o.name.to_string(),
                kind:             o.kind,
                source:           o.source.to_string(),
                physical_basis:   o.physical_basis.to_string(),
                submitted_to_gate: o.submitted_to_gate,
            })
            .collect();

        // ── Regional M prerequisite report ───────────────────────────────────
        let p = &spec.prerequisite;
        let all_fields_declared =
            !p.region_name.is_empty()
            && !p.orientation_preserved.is_empty()
            && !p.orientation_discarded.is_empty()
            && !p.admission_rule_name.is_empty()
            && !p.admission_rule_description.is_empty();

        let regional_prerequisite = PrerequisiteReport {
            region_name:               p.region_name.to_string(),
            extents:                   format!("({}, {}) {}",
                                           p.extents_lower,
                                           p.extents_upper,
                                           p.extents_label),
            orientation_preserved:     p.orientation_preserved.to_string(),
            orientation_discarded:     p.orientation_discarded.to_string(),
            admission_rule_name:       p.admission_rule_name.to_string(),
            admission_rule_description: p.admission_rule_description.to_string(),
            all_fields_declared,
            interior_locus_verified:   p.interior_locus_verified,
        };

        // ── Mapping chain ────────────────────────────────────────────────────
        let mapping_chain: Vec<MappingChainEntry> = spec.mappings
            .into_iter()
            .map(|m| MappingChainEntry {
                mapping_name:        m.name.to_string(),
                input_description:   m.input_description.to_string(),
                output_description:  m.output_description.to_string(),
                preserves:           m.preserves.to_string(),
                discards:            m.discards.to_string(),
                executor:            "AdmissionGate (not Generator)".to_string(),
            })
            .collect();

        // ── Evaluation result ────────────────────────────────────────────────
        let (status, failure_reason) = if !all_fields_declared {
            (EvaluationStatus::Failed,
             Some("Regional M prerequisite: one or more fields not declared (no defaults permitted)".to_string()))
        } else if !p.interior_locus_verified {
            (EvaluationStatus::Failed,
             Some("Interior-locus condition not verified: establish(locus) must be called with a verified interior point before topology instantiation".to_string()))
        } else if mapping_chain.is_empty() {
            (EvaluationStatus::Failed,
             Some("No authorized mappings declared: at least one AuthorizedMapping must be registered".to_string()))
        } else {
            (EvaluationStatus::PassedAwaitingAuthorization, None)
        };

        EvaluationRecord {
            record_id,
            domain:      spec.domain.to_string(),
            topology_id: spec.topology_id.to_string(),
            version:     spec.version.to_string(),
            observable_inventory,
            regional_prerequisite,
            mapping_chain,
            status,
            failure_reason,
            authorization: None,
        }
    }
}

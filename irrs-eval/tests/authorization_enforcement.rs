/// Negative tests for authorization enforcement.
///
/// These tests verify that the three enforcement rules in `authorization::apply`
/// cannot be bypassed under any circumstance:
///
/// 1. A failed evaluation cannot be authorized.
/// 2. A mismatched declaration_digest is rejected.
/// 3. An input claiming "verified" status without evidence is flagged
///    by the evaluator (structural inconsistency test).
///
/// These are the three authorization security properties identified by
/// the Verifier in commit b245038d review.

use irrs_eval::{
    authorization,
    record::{EvaluationStatus, OriginAuthorization},
};

// ── Fixtures ──────────────────────────────────────────────────────────────────

fn make_authorization(record_id: &str, declaration_digest: &str) -> OriginAuthorization {
    OriginAuthorization {
        record_id:          record_id.to_string(),
        declaration_digest: declaration_digest.to_string(),
        authorized_by:      "test-origin".to_string(),
        authorized_at:      "2026-01-01".to_string(),
        statement:          "Test authorization.".to_string(),
    }
}

// ── Test 1: A failed evaluation cannot be authorized ─────────────────────────

#[test]
fn test_failed_cannot_be_authorized() {
    use irrs_eval::declaration::{
        Declaration, RegionalPrerequisiteDecl,
    };
    use irrs_eval::evaluator::Evaluator;

    // Build a declaration that will fail: interior_locus_verified = false.
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "fail_topology".to_string(),
        version:     "0.0.1".to_string(),
        description: Some("Declaration that fails prerequisite check.".to_string()),
        inputs: vec![
            irrs_eval::declaration::InputDeclaration {
                name:                  "x".to_string(),
                classification:        irrs_eval::declaration::InputClassification::DeclaredParameter,
                source_identity:       "test".to_string(),
                acquisition_method:    "test".to_string(),
                transformation_history: None,
                relationship_to_problem: "test input".to_string(),
                supporting_evidence:   vec![],
                known_limitations:     None,
                provenance_status:     irrs_eval::declaration::ProvenanceStatus::Declared,
                submitted_to_gate:     true,
            }
        ],
        regional_prerequisite: RegionalPrerequisiteDecl {
            region_name:              "test_region".to_string(),
            extents_lower:            0.0,
            extents_upper:            1.0,
            extents_label:            "fraction".to_string(),
            orientation_preserved:    "ratio".to_string(),
            orientation_discarded:    "magnitude".to_string(),
            admission_rule_name:      "test_rule".to_string(),
            admission_rule_description: "Test admission rule.".to_string(),
            // FAIL: interior locus not verified.
            interior_locus_verified:  false,
        },
        mappings: vec![
            irrs_eval::declaration::MappingDeclaration {
                name:               "test_mapping".to_string(),
                input_description:  "x".to_string(),
                output_description: "y".to_string(),
                preserves:          "order".to_string(),
                discards:           "magnitude".to_string(),
            }
        ],
    };

    let mut record = Evaluator::evaluate(decl, "test-digest-failed".to_string());

    // Confirm the evaluation failed.
    assert_eq!(
        record.status, EvaluationStatus::Failed,
        "Expected evaluation to fail due to interior_locus_verified = false"
    );

    let auth = make_authorization(&record.record_id, &record.declaration_digest);
    authorization::apply(&mut record, auth);

    // Authorization must not have been applied.
    assert_eq!(
        record.status, EvaluationStatus::Failed,
        "Status must remain Failed after authorization attempt on a failed record"
    );
    assert!(
        record.authorization.is_none(),
        "Authorization must not be stored on a failed record"
    );
    // The enforcement rejection must be noted.
    assert!(
        record.evaluation_notes.iter().any(|n| n.contains("Authorization REJECTED")),
        "evaluation_notes must contain an 'Authorization REJECTED' entry"
    );
}

// ── Test 2: Mismatched declaration_digest is rejected ────────────────────────

#[test]
fn test_mismatched_digest_rejected() {
    use irrs_eval::declaration::{
        Declaration, RegionalPrerequisiteDecl,
    };
    use irrs_eval::evaluator::Evaluator;

    // Build a declaration that passes.
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "digest_topology".to_string(),
        version:     "0.0.1".to_string(),
        description: Some("Declaration that passes.".to_string()),
        inputs: vec![
            irrs_eval::declaration::InputDeclaration {
                name:                  "y".to_string(),
                classification:        irrs_eval::declaration::InputClassification::DeclaredParameter,
                source_identity:       "test".to_string(),
                acquisition_method:    "test".to_string(),
                transformation_history: None,
                relationship_to_problem: "test input".to_string(),
                supporting_evidence:   vec![],
                known_limitations:     None,
                provenance_status:     irrs_eval::declaration::ProvenanceStatus::Declared,
                submitted_to_gate:     true,
            }
        ],
        regional_prerequisite: RegionalPrerequisiteDecl {
            region_name:              "test_region".to_string(),
            extents_lower:            0.0,
            extents_upper:            1.0,
            extents_label:            "fraction".to_string(),
            orientation_preserved:    "ratio".to_string(),
            orientation_discarded:    "magnitude".to_string(),
            admission_rule_name:      "test_rule".to_string(),
            admission_rule_description: "Test admission rule.".to_string(),
            interior_locus_verified:  true,
        },
        mappings: vec![
            irrs_eval::declaration::MappingDeclaration {
                name:               "test_mapping".to_string(),
                input_description:  "y".to_string(),
                output_description: "z".to_string(),
                preserves:          "order".to_string(),
                discards:           "magnitude".to_string(),
            }
        ],
    };

    let real_digest = "aabbccdd1234567890aabbccdd1234567890aabbccdd1234567890aabbccdd12";
    let mut record = Evaluator::evaluate(decl, real_digest.to_string());

    assert_eq!(record.status, EvaluationStatus::PassedAwaitingAuthorization);

    // Construct an authorization with a DIFFERENT digest — simulating an
    // authorization that was issued for an older version of the declaration.
    let tampered_digest = "0000000000000000000000000000000000000000000000000000000000000000";
    assert_ne!(tampered_digest, real_digest, "Test setup: digests must differ");

    let auth = make_authorization(&record.record_id, tampered_digest);
    authorization::apply(&mut record, auth);

    // Authorization must not have been applied.
    assert_eq!(
        record.status, EvaluationStatus::PassedAwaitingAuthorization,
        "Status must remain PassedAwaitingAuthorization when digest does not match"
    );
    assert!(
        record.authorization.is_none(),
        "Authorization must not be stored when declaration_digest does not match"
    );
    assert!(
        record.evaluation_notes.iter().any(|n| n.contains("Authorization REJECTED")),
        "evaluation_notes must contain an 'Authorization REJECTED' entry"
    );
    assert!(
        record.evaluation_notes.iter().any(|n| n.contains("declaration_digest mismatch")),
        "Rejection note must mention 'declaration_digest mismatch'"
    );
}

// ── Test 3: Verified-without-evidence is flagged ─────────────────────────────

#[test]
fn test_verified_without_evidence_flagged() {
    use irrs_eval::declaration::{
        Declaration, RegionalPrerequisiteDecl, InputDeclaration,
        InputClassification, ProvenanceStatus,
    };
    use irrs_eval::evaluator::Evaluator;
    use irrs_eval::record::ProvenanceAssessment;

    // Build a declaration where an input claims "verified" provenance
    // but supplies no evidence references.
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "provenance_topology".to_string(),
        version:     "0.0.1".to_string(),
        description: Some("Declaration with inconsistent provenance claim.".to_string()),
        inputs: vec![
            InputDeclaration {
                name:                  "sensor_reading".to_string(),
                classification:        InputClassification::PrimaryObservable,
                source_identity:       "claimed_sensor".to_string(),
                acquisition_method:    "Direct sensor read".to_string(),
                transformation_history: None,
                relationship_to_problem: "Primary observable".to_string(),
                // Claims "verified" but provides NO evidence references.
                supporting_evidence:   vec![],
                known_limitations:     None,
                provenance_status:     ProvenanceStatus::Verified,
                submitted_to_gate:     true,
            }
        ],
        regional_prerequisite: RegionalPrerequisiteDecl {
            region_name:              "test_region".to_string(),
            extents_lower:            0.0,
            extents_upper:            1.0,
            extents_label:            "fraction".to_string(),
            orientation_preserved:    "ratio".to_string(),
            orientation_discarded:    "magnitude".to_string(),
            admission_rule_name:      "test_rule".to_string(),
            admission_rule_description: "Test admission rule.".to_string(),
            interior_locus_verified:  true,
        },
        mappings: vec![
            irrs_eval::declaration::MappingDeclaration {
                name:               "test_mapping".to_string(),
                input_description:  "sensor_reading".to_string(),
                output_description: "output".to_string(),
                preserves:          "order".to_string(),
                discards:           "magnitude".to_string(),
            }
        ],
    };

    let record = Evaluator::evaluate(decl, "test-digest-provenance".to_string());

    // The evaluation should pass (inconsistent claim does not fail the evaluation).
    assert_eq!(
        record.status, EvaluationStatus::PassedAwaitingAuthorization,
        "An inconsistent provenance claim does not fail evaluation; it is flagged"
    );

    // The input inventory must flag the inconsistency.
    let sensor_item = record.input_inventory
        .iter()
        .find(|i| i.name == "sensor_reading")
        .expect("sensor_reading must appear in input_inventory");

    match &sensor_item.evaluator_provenance_assessment {
        ProvenanceAssessment::InconsistentClaim { reason } => {
            assert!(
                reason.contains("verified") || reason.contains("evidence"),
                "Inconsistency reason must mention 'verified' or 'evidence'; got: {}", reason
            );
        }
        ProvenanceAssessment::ConsistentClaim => {
            panic!(
                "Expected InconsistentClaim for Verified provenance with no evidence; \
                 got ConsistentClaim"
            );
        }
    }

    // The provenance summary must count one inconsistent claim.
    assert_eq!(
        record.provenance_summary.inconsistent_claims, 1,
        "provenance_summary.inconsistent_claims must be 1"
    );

    // The evaluator_note must open with the required disclaimer.
    assert!(
        record.provenance_summary.evaluator_note.contains(
            "The evaluator records submitter claims. It does not independently verify"
        ),
        "evaluator_note must begin with the evaluator-limits disclaimer"
    );

    // The evaluation_notes must contain a PROVENANCE INCONSISTENCY entry.
    assert!(
        record.evaluation_notes.iter().any(|n| n.contains("PROVENANCE INCONSISTENCY")),
        "evaluation_notes must contain a 'PROVENANCE INCONSISTENCY' entry for sensor_reading"
    );
}

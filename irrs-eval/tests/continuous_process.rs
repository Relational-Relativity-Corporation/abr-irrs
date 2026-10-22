/// Regression tests for the Continuous Relational Process Rule.
///
/// Governing requirements under test:
///   1. Every evaluation enters an ongoing relational process. The first
///      admitted observation begins the evaluation record, not the physical
///      process.
///   2. No synthetic zero position, zero input, cold-start state, or fabricated
///      prior relational result is permitted.
///   3. Actual observed zero values remain admissible.
///   4. Persistence follows the established Kernel V8 process including R and
///      its prerequisites.
///   5. Where prior relational evidence is unavailable, report as OPEN. Do not
///      fabricate it or prevent otherwise admissible calculations.
///   6. Preserve mathematical provenance from observable relations through
///      declared M and Kernel V8 operations back to observational comparison.
///
/// Test coverage:
///   test_nonzero_initial_observation_admitted
///   test_observed_zero_admissible
///   test_missing_prior_evidence_reported_as_open
///   test_synthetic_initialization_rejected

use irrs_eval::{
    declaration::{
        Declaration, InputDeclaration, InputClassification, ProvenanceStatus,
        RegionalPrerequisiteDecl, MappingDeclaration,
        PersistenceDeclaration, PriorEvidenceStatus,
    },
    evaluator::Evaluator,
    record::{EvaluationStatus, PriorEvidenceReport},
};

// ── Shared fixture helpers ────────────────────────────────────────────────────

/// A minimal valid regional prerequisite with interior locus verified.
fn valid_prerequisite() -> RegionalPrerequisiteDecl {
    RegionalPrerequisiteDecl {
        region_name:               "actuator_output_locus".to_string(),
        extents_lower:             -1.0,
        extents_upper:              1.0,
        extents_label:             "normalized_torque".to_string(),
        orientation_preserved:     "relational direction of torque change".to_string(),
        orientation_discarded:     "absolute torque magnitude".to_string(),
        admission_rule_name:       "interior_locus_rule".to_string(),
        admission_rule_description:
            "Both LIDAR and torque sensor measure at the same physical output locus v_out, \
             constituting an interior point without fabrication.".to_string(),
        interior_locus_verified:   true,
    }
}

/// A minimal valid mapping entry.
fn valid_mapping(name: &str) -> MappingDeclaration {
    MappingDeclaration {
        name:               name.to_string(),
        input_description:  "Sensor observations at v_out".to_string(),
        output_description: "Relational result R after A→B→R".to_string(),
        preserves:          "directed relational difference and its accumulation".to_string(),
        discards:           "absolute sensor magnitude".to_string(),
    }
}

/// A PrimaryObservable sensor input with declared provenance.
fn sensor_input(name: &str, description: &str) -> InputDeclaration {
    InputDeclaration {
        name:                    name.to_string(),
        classification:          InputClassification::PrimaryObservable,
        source_identity:         "hardware sensor on v_out".to_string(),
        acquisition_method:      "direct sensor read at evaluation window open".to_string(),
        transformation_history:  None,
        relationship_to_problem: description.to_string(),
        supporting_evidence:     vec![],
        known_limitations:       None,
        provenance_status:       ProvenanceStatus::Declared,
        submitted_to_gate:       true,
    }
}

/// Standard Kernel V8 persistence operator path.
fn kernel_v8_persistence_path() -> Vec<String> {
    vec![
        "A_persistence".to_string(),
        "B_persistence".to_string(),
        "R_persistence".to_string(),
    ]
}

// ── Test 1: Nonzero initial observation is admitted ───────────────────────────

/// A nonzero sensor reading on first evaluation window open is admissible.
///
/// The actuator is already running. The first observation acquired by the
/// evaluation window is an actual nonzero sensor value. This is a
/// PrimaryObservable input. With prior relational evidence declared as
/// available, evaluation must pass.
///
/// Requirement 1: first admitted observation begins the evaluation record.
/// Requirement 4: persistence path declared per Kernel V8.
#[test]
fn test_nonzero_initial_observation_admitted() {
    let decl = Declaration {
        domain:      "actuator-lidar-torque".to_string(),
        topology_id: "continuous_relational_process_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: Some(
            "Continuous relational process: actuator, LIDAR, torque sensor. \
             Evaluation window opens onto ongoing process. \
             First observation is an actual nonzero sensor reading.".to_string()
        ),
        persistence: Some(PersistenceDeclaration {
            process_entry:
                "Evaluation window opens onto an ongoing relational process. \
                 The actuator has been operating before the evaluation record begins. \
                 The first admitted observation is actual sensor data from v_out; \
                 it begins the evaluation record, not the physical process.".to_string(),
            prior_evidence: PriorEvidenceStatus::Available,
            persistence_path: kernel_v8_persistence_path(),
        }),
        inputs: vec![
            sensor_input(
                "lidar_displacement_v_out",
                "LIDAR displacement measurement at actuator output locus v_out. \
                 Nonzero at evaluation window open — actuator is already in motion.",
            ),
            sensor_input(
                "torque_sensor_v_out",
                "Torque sensor reading at actuator output locus v_out. \
                 Actual observed value; may be nonzero.",
            ),
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping("A_operator_then_B_then_R")],
    };

    let record = Evaluator::evaluate(decl, "test-digest-nonzero-initial".to_string());

    assert_eq!(
        record.status, EvaluationStatus::PassedAwaitingAuthorization,
        "Nonzero initial observation must be admitted: actual sensor data is admissible \
         regardless of magnitude"
    );

    let cp = record.continuous_process
        .as_ref()
        .expect("continuous_process report must be present");

    assert!(cp.process_entry_declared, "process_entry_declared must be true");
    assert_eq!(
        cp.prior_evidence, PriorEvidenceReport::Available,
        "prior_evidence must be Available when declared as such"
    );
    assert!(
        cp.open_conditions.is_empty(),
        "No OPEN conditions expected when prior evidence is available"
    );
    assert_eq!(
        cp.persistence_path,
        kernel_v8_persistence_path(),
        "Persistence path must match the declared Kernel V8 path"
    );
}

// ── Test 2: Observed zero is admissible ───────────────────────────────────────

/// An actual sensor reading of zero is admissible.
///
/// Requirement 3: actual observed zero values remain admissible.
///
/// A sensor that genuinely reads 0.0 is a PrimaryObservable. Its value is
/// determined by the physical world, not fabricated by the Generator. This is
/// fundamentally different from a synthetic zero used as a prior state.
/// The evaluator must not refuse it.
#[test]
fn test_observed_zero_admissible() {
    let decl = Declaration {
        domain:      "actuator-lidar-torque".to_string(),
        topology_id: "continuous_relational_process_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: Some(
            "Observed zero test: sensor reads 0.0 at window open. \
             This is an actual observed value, not a fabricated prior. \
             It must be admitted as PrimaryObservable.".to_string()
        ),
        persistence: Some(PersistenceDeclaration {
            process_entry:
                "Evaluation window opens onto an ongoing relational process. \
                 Sensor at v_out reads 0.0 — the actuator output happens to be \
                 at rest at this instant. This is an actual observed value acquired \
                 from the physical sensor, not a fabricated zero used as initialization.".to_string(),
            prior_evidence: PriorEvidenceStatus::Available,
            persistence_path: kernel_v8_persistence_path(),
        }),
        inputs: vec![
            // Sensor actually reads zero. Still PrimaryObservable — the value
            // comes from the sensor, not from a Generator fabrication.
            InputDeclaration {
                name:                    "lidar_displacement_v_out".to_string(),
                classification:          InputClassification::PrimaryObservable,
                source_identity:         "LIDAR unit at v_out".to_string(),
                acquisition_method:
                    "Direct sensor read; reading is 0.0 at this evaluation instant. \
                     The sensor is operational and reporting an actual displacement \
                     of zero, not a fabricated initialization value.".to_string(),
                transformation_history:  None,
                relationship_to_problem:
                    "Actual observed displacement at v_out. Value is 0.0 because \
                     the actuator output is momentarily at the reference position. \
                     This is observationally determined, not stipulated.".to_string(),
                supporting_evidence:     vec![],
                known_limitations:       None,
                provenance_status:       ProvenanceStatus::Declared,
                submitted_to_gate:       true,
            },
            sensor_input(
                "torque_sensor_v_out",
                "Torque at v_out; also reads near zero when actuator is at rest.",
            ),
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping("A_operator_then_B_then_R")],
    };

    let record = Evaluator::evaluate(decl, "test-digest-observed-zero".to_string());

    // An actual observed zero must pass. The evaluator does not distinguish
    // between zero and nonzero sensor readings for admissibility.
    assert_eq!(
        record.status, EvaluationStatus::PassedAwaitingAuthorization,
        "An actual observed sensor reading of 0.0 must be admitted. \
         Observed zeros are PrimaryObservable, not synthetic."
    );

    let cp = record.continuous_process
        .as_ref()
        .expect("continuous_process report must be present");

    assert!(cp.process_entry_declared, "process_entry_declared must be true");
    assert_eq!(
        cp.prior_evidence, PriorEvidenceReport::Available,
        "prior_evidence must be Available; observed zeros do not constitute \
         synthetic initialization"
    );
    assert!(
        cp.open_conditions.is_empty(),
        "No OPEN conditions expected for an actual observed zero"
    );

    // All inputs must be classified as PrimaryObservable — including the one
    // that happens to read 0.0. Classification is by acquisition method, not value.
    let lidar_item = record.input_inventory
        .iter()
        .find(|i| i.name == "lidar_displacement_v_out")
        .expect("lidar_displacement_v_out must appear in inventory");

    assert!(
        matches!(lidar_item.classification, irrs_eval::record::InventoryKind::PrimaryObservable),
        "A sensor reading of 0.0 must be classified as PrimaryObservable, not AbstractInput"
    );
}

// ── Test 3: Missing prior evidence is reported as OPEN ────────────────────────

/// When prior relational evidence is unavailable at evaluation window open,
/// the evaluator reports the dependency as OPEN and does not prevent otherwise
/// admissible calculations.
///
/// Requirement 5: Where prior relational evidence is unavailable, report as OPEN.
/// Do not fabricate it or prevent otherwise admissible calculations.
///
/// This covers the case where the evaluation record has just started: the
/// process is running, the first observation is being admitted, but no prior
/// relational result (from a previous evaluation cycle) yet exists.
#[test]
fn test_missing_prior_evidence_reported_as_open() {
    let decl = Declaration {
        domain:      "actuator-lidar-torque".to_string(),
        topology_id: "continuous_relational_process_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: Some(
            "Prior evidence OPEN test: evaluation record has just started. \
             No prior relational result exists yet. \
             The evaluator must report OPEN, not fabricate a prior.".to_string()
        ),
        persistence: Some(PersistenceDeclaration {
            process_entry:
                "Evaluation window opens onto an ongoing relational process. \
                 This is the first admitted observation; the evaluation record begins now. \
                 No prior relational result from a previous evaluation cycle is available. \
                 The prior relational evidence is OPEN.".to_string(),
            prior_evidence: PriorEvidenceStatus::Open,
            persistence_path: kernel_v8_persistence_path(),
        }),
        inputs: vec![
            sensor_input(
                "lidar_displacement_v_out",
                "First LIDAR observation admitted to the evaluation record.",
            ),
            sensor_input(
                "torque_sensor_v_out",
                "First torque observation admitted to the evaluation record.",
            ),
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping("A_operator_then_B_then_R")],
    };

    let record = Evaluator::evaluate(decl, "test-digest-open-prior".to_string());

    // OPEN prior evidence must NOT fail evaluation.
    // Requirement: report as OPEN, do not prevent otherwise admissible calculations.
    assert_eq!(
        record.status, EvaluationStatus::PassedAwaitingAuthorization,
        "An OPEN prior evidence condition must not fail evaluation. \
         Otherwise admissible calculations proceed."
    );

    let cp = record.continuous_process
        .as_ref()
        .expect("continuous_process report must be present");

    assert!(cp.process_entry_declared, "process_entry_declared must be true");

    assert_eq!(
        cp.prior_evidence, PriorEvidenceReport::Open,
        "prior_evidence report must be Open when declared as open"
    );

    // At least one OPEN condition must be recorded.
    assert!(
        !cp.open_conditions.is_empty(),
        "open_conditions must contain at least one entry for an OPEN prior"
    );

    // The OPEN condition must be documented in evaluation_notes.
    assert!(
        record.evaluation_notes.iter().any(|n| n.contains("OPEN")),
        "evaluation_notes must contain an OPEN condition entry"
    );

    // The OPEN condition must not say the value is zero.
    for cond in &cp.open_conditions {
        assert!(
            !cond.to_lowercase().contains("zero") || cond.contains("not zero"),
            "OPEN condition must not equate unavailable prior with zero: '{}'", cond
        );
    }
}

// ── Test 4: Synthetic initialization is rejected ─────────────────────────────

/// A declaration that claims synthetic initialization is rejected.
///
/// Requirement 2: No synthetic zero position, zero input, cold-start state,
/// or fabricated prior relational result is permitted.
///
/// If a submitter declares `prior_evidence: synthetic`, the evaluator must
/// fail the evaluation. Authorization cannot override a failed evaluation.
/// (Authorization rule 1 is preserved.)
#[test]
fn test_synthetic_initialization_rejected() {
    let decl = Declaration {
        domain:      "actuator-lidar-torque".to_string(),
        topology_id: "continuous_relational_process_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: Some(
            "Synthetic initialization rejection test. \
             Declares a fabricated cold-start prior. Must fail.".to_string()
        ),
        persistence: Some(PersistenceDeclaration {
            process_entry:
                "Attempting to use a synthetic zero as the initial prior relational result. \
                 This is prohibited under the Continuous Relational Process Rule.".to_string(),
            // PROHIBITED: synthetic prior declared.
            prior_evidence: PriorEvidenceStatus::Synthetic,
            persistence_path: kernel_v8_persistence_path(),
        }),
        inputs: vec![
            // Even with valid inputs, the synthetic prior fails the whole declaration.
            sensor_input(
                "lidar_displacement_v_out",
                "LIDAR reading at v_out.",
            ),
            sensor_input(
                "torque_sensor_v_out",
                "Torque reading at v_out.",
            ),
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping("A_operator_then_B_then_R")],
    };

    let record = Evaluator::evaluate(decl, "test-digest-synthetic-init".to_string());

    // The synthetic prior must fail evaluation.
    assert_eq!(
        record.status, EvaluationStatus::Failed,
        "A declaration with synthetic initialization must fail evaluation"
    );

    // The failure reason must be present and must name the violation.
    let reason = record.failure_reason
        .as_deref()
        .expect("failure_reason must be present on a failed evaluation");

    assert!(
        reason.to_lowercase().contains("synthetic") || reason.to_lowercase().contains("prohibited"),
        "failure_reason must identify the synthetic initialization as the cause: '{}'", reason
    );

    // The evaluation_notes must contain the VIOLATION entry.
    assert!(
        record.evaluation_notes.iter().any(|n|
            n.contains("PROHIBITED") || n.contains("SYNTHETIC") || n.contains("VIOLATION")
        ),
        "evaluation_notes must document the synthetic initialization violation"
    );

    // The continuous_process report must record a SyntheticViolation.
    let cp = record.continuous_process
        .as_ref()
        .expect("continuous_process report must be present");

    assert_eq!(
        cp.prior_evidence, PriorEvidenceReport::SyntheticViolation,
        "continuous_process.prior_evidence must be SyntheticViolation"
    );

    // Verify that authorization cannot override the failure (authorization rule 1).
    use irrs_eval::{authorization, record::OriginAuthorization};

    let auth = OriginAuthorization {
        record_id:          record.record_id.clone(),
        declaration_digest: record.declaration_digest.clone(),
        authorized_by:      "test-origin".to_string(),
        authorized_at:      "2026-10-22".to_string(),
        statement:          "Attempting to authorize a failed evaluation.".to_string(),
    };

    let mut record_with_auth = record.clone();
    authorization::apply(&mut record_with_auth, auth);

    assert_eq!(
        record_with_auth.status, EvaluationStatus::Failed,
        "Authorization must not override a failed evaluation \
         (authorization enforcement rule 1)"
    );
    assert!(
        record_with_auth.authorization.is_none(),
        "Authorization must not be stored on a failed record"
    );
}

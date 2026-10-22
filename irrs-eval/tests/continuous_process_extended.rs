/// Extended unit tests: UT-05 through UT-15, plus adversarial variations.
///
/// Generator Directive — IRRS Unit Test and Enforcement Verification
/// Baseline commit: eba9ef6352975f943a429d9af548478220ccfcd6
///
/// These tests verify enforcement of the Continuous Relational Process Rule
/// beyond the four basic cases already in continuous_process.rs.
///
/// | Test  | Input condition                                        | Expected result                                            |
/// |-------|--------------------------------------------------------|------------------------------------------------------------|
/// | UT-05 | Persistence declaration missing                        | FAILED: not eligible for authorization                     |
/// | UT-06 | Persistence operator path empty                        | FAILED: not a valid persistence path                       |
/// | UT-07 | Persistence operator path incomplete                   | FAILED: not a valid persistence path                       |
/// | UT-08 | Persistence operators in incorrect order               | FAILED: not a valid persistence path                       |
/// | UT-09 | Correct persistence path declared                      | PASS: structurally valid, subject to other prerequisites   |
/// | UT-10 | Prior evidence claimed available without traceable     | OPEN: availability not independently established           |
///         | evidence                                               |                                                            |
/// | UT-11 | Prior evidence available with valid traceable evidence | PASS: eligible for admission                               |
/// | UT-14 | First valid relational result for subsequent           | PASS: provenance preserved; no fabricated intermediate     |
///         | persistence                                            |                                                            |
/// | UT-15 | Valid observations with unrelated OPEN persistence     | PASS: admissible operations available; OPEN dependency     |
///         | dependency                                             | reported                                                   |
///
/// Adversarial cases are grouped with the test they attack.

use irrs_eval::{
    declaration::{
        Declaration, InputDeclaration, InputClassification, ProvenanceStatus,
        RegionalPrerequisiteDecl, MappingDeclaration,
        PersistenceDeclaration, PriorEvidenceStatus, EvidenceRef,
    },
    evaluator::Evaluator,
    record::{EvaluationStatus, PriorEvidenceReport},
};

// ── Shared fixtures ───────────────────────────────────────────────────────────

fn valid_prerequisite() -> RegionalPrerequisiteDecl {
    RegionalPrerequisiteDecl {
        region_name:               "actuator_output_locus_v_out".to_string(),
        extents_lower:             -1.0,
        extents_upper:              1.0,
        extents_label:             "normalized_output".to_string(),
        orientation_preserved:     "relational direction of change".to_string(),
        orientation_discarded:     "absolute sensor magnitude".to_string(),
        admission_rule_name:       "interior_locus_rule_v_out".to_string(),
        admission_rule_description:
            "LIDAR and torque sensor co-located at v_out constitute a verifiable \
             interior point.".to_string(),
        interior_locus_verified:   true,
    }
}

fn valid_mapping() -> MappingDeclaration {
    MappingDeclaration {
        name:               "A_then_B_then_R".to_string(),
        input_description:  "Sensor observations at v_out".to_string(),
        output_description: "Relational result R".to_string(),
        preserves:          "relational direction and accumulation".to_string(),
        discards:           "absolute magnitude".to_string(),
    }
}

fn sensor_input(name: &str, desc: &str) -> InputDeclaration {
    InputDeclaration {
        name:                    name.to_string(),
        classification:          InputClassification::PrimaryObservable,
        source_identity:         "hardware sensor at v_out".to_string(),
        acquisition_method:      "direct sensor read".to_string(),
        transformation_history:  None,
        relationship_to_problem: desc.to_string(),
        supporting_evidence:     vec![],
        known_limitations:       None,
        provenance_status:       ProvenanceStatus::Declared,
        submitted_to_gate:       true,
    }
}

fn kernel_v8_path() -> Vec<String> {
    vec![
        "A_persistence".to_string(),
        "B_persistence".to_string(),
        "R_persistence".to_string(),
    ]
}

fn eval_record_evidence(id: &str) -> EvidenceRef {
    EvidenceRef {
        id:          id.to_string(),
        description: format!("EvaluationRecord produced by irrs-eval for cycle {}. \
                              Contains R result from Phase 1 A→B→R chain, produced \
                              by AdmissionGate, not fabricated.", id),
        kind:        "evaluation_record".to_string(),
    }
}

/// Minimal declaration that passes all checks except persistence.
/// Used as a base for UT-05.
fn base_decl_no_persistence() -> Declaration {
    Declaration {
        domain:      "test".to_string(),
        topology_id: "continuous_test_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: Some("Base declaration without persistence block.".to_string()),
        persistence: None,
        inputs: vec![
            sensor_input("lidar_v_out", "LIDAR at v_out"),
            sensor_input("torque_v_out", "torque at v_out"),
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    }
}

/// Valid persistence declaration with the correct Kernel V8 path and OPEN prior.
fn open_persistence() -> PersistenceDeclaration {
    PersistenceDeclaration {
        process_entry:
            "Evaluation window opens onto an ongoing relational process. \
             First admitted observation begins the evaluation record.".to_string(),
        prior_evidence:      PriorEvidenceStatus::Open,
        prior_evidence_refs: vec![],
        persistence_path:    kernel_v8_path(),
    }
}

// ── UT-05: Persistence declaration missing ────────────────────────────────────

/// UT-05: A declaration with no persistence block must fail evaluation.
/// It is not eligible for Origin authorization until the required persistence
/// declaration is supplied.
///
/// Governing rule: every evaluation must declare its process entry posture.
/// A declaration that omits persistence cannot be assessed for compliance with
/// the Continuous Relational Process Rule.
#[test]
fn test_ut05_persistence_missing_fails() {
    let decl = base_decl_no_persistence();
    let record = Evaluator::evaluate(decl, "test-digest-ut05".to_string());

    assert_eq!(
        record.status, EvaluationStatus::Failed,
        "UT-05: A declaration without a persistence block must fail. \
         It is not eligible for authorization."
    );

    let reason = record.failure_reason.as_deref()
        .expect("failure_reason must be present");

    assert!(
        reason.to_lowercase().contains("persistence"),
        "UT-05: failure_reason must name the missing persistence declaration: '{}'", reason
    );
}

/// Adversarial UT-05: Descriptive text in the process_entry of an otherwise
/// absent persistence block cannot grant authorization.
/// (Persistence is structurally absent; the evaluator must fail, not read prose.)
///
/// This is tested implicitly by the above: the evaluator checks for the presence
/// of the structured `persistence` field, not for prose assertions.
/// A separate adversarial test using an *empty* persistence_path tests UT-06.

// ── UT-06: Persistence path empty ────────────────────────────────────────────

/// UT-06: A persistence block with an empty operator path must fail.
/// An empty path is not a valid persistence path under Kernel V8.
#[test]
fn test_ut06_empty_persistence_path_fails() {
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "continuous_test_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: None,
        persistence: Some(PersistenceDeclaration {
            process_entry: "Test process entry.".to_string(),
            prior_evidence: PriorEvidenceStatus::Open,
            prior_evidence_refs: vec![],
            // UT-06: empty path.
            persistence_path: vec![],
        }),
        inputs: vec![
            sensor_input("lidar_v_out", "LIDAR at v_out"),
            sensor_input("torque_v_out", "torque at v_out"),
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut06".to_string());

    assert_eq!(
        record.status, EvaluationStatus::Failed,
        "UT-06: An empty persistence path must fail evaluation."
    );

    let reason = record.failure_reason.as_deref().expect("failure_reason required");
    assert!(
        reason.to_lowercase().contains("empty") || reason.to_lowercase().contains("path"),
        "UT-06: failure_reason must identify the empty persistence path: '{}'", reason
    );

    let cp = record.continuous_process.as_ref().expect("continuous_process required");
    assert!(
        !cp.persistence_path_valid,
        "UT-06: persistence_path_valid must be false for an empty path"
    );
}

/// Adversarial UT-06: A persistence block that declares the correct path text
/// in process_entry prose but has an empty persistence_path array must still fail.
/// Descriptive text is not a substitute for structured field declarations.
#[test]
fn test_ut06_adv_prose_claim_not_a_persistence_path() {
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "continuous_test_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: None,
        persistence: Some(PersistenceDeclaration {
            // Claims compliance in prose — but the structured path is empty.
            process_entry:
                "Persistence path is A_persistence then B_persistence then R_persistence \
                 per Kernel V8.".to_string(),
            prior_evidence: PriorEvidenceStatus::Open,
            prior_evidence_refs: vec![],
            // Structural path is empty — the prose claim above must not substitute.
            persistence_path: vec![],
        }),
        inputs: vec![sensor_input("s", "sensor")],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut06-adv".to_string());

    assert_eq!(
        record.status, EvaluationStatus::Failed,
        "Adversarial UT-06: descriptive text asserting a valid path must not \
         substitute for a structured persistence_path declaration."
    );
}

// ── UT-07: Persistence path incomplete ───────────────────────────────────────

/// UT-07: A persistence path that names some but not all required operators
/// must fail. A partial path is not a valid Kernel V8 persistence path.
#[test]
fn test_ut07_incomplete_persistence_path_fails() {
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "continuous_test_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: None,
        persistence: Some(PersistenceDeclaration {
            process_entry: "Test.".to_string(),
            prior_evidence: PriorEvidenceStatus::Open,
            prior_evidence_refs: vec![],
            // UT-07: missing R_persistence.
            persistence_path: vec![
                "A_persistence".to_string(),
                "B_persistence".to_string(),
            ],
        }),
        inputs: vec![
            sensor_input("lidar_v_out", "LIDAR"),
            sensor_input("torque_v_out", "torque"),
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut07".to_string());

    assert_eq!(
        record.status, EvaluationStatus::Failed,
        "UT-07: An incomplete persistence path must fail."
    );

    let reason = record.failure_reason.as_deref().expect("failure_reason required");
    assert!(
        reason.to_lowercase().contains("incomplete") || reason.to_lowercase().contains("missing"),
        "UT-07: failure_reason must identify the incomplete path: '{}'", reason
    );

    let cp = record.continuous_process.as_ref().expect("continuous_process required");
    assert!(
        !cp.persistence_path_valid,
        "UT-07: persistence_path_valid must be false for an incomplete path"
    );
}

/// Adversarial UT-07: A path missing only R_persistence (the final stage)
/// must still fail. Omitting the antisymmetric adjustment step is not admissible.
#[test]
fn test_ut07_adv_missing_final_stage_fails() {
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "continuous_test_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: None,
        persistence: Some(PersistenceDeclaration {
            process_entry: "Missing R step.".to_string(),
            prior_evidence: PriorEvidenceStatus::Open,
            prior_evidence_refs: vec![],
            // A and B present, R missing.
            persistence_path: vec!["A_persistence".to_string(), "B_persistence".to_string()],
        }),
        inputs: vec![sensor_input("s", "sensor")],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut07-adv".to_string());
    assert_eq!(record.status, EvaluationStatus::Failed,
        "Adversarial UT-07: path missing R_persistence must fail.");
}

// ── UT-08: Persistence operators in incorrect order ───────────────────────────

/// UT-08: Persistence operators present but not in the required A→B→R sequence
/// must fail. A correct operator name is not evidence of correct operator execution;
/// order is part of the declaration.
#[test]
fn test_ut08_wrong_order_persistence_path_fails() {
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "continuous_test_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: None,
        persistence: Some(PersistenceDeclaration {
            process_entry: "Test.".to_string(),
            prior_evidence: PriorEvidenceStatus::Open,
            prior_evidence_refs: vec![],
            // UT-08: all three present but B before A.
            persistence_path: vec![
                "B_persistence".to_string(),
                "A_persistence".to_string(),
                "R_persistence".to_string(),
            ],
        }),
        inputs: vec![
            sensor_input("lidar_v_out", "LIDAR"),
            sensor_input("torque_v_out", "torque"),
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut08".to_string());

    assert_eq!(
        record.status, EvaluationStatus::Failed,
        "UT-08: All operators present but in wrong order must fail."
    );

    let reason = record.failure_reason.as_deref().expect("failure_reason required");
    assert!(
        reason.to_lowercase().contains("order") || reason.to_lowercase().contains("sequence"),
        "UT-08: failure_reason must identify the incorrect order: '{}'", reason
    );

    let cp = record.continuous_process.as_ref().expect("continuous_process required");
    assert!(
        !cp.persistence_path_valid,
        "UT-08: persistence_path_valid must be false for wrong order"
    );
}

/// Adversarial UT-08: Providing the correct operator names in the process_entry
/// prose but the wrong order in the persistence_path array must still fail.
#[test]
fn test_ut08_adv_correct_names_wrong_order_fails() {
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "continuous_test_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: None,
        persistence: Some(PersistenceDeclaration {
            // Correct sequence described in prose.
            process_entry:
                "Persistence follows A_persistence then B_persistence then R_persistence.".to_string(),
            prior_evidence: PriorEvidenceStatus::Open,
            prior_evidence_refs: vec![],
            // Structural array has wrong order.
            persistence_path: vec![
                "R_persistence".to_string(),
                "B_persistence".to_string(),
                "A_persistence".to_string(),
            ],
        }),
        inputs: vec![sensor_input("s", "sensor")],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut08-adv".to_string());
    assert_eq!(
        record.status, EvaluationStatus::Failed,
        "Adversarial UT-08: correct names in prose but wrong order in array must fail. \
         A declaration that merely describes compliance must not be treated as evidence."
    );
}

// ── UT-09: Correct persistence path declared ──────────────────────────────────

/// UT-09: A declaration with the correct Kernel V8 persistence path and a valid
/// prior_evidence status passes structural validation, subject to other prerequisites.
#[test]
fn test_ut09_correct_persistence_path_passes() {
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "continuous_test_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: Some("UT-09: valid persistence path.".to_string()),
        persistence: Some(PersistenceDeclaration {
            process_entry:
                "Evaluation window opens onto ongoing process.".to_string(),
            prior_evidence:      PriorEvidenceStatus::Open,
            prior_evidence_refs: vec![],
            // UT-09: correct path.
            persistence_path:    kernel_v8_path(),
        }),
        inputs: vec![
            sensor_input("lidar_v_out", "LIDAR"),
            sensor_input("torque_v_out", "torque"),
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut09".to_string());

    assert_eq!(
        record.status, EvaluationStatus::PassedAwaitingAuthorization,
        "UT-09: Correct persistence path with valid prior_evidence must pass structural validation."
    );

    let cp = record.continuous_process.as_ref().expect("continuous_process required");
    assert!(
        cp.persistence_path_valid,
        "UT-09: persistence_path_valid must be true for the correct Kernel V8 path"
    );
    assert_eq!(
        cp.persistence_path, kernel_v8_path(),
        "UT-09: Declared path must be preserved in the report"
    );
}

// ── UT-10: Available without traceable evidence → OPEN ────────────────────────

/// UT-10: Declaring `prior_evidence: available` without supplying traceable
/// evidence references does not establish availability. The evaluator must
/// downgrade the claim to OPEN.
///
/// A declaration that merely asserts availability without evidence cannot be
/// treated as evidence of availability. The evaluator enforces this structurally.
#[test]
fn test_ut10_available_without_evidence_downgraded_to_open() {
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "continuous_test_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: Some("UT-10: available claimed, no evidence.".to_string()),
        persistence: Some(PersistenceDeclaration {
            process_entry:
                "Prior relational evidence is available from the previous cycle.".to_string(),
            // Claims available — but no evidence refs to support the claim.
            prior_evidence:      PriorEvidenceStatus::Available,
            prior_evidence_refs: vec![],    // ← no refs: claim unsubstantiated
            persistence_path:    kernel_v8_path(),
        }),
        inputs: vec![
            sensor_input("lidar_v_out", "LIDAR"),
            sensor_input("torque_v_out", "torque"),
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut10".to_string());

    // Must NOT fail — OPEN is not FAILED.
    assert_eq!(
        record.status, EvaluationStatus::PassedAwaitingAuthorization,
        "UT-10: An unsubstantiated 'available' claim must not fail evaluation. \
         It is downgraded to OPEN."
    );

    let cp = record.continuous_process.as_ref().expect("continuous_process required");

    assert_eq!(
        cp.prior_evidence, PriorEvidenceReport::Open,
        "UT-10: An 'available' claim without evidence refs must be reported as Open, \
         not Available. Availability not independently established."
    );

    assert!(
        !cp.open_conditions.is_empty(),
        "UT-10: OPEN condition must be recorded when availability is unsubstantiated"
    );

    // The evidence count must be zero.
    assert_eq!(
        cp.prior_evidence_refs_count, 0,
        "UT-10: prior_evidence_refs_count must be 0 when no refs were supplied"
    );
}

/// Adversarial UT-10: Asserting availability in the process_entry prose without
/// providing structured evidence refs must still be downgraded to OPEN.
/// Descriptive text claiming compliance is not evidence.
#[test]
fn test_ut10_adv_prose_availability_claim_not_evidence() {
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "continuous_test_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: None,
        persistence: Some(PersistenceDeclaration {
            // Elaborate prose claim of availability — but no structured refs.
            process_entry:
                "Prior relational evidence is fully available. The previous evaluation cycle \
                 completed successfully, R is known, PersistenceState::from_prior(E_1) has \
                 been called, and the prior result is mathematically verified and traceable \
                 through the Kernel V8 operators. Availability is established.".to_string(),
            prior_evidence:      PriorEvidenceStatus::Available,
            prior_evidence_refs: vec![],   // ← still empty
            persistence_path:    kernel_v8_path(),
        }),
        inputs: vec![sensor_input("s", "sensor")],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut10-adv".to_string());

    let cp = record.continuous_process.as_ref().expect("continuous_process required");

    assert_eq!(
        cp.prior_evidence, PriorEvidenceReport::Open,
        "Adversarial UT-10: elaborate prose claiming availability must still be \
         downgraded to Open when no structured evidence refs are supplied. \
         A declaration that merely describes compliance is not evidence of compliance."
    );
}

// ── UT-11: Available with valid traceable evidence ────────────────────────────

/// UT-11: When `prior_evidence: available` is declared WITH at least one
/// traceable evidence reference, the evaluator accepts the availability claim
/// and does not downgrade to OPEN.
///
/// The evaluator does not independently verify the contents of the referenced
/// evaluation record. It records what was declared and assesses structural
/// consistency. Independent verification is Origin's responsibility.
#[test]
fn test_ut11_available_with_evidence_passes() {
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "continuous_test_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: Some("UT-11: available with evidence.".to_string()),
        persistence: Some(PersistenceDeclaration {
            process_entry:
                "Prior relational evidence is available from cycle 001. \
                 R from cycle 001 is the input to A_persistence for this cycle.".to_string(),
            prior_evidence:      PriorEvidenceStatus::Available,
            // UT-11: evidence ref supplied.
            prior_evidence_refs: vec![eval_record_evidence("cycle-001")],
            persistence_path:    kernel_v8_path(),
        }),
        inputs: vec![
            sensor_input("lidar_v_out", "LIDAR"),
            sensor_input("torque_v_out", "torque"),
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut11".to_string());

    assert_eq!(
        record.status, EvaluationStatus::PassedAwaitingAuthorization,
        "UT-11: Available with evidence must pass evaluation."
    );

    let cp = record.continuous_process.as_ref().expect("continuous_process required");

    assert_eq!(
        cp.prior_evidence, PriorEvidenceReport::Available,
        "UT-11: Prior evidence must be reported as Available when refs are supplied."
    );

    assert!(
        cp.open_conditions.is_empty(),
        "UT-11: No OPEN conditions expected when prior evidence is supported by refs."
    );

    assert_eq!(
        cp.prior_evidence_refs_count, 1,
        "UT-11: prior_evidence_refs_count must reflect the supplied evidence ref."
    );
}

// ── UT-14: First valid result enables subsequent persistence ──────────────────

/// UT-14: When the first evaluation cycle completes and R is available, the
/// second evaluation cycle's declaration may declare `prior_evidence: available`
/// with a reference to the first cycle's EvaluationRecord.
///
/// The prior R input must be classified as DerivedValue (gate-produced, not
/// Generator-submitted). Provenance must trace back to the gate output, not to
/// a Generator-fabricated value.
///
/// This verifies: provenance preserved; no fabricated intermediate state.
#[test]
fn test_ut14_first_result_enables_subsequent_persistence() {
    // This is the declaration for CYCLE 2: prior evidence is now available
    // from cycle 1's completed evaluation.
    let decl = Declaration {
        domain:      "actuator-lidar-torque".to_string(),
        topology_id: "continuous_relational_process_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: Some(
            "UT-14: Cycle 2. Prior R from cycle 1 is available. \
             Provenance traces back to the gate's output, not to a fabrication.".to_string()
        ),
        persistence: Some(PersistenceDeclaration {
            process_entry:
                "Evaluation window opens onto an ongoing relational process. \
                 Cycle 1 has completed. R from cycle 1 is the prior relational result. \
                 It was produced by AdmissionGate (not Generator) and is traceable \
                 through the Kernel V8 A→B→R chain from actual sensor observations. \
                 No fabricated intermediate state exists.".to_string(),
            prior_evidence:      PriorEvidenceStatus::Available,
            prior_evidence_refs: vec![eval_record_evidence("cycle-001")],
            persistence_path:    kernel_v8_path(),
        }),
        inputs: vec![
            // Current cycle observations.
            sensor_input(
                "lidar_displacement_v_out",
                "LIDAR displacement at v_out for current cycle.",
            ),
            sensor_input(
                "torque_sensor_v_out",
                "Torque at v_out for current cycle.",
            ),
            // The prior R: DerivedValue produced by the gate in cycle 1.
            // Not submitted by the Generator. Not fabricated.
            InputDeclaration {
                name:             "R_prior_from_cycle_001".to_string(),
                classification:   InputClassification::DerivedValue,
                source_identity:
                    "AdmissionGate output from cycle 001 EvaluationRecord; \
                     PersistenceState::from_prior(E_1)".to_string(),
                acquisition_method:
                    "Carried forward from the gate's completed cycle 001 output. \
                     Produced by the gate's A→B→R chain from actual sensor observations \
                     at v_out. Not computed or submitted by the Generator.".to_string(),
                transformation_history: Some(
                    "cycle-001: lidar_v_out, torque_v_out → A → B → R → \
                     AdmittedObservation<R> (gate output) → PersistenceState::from_prior".to_string()
                ),
                relationship_to_problem:
                    "Prior relational result R. Input to A_persistence for this cycle. \
                     Provenance is fully traceable: O→M→D→A→B→R (cycle 1) → \
                     A_persistence→B_persistence→R_persistence (cycle 2).".to_string(),
                supporting_evidence: vec![eval_record_evidence("cycle-001")],
                known_limitations:   None,
                provenance_status:   ProvenanceStatus::Declared,
                // False: gate produces this; Generator does not submit it.
                submitted_to_gate: false,
            },
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut14".to_string());

    // Must pass — valid prior, valid observations, valid path.
    assert_eq!(
        record.status, EvaluationStatus::PassedAwaitingAuthorization,
        "UT-14: Second cycle with available prior from first cycle must pass evaluation."
    );

    let cp = record.continuous_process.as_ref().expect("continuous_process required");

    assert_eq!(
        cp.prior_evidence, PriorEvidenceReport::Available,
        "UT-14: prior_evidence must be Available when evidence refs are present."
    );

    assert!(cp.open_conditions.is_empty(),
        "UT-14: No OPEN conditions when prior evidence is available and supported.");

    // The prior R input must be classified as DerivedValue, not as AbstractInput.
    // DerivedValue: gate-produced. AbstractInput: fabricated. This is the key
    // provenance distinction.
    let r_prior = record.input_inventory.iter()
        .find(|i| i.name == "R_prior_from_cycle_001")
        .expect("R_prior_from_cycle_001 must appear in inventory");

    assert!(
        matches!(r_prior.classification, irrs_eval::record::InventoryKind::DerivedValue),
        "UT-14: Prior R must be classified as DerivedValue (gate-produced), not AbstractInput."
    );

    // Must not be submitted_to_gate (the gate produces it, the Generator does not submit it).
    assert!(
        !r_prior.submitted_to_gate,
        "UT-14: The gate-produced prior R must not be marked as submitted_to_gate."
    );
}

/// Adversarial UT-14: Attempting to supply the prior R as an AbstractInput
/// (stipulated without observational basis) must still be classified as such.
/// The evaluator should note the classification; Origin must reject it.
///
/// This does not automatically fail evaluation (the evaluator records classifications,
/// it does not prohibit AbstractInput per se). But the classification is recorded
/// and a PROVENANCE INCONSISTENCY note is added when appropriate.
/// The evaluator's note must distinguish DerivedValue from AbstractInput.
#[test]
fn test_ut14_adv_abstract_prior_classified_correctly() {
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "continuous_test_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: None,
        persistence: Some(PersistenceDeclaration {
            process_entry: "Test.".to_string(),
            prior_evidence:      PriorEvidenceStatus::Available,
            prior_evidence_refs: vec![eval_record_evidence("cycle-001")],
            persistence_path:    kernel_v8_path(),
        }),
        inputs: vec![
            sensor_input("lidar_v_out", "LIDAR"),
            sensor_input("torque_v_out", "torque"),
            // Adversarial: prior R submitted as AbstractInput, not DerivedValue.
            InputDeclaration {
                name:             "R_prior_abstract".to_string(),
                // WRONG classification: AbstractInput = no observational basis.
                classification:   InputClassification::AbstractInput,
                source_identity:  "Generator stipulation".to_string(),
                acquisition_method:
                    "Stipulated as zero to initialize the persistence chain.".to_string(),
                transformation_history: None,
                relationship_to_problem:
                    "Attempting to use AbstractInput as prior R.".to_string(),
                supporting_evidence: vec![],
                known_limitations:   None,
                provenance_status:   ProvenanceStatus::Declared,
                submitted_to_gate:   true,  // Adversarial: trying to submit it.
            },
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut14-adv".to_string());

    // The evaluator records classifications; it does not fail on AbstractInput alone.
    // But the classification is visible to Origin for review.
    let r_abstract = record.input_inventory.iter()
        .find(|i| i.name == "R_prior_abstract")
        .expect("R_prior_abstract must appear in inventory");

    assert!(
        matches!(r_abstract.classification, irrs_eval::record::InventoryKind::AbstractInput),
        "Adversarial UT-14: AbstractInput classification must be recorded as-is for Origin review."
    );

    // The key distinction: DerivedValue (gate-produced) vs AbstractInput (stipulated).
    // Origin must see this difference; the evaluator must not silently promote AbstractInput.
    assert!(
        !matches!(r_abstract.classification, irrs_eval::record::InventoryKind::DerivedValue),
        "Adversarial UT-14: The evaluator must not reclassify AbstractInput as DerivedValue."
    );
}

// ── UT-15: Valid observations with OPEN persistence dependency ────────────────

/// UT-15: When prior relational evidence is unavailable (OPEN), valid sensor
/// observations remain admissible. The OPEN condition applies specifically to the
/// persistence dependency, not to the entire evaluation.
///
/// "Otherwise admissible operations remain available; dependent operations remain OPEN."
///
/// The evaluator must:
/// - Pass the evaluation (not fail on OPEN prior).
/// - Record valid sensor observations in the input inventory as PrimaryObservable.
/// - Report the OPEN condition in continuous_process.open_conditions.
/// - Not prevent the mapping chain from being declared.
#[test]
fn test_ut15_valid_observations_with_open_prior_remain_admissible() {
    let decl = Declaration {
        domain:      "actuator-lidar-torque".to_string(),
        topology_id: "continuous_relational_process_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: Some(
            "UT-15: OPEN prior dependency. Valid observations remain admissible. \
             Phase 1 spatial operators (A→B→R on current obs) can proceed. \
             Persistence chain (requires prior R) is OPEN.".to_string()
        ),
        persistence: Some(PersistenceDeclaration {
            process_entry:
                "Evaluation window opens onto ongoing process. \
                 First evaluation cycle: no prior R exists yet. \
                 Phase 1 operators run on current observations. \
                 Persistence chain awaits first completed prior.".to_string(),
            prior_evidence:      PriorEvidenceStatus::Open,
            prior_evidence_refs: vec![],
            persistence_path:    kernel_v8_path(),
        }),
        inputs: vec![
            sensor_input(
                "lidar_displacement_v_out",
                "Primary observable: LIDAR displacement at v_out. \
                 Admissible regardless of prior availability.",
            ),
            sensor_input(
                "torque_sensor_v_out",
                "Primary observable: torque at v_out. \
                 Admissible regardless of prior availability.",
            ),
        ],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![
            // Phase 1 spatial operators — do not require prior R.
            MappingDeclaration {
                name:               "A_spatial".to_string(),
                input_description:  "Successive sensor observations at v_out".to_string(),
                output_description: "Directed relational difference".to_string(),
                preserves:          "relational direction of change".to_string(),
                discards:           "absolute magnitude".to_string(),
            },
            // Persistence mapping — requires prior R, which is OPEN.
            MappingDeclaration {
                name:               "A_persistence_OPEN".to_string(),
                input_description:  "Prior R (OPEN) + current Phase 1 result".to_string(),
                output_description: "Persistence result (OPEN pending prior R)".to_string(),
                preserves:          "relational continuity across cycles".to_string(),
                discards:           "cycle-boundary discontinuity".to_string(),
            },
        ],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut15".to_string());

    // OPEN must not fail evaluation.
    assert_eq!(
        record.status, EvaluationStatus::PassedAwaitingAuthorization,
        "UT-15: OPEN prior dependency must not fail evaluation. \
         Otherwise admissible operations remain available."
    );

    // Sensor observations must be in the inventory as PrimaryObservable.
    let lidar_item = record.input_inventory.iter()
        .find(|i| i.name == "lidar_displacement_v_out")
        .expect("lidar_displacement_v_out must be in inventory");
    assert!(
        matches!(lidar_item.classification, irrs_eval::record::InventoryKind::PrimaryObservable),
        "UT-15: LIDAR observation must be admitted as PrimaryObservable despite OPEN prior."
    );

    let torque_item = record.input_inventory.iter()
        .find(|i| i.name == "torque_sensor_v_out")
        .expect("torque_sensor_v_out must be in inventory");
    assert!(
        matches!(torque_item.classification, irrs_eval::record::InventoryKind::PrimaryObservable),
        "UT-15: Torque observation must be admitted as PrimaryObservable despite OPEN prior."
    );

    // Mapping chain must be declared (Phase 1 spatial ops are available).
    assert!(
        !record.mapping_chain.is_empty(),
        "UT-15: mapping_chain must be non-empty; Phase 1 spatial operations are declared."
    );

    // OPEN condition must be reported.
    let cp = record.continuous_process.as_ref().expect("continuous_process required");
    assert_eq!(
        cp.prior_evidence, PriorEvidenceReport::Open,
        "UT-15: prior_evidence must be Open."
    );
    assert!(
        !cp.open_conditions.is_empty(),
        "UT-15: open_conditions must document the persistence dependency."
    );

    // OPEN ≠ FAILED. The evaluation_notes must contain the OPEN notice,
    // not an error that stopped evaluation.
    assert!(
        record.evaluation_notes.iter().any(|n| n.contains("OPEN")),
        "UT-15: evaluation_notes must include the OPEN condition notice."
    );
    assert!(
        !record.evaluation_notes.iter().any(|n|
            n.contains("VIOLATION") || n.contains("PROHIBITED")
        ),
        "UT-15: evaluation_notes must not contain VIOLATION entries for an OPEN prior."
    );
}

/// Adversarial UT-15: Verify that OPEN is not silently converted to zero in
/// evaluation_notes or open_conditions text.
#[test]
fn test_ut15_adv_open_not_reported_as_zero() {
    let decl = Declaration {
        domain:      "test".to_string(),
        topology_id: "continuous_test_v1".to_string(),
        version:     "1.0.0".to_string(),
        description: None,
        persistence: Some(open_persistence()),
        inputs: vec![sensor_input("s", "sensor")],
        regional_prerequisite: valid_prerequisite(),
        mappings: vec![valid_mapping()],
    };

    let record = Evaluator::evaluate(decl, "test-digest-ut15-adv".to_string());

    let cp = record.continuous_process.as_ref().expect("continuous_process required");
    for cond in &cp.open_conditions {
        let lower = cond.to_lowercase();
        // The condition must explicitly state the value does not exist, not that it is zero.
        // It must not imply zero without qualification.
        assert!(
            !lower.contains("is zero") && !lower.contains("= 0") && !lower.contains("equals zero"),
            "Adversarial UT-15: open_conditions must not imply that the unavailable \
             prior equals zero. Got: '{}'", cond
        );
    }
}

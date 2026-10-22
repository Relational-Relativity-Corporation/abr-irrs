/// irrs-eval binary entry point.
///
/// Evaluates the DataFeel circadian declaration, checks for an existing
/// eval/authorization.json, and writes the record to stdout as JSON.
///
/// Exit codes (consumed by the GitHub workflow):
///   0  — Authorized. Build may proceed.
///   1  — Evaluation failed. A prerequisite was not satisfied.
///   2  — Passed; awaiting Origin authorization.
///        CI uploads the record as an artifact. Build is held.

mod record;
mod evaluator;

use evaluator::{DeclarationSpec, Evaluator, ObservableSpec, PrerequisiteSpec, MappingSpec};
use record::{EvaluationStatus, InventoryKind};

fn main() {
    let spec = datafeel_circadian_spec();
    let mut rec = Evaluator::evaluate(spec);

    // Attempt to load authorization.
    if let Some(auth) = load_authorization("eval/authorization.json") {
        if auth.record_id == rec.record_id {
            rec.authorization = Some(auth);
            rec.status = EvaluationStatus::Authorized;
            rec.failure_reason = None;
        } else {
            eprintln!(
                "WARNING: eval/authorization.json record_id ({}) does not match \
                 current record_id ({}). Authorization not applied.",
                auth.record_id, rec.record_id
            );
        }
    }

    let json = serde_json::to_string_pretty(&rec)
        .expect("serialization failed");
    println!("{}", json);

    match rec.status {
        EvaluationStatus::Authorized => {
            eprintln!("✅  Declaration authorized. Build may proceed.");
            std::process::exit(0);
        }
        EvaluationStatus::PassedAwaitingAuthorization => {
            eprintln!(
                "⏸   Evaluation passed. Awaiting Origin authorization.\n\
                 Review eval/record.json (uploaded as CI artifact), then commit:\n\
                 \n\
                     eval/authorization.json\n\
                 \n\
                 with record_id = \"{}\"",
                rec.record_id
            );
            std::process::exit(2);
        }
        EvaluationStatus::Failed => {
            eprintln!(
                "❌  Evaluation failed: {}",
                rec.failure_reason.as_deref().unwrap_or("unknown")
            );
            std::process::exit(1);
        }
    }
}

fn load_authorization(path: &str) -> Option<record::OriginAuthorization> {
    let content = std::fs::read_to_string(path).ok()?;
    match serde_json::from_str(&content) {
        Ok(a)  => Some(a),
        Err(e) => {
            eprintln!("WARNING: could not parse {}: {}", path, e);
            None
        }
    }
}

// ── DataFeel circadian declaration spec ──────────────────────────────────────
//
// This is the description of what datafeel-v05 declares.
// It does not execute any mappings; it describes them.
// The evaluator uses this to produce the evidence record Origin reviews.

fn datafeel_circadian_spec() -> DeclarationSpec {
    DeclarationSpec {
        domain:      "datafeel",
        topology_id: "circadian_v1",
        version:     env!("CARGO_PKG_VERSION"),

        observables: vec![
            ObservableSpec {
                name:           "elapsed_s",
                source:         "system_monotonic",
                physical_basis: "Host monotonic clock: elapsed seconds \
                                 since session start. Directly measurable; \
                                 does not require derivation.",
                kind:           InventoryKind::PrimaryObservable,
                submitted_to_gate: true,
            },
            ObservableSpec {
                name:           "period_s",
                source:         "datafeel-v05 configuration",
                physical_basis: "Declared circadian period (86400 s). \
                                 Not sensed from the environment. \
                                 Entered by the application as a declared reference value.",
                kind:           InventoryKind::DeclaredParameter,
                submitted_to_gate: true,
            },
            ObservableSpec {
                name:           "CircadianPhase",
                source:         "ratio_projection(elapsed_s, period_s)",
                physical_basis: "Fractional ratio of elapsed seconds to declared period. \
                                 Computed by the AdmissionGate from the two submitted values. \
                                 The Generator does not compute or submit this value.",
                kind:           InventoryKind::DerivedValue,
                submitted_to_gate: false,
            },
        ],

        prerequisite: PrerequisiteSpec {
            region_name:              "circadian_v1",
            extents_lower:            0.0,
            extents_upper:            1.0,
            extents_label:            "phase_fraction",
            orientation_preserved:    "circadian phase ratio (elapsed / period)",
            orientation_discarded:    "absolute elapsed time and absolute period magnitude",
            admission_rule_name:      "circadian_phase_interior",
            admission_rule_description:
                "Phase must be strictly interior to the open interval (0, 1). \
                 Values at the boundary (0.0 or 1.0) are rejected.",
            // The domain builder calls RegionBuilder::establish(locus) with a
            // verified interior point before building the topology.
            interior_locus_verified: true,
        },

        mappings: vec![
            MappingSpec {
                name:               "ratio_projection",
                input_description:  "elapsed_s (primary observable) + \
                                     period_s (declared reference parameter)",
                output_description: "CircadianPhase(fract(elapsed_s / period_s)) — \
                                     a value in (0, 1) representing position \
                                     within the circadian cycle",
                preserves:          "circadian phase ratio (elapsed / period)",
                discards:           "absolute elapsed time and absolute period magnitude",
            },
        ],
    }
}

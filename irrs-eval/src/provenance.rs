/// Provenance analysis helpers.
///
/// Analyzes the provenance claims in a set of `InputDeclaration`s and
/// produces a `ProvenanceSummary` that the evaluator includes in the record.

use crate::declaration::{InputDeclaration, ProvenanceStatus};

// ── Summary ───────────────────────────────────────────────────────────────────

/// Aggregated provenance status across all declared inputs.
///
/// Included in `EvaluationRecord` so Origin sees at a glance how much
/// of the declared provenance has been independently verified.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProvenanceSummary {
    /// Inputs whose provenance is asserted by the submitter but not verified.
    pub declared_count: usize,

    /// Inputs with supporting evidence evaluated.
    pub verified_count: usize,

    /// Inputs explicitly marked as unverifiable.
    pub unverified_count: usize,

    /// Names of inputs that carry no supporting evidence references.
    /// Non-empty does not fail evaluation; it is a record for Origin review.
    pub inputs_without_evidence: Vec<String>,

    /// Evaluator note summarising the provenance posture.
    pub note: String,
}

/// Analyze all declared inputs and produce a provenance summary.
pub fn analyze(inputs: &[InputDeclaration]) -> ProvenanceSummary {
    let mut declared_count    = 0usize;
    let mut verified_count    = 0usize;
    let mut unverified_count  = 0usize;
    let mut without_evidence  = Vec::new();

    for input in inputs {
        match input.provenance_status {
            ProvenanceStatus::Declared    => declared_count    += 1,
            ProvenanceStatus::Verified    => verified_count    += 1,
            ProvenanceStatus::Unverified  => unverified_count  += 1,
        }
        if input.supporting_evidence.is_empty() {
            without_evidence.push(input.name.clone());
        }
    }

    let note = build_note(declared_count, verified_count, unverified_count, &without_evidence);

    ProvenanceSummary {
        declared_count,
        verified_count,
        unverified_count,
        inputs_without_evidence: without_evidence,
        note,
    }
}

fn build_note(
    declared: usize,
    verified: usize,
    unverified: usize,
    without_evidence: &[String],
) -> String {
    let total = declared + verified + unverified;
    if total == 0 {
        return "No inputs declared.".to_string();
    }

    let mut parts = Vec::new();

    if unverified > 0 {
        parts.push(format!(
            "{} input(s) marked unverified — Origin should assess whether independent verification is required",
            unverified
        ));
    }

    if !without_evidence.is_empty() {
        parts.push(format!(
            "{} input(s) carry no supporting evidence ({})",
            without_evidence.len(),
            without_evidence.join(", ")
        ));
    }

    if verified == total {
        parts.push("All declared inputs carry supporting evidence.".to_string());
    } else if verified == 0 {
        parts.push(
            "All provenance is declared, not independently verified. \
             This is acceptable for an initial declaration; Origin should note \
             which inputs would benefit from independent evidence.".to_string()
        );
    }

    if parts.is_empty() {
        format!(
            "{} declared, {} verified, {} unverified.",
            declared, verified, unverified
        )
    } else {
        parts.join(" ")
    }
}

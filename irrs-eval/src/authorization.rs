/// Authorization loading and application.
///
/// Loads `eval/authorization.json` (or a caller-specified path), verifies
/// that the `record_id` AND `declaration_digest` match the current record,
/// and applies the authorization only when all conditions are satisfied.
///
/// Enforcement rules (none may be bypassed):
/// 1. A FAILED evaluation cannot be authorized. Authorization is not a fix.
/// 2. The `record_id` must match the current record's `record_id`.
/// 3. The `declaration_digest` must match the current record's digest.
///    A change to any byte of the declaration produces a different digest.
///    An authorization is bound to the exact declaration text that was evaluated.

use crate::record::{EvaluationRecord, EvaluationStatus, OriginAuthorization};

/// Attempt to load an authorization file from `path`.
///
/// Returns `Some(auth)` when the file exists and parses correctly.
/// Returns `None` when the file is absent (not yet committed by Origin).
/// Emits a warning and returns `None` if the file is present but malformed.
pub fn load(path: &str) -> Option<OriginAuthorization> {
    let content = std::fs::read_to_string(path).ok()?;
    match serde_json::from_str::<OriginAuthorization>(&content) {
        Ok(a)  => Some(a),
        Err(e) => {
            eprintln!("WARNING: could not parse {}: {}", path, e);
            None
        }
    }
}

/// Apply an `OriginAuthorization` to a record if all three conditions pass.
///
/// Condition 1 — Not failed: A failed evaluation cannot be authorized.
/// Condition 2 — record_id matches: Binds authorization to this domain/topology/version.
/// Condition 3 — declaration_digest matches: Binds authorization to the exact declaration text.
///
/// Modifies `record` in place on success. On any failure, emits a specific
/// warning, appends to `evaluation_notes`, and leaves the record unchanged.
pub fn apply(record: &mut EvaluationRecord, auth: OriginAuthorization) {
    // Rule 1: Authorization cannot fix a failed evaluation.
    if record.status == EvaluationStatus::Failed {
        let note = format!(
            "Authorization REJECTED: evaluation status is 'failed'. \
             Authorization cannot override a failed evaluation. \
             Failure reason: {}",
            record.failure_reason.as_deref().unwrap_or("unknown")
        );
        eprintln!("ERROR: {}", note);
        record.evaluation_notes.push(note);
        return;
    }

    // Rule 2: record_id must match.
    if auth.record_id != record.record_id {
        let note = format!(
            "Authorization REJECTED: record_id mismatch. \
             Authorization has record_id '{}'; current record_id is '{}'. \
             Re-authorize with the current record_id.",
            auth.record_id, record.record_id
        );
        eprintln!("WARNING: {}", note);
        record.evaluation_notes.push(note);
        return;
    }

    // Rule 3: declaration_digest must match.
    if auth.declaration_digest != record.declaration_digest {
        let note = format!(
            "Authorization REJECTED: declaration_digest mismatch. \
             The declaration has changed since this authorization was issued. \
             The authorization was issued for digest '{}'; \
             the current declaration produces digest '{}'. \
             Re-evaluate and re-authorize.",
            auth.declaration_digest, record.declaration_digest
        );
        eprintln!("WARNING: {}", note);
        record.evaluation_notes.push(note);
        return;
    }

    // All conditions passed.
    record.authorization = Some(auth);
    record.status = EvaluationStatus::Authorized;
    record.failure_reason = None;
}

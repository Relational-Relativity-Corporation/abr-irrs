/// Authorization loading and application.
///
/// Loads `eval/authorization.json` (or a caller-specified path), verifies
/// that the `record_id` matches the current record, and applies the
/// authorization if so.

use crate::record::{EvaluationRecord, EvaluationStatus, OriginAuthorization};

/// Attempt to load `eval/authorization.json` from `path`.
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

/// Apply an `OriginAuthorization` to a record if the `record_id` matches.
///
/// Modifies `record` in place:
/// - On match: sets `status` to `Authorized`, attaches the authorization.
/// - On mismatch: emits a warning; record is unchanged.
pub fn apply(record: &mut EvaluationRecord, auth: OriginAuthorization) {
    if auth.record_id == record.record_id {
        record.authorization = Some(auth);
        record.status = EvaluationStatus::Authorized;
        record.failure_reason = None;
    } else {
        eprintln!(
            "WARNING: eval/authorization.json record_id ({}) does not match \
             current record_id ({}). Authorization not applied. \
             Re-authorize with the current record_id.",
            auth.record_id, record.record_id
        );
        record.evaluation_notes.push(format!(
            "Stale authorization found (record_id: {}). \
             Does not match current record_id ({}). Not applied.",
            auth.record_id, record.record_id
        ));
    }
}

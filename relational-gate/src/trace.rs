/// Operator Traces — mathematical and observational witnesses.
///
/// Every admitted execution produces a `TraceRecord<V>`.
/// Rejected packages produce no trace entry.
/// Divergent results produce a trace entry with `Divergence` attached.
///
/// C4: `Rejected` ≠ divergent.
///   Rejected  → not admitted; mathematical/structural; physical relation unknown.
///   Divergent → admitted and executed; result preserved; not suppressed.

use crate::admitted::ProvenanceRecord;

/// A divergence: the conformant result disagreed with a declared expected value.
/// Preserved alongside the trace entry. Never used as a reason to reject.
#[derive(Debug, Clone)]
pub struct Divergence {
    pub expected:    f64,
    pub got:         f64,
    pub description: &'static str,
}

/// One complete trace record for an admitted, executed operation.
#[derive(Debug, Clone)]
pub struct TraceRecord<V: Clone> {
    pub provenance:     ProvenanceRecord,
    pub admitted_value: V,
    /// `Some` if the result diverged from a declared expected value.
    pub divergence:     Option<Divergence>,
}

/// The operator trace for a complete run.
pub struct OperatorTrace<V: Clone> {
    records:        Vec<TraceRecord<V>>,
    rejected_count: u32,
}

impl<V: Clone> std::fmt::Debug for OperatorTrace<V>
where V: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OperatorTrace {{ executed: {}, rejected: {} }}",
            self.records.len(), self.rejected_count)
    }
}

impl<V: Clone> OperatorTrace<V> {
    pub fn new() -> Self {
        OperatorTrace { records: Vec::new(), rejected_count: 0 }
    }

    pub fn record_execution(&mut self, record: TraceRecord<V>) {
        self.records.push(record);
    }

    /// Record a rejection. No trace entry is created; the count is preserved.
    pub fn record_rejection(&mut self) {
        self.rejected_count += 1;
    }

    pub fn records(&self)         -> &[TraceRecord<V>] { &self.records }
    pub fn execution_count(&self) -> usize  { self.records.len() }
    pub fn rejected_count(&self)  -> u32    { self.rejected_count }
    pub fn divergence_count(&self)-> usize  {
        self.records.iter().filter(|r| r.divergence.is_some()).count()
    }

    pub fn conformance_summary(&self) -> ConformanceSummary {
        ConformanceSummary {
            executed:  self.execution_count(),
            rejected:  self.rejected_count as usize,
            divergent: self.divergence_count(),
        }
    }
}

#[derive(Debug)]
pub struct ConformanceSummary {
    pub executed:  usize,
    pub rejected:  usize,
    pub divergent: usize,
}

impl std::fmt::Display for ConformanceSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} executed  /  {} rejected (not evaluated)  /  {} divergent (preserved)",
            self.executed, self.rejected, self.divergent
        )
    }
}

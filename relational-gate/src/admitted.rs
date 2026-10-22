/// AdmittedObservation<T> — the core enforcement primitive.
///
/// This is the struct you call alongside any build.
///
/// If a function's signature requires `AdmittedObservation<T>`,
/// the caller cannot satisfy it without passing through the gate.
/// The type system makes any other path a compile error.
///
/// `new_admitted` is `pub(crate)`. It is invisible outside this crate.
/// The gate is the only caller. Application code cannot construct this type.
///
/// This is true regardless of what domain mathematics the application implements,
/// what libraries it imports, or what calculations it performs — none of those
/// can produce an `AdmittedObservation<T>`.

/// The gate's own record of what it verified and computed.
/// This is not the Generator's declared chain — it is the gate's witness.
#[derive(Debug, Clone)]
pub struct ProvenanceRecord {
    pub source_id:           String,
    pub topology_id:         String,
    pub mapping_applied:     String,
    pub primary_input:       f64,
    pub reference_input:     Option<f64>,
    pub gate_computed_result: f64,
}

/// An observation that has passed through the relational admission gate.
///
/// `T` is the admitted value type (e.g. `CircadianPhase`, `ThermalDelta`).
///
/// The constructor is `pub(crate)`. No code outside `relational-gate` can produce this.
#[derive(Debug, Clone)]
pub struct AdmittedObservation<T> {
    inner:      T,
    provenance: ProvenanceRecord,
}

impl<T> AdmittedObservation<T> {
    /// Private to this crate. The gate is the only caller.
    pub(crate) fn new_admitted(inner: T, provenance: ProvenanceRecord) -> Self {
        AdmittedObservation { inner, provenance }
    }

    /// Read the admitted value. Public — anyone may observe.
    pub fn value(&self) -> &T { &self.inner }

    /// The gate's provenance record for this observation.
    pub fn provenance(&self) -> &ProvenanceRecord { &self.provenance }

    /// The gate-computed numerical result of the applied M mapping.
    /// This is the value the gate independently computed —
    /// not any value the Generator supplied.
    pub fn gate_result(&self) -> f64 { self.provenance.gate_computed_result }
}

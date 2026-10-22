/// The `Observable` trait — what a raw source must provide.
///
/// Any type that represents a physical or logical observation implements this.
/// The gate uses `primary_value()` and optionally `reference_value()` as inputs
/// to the authorized M mapping it executes.
///
/// Implementing this trait does not admit the observation.
/// It merely declares that the type is a candidate for submission.

pub trait Observable: Send + Sync {
    /// A stable identifier for this observation source.
    /// Used in the gate's provenance record.
    fn source_id(&self) -> &str;

    /// The primary raw value of this observation, before any M transformation.
    /// This is what the gate independently transforms — never a pre-computed result.
    fn primary_value(&self) -> f64;

    /// An optional second raw input required by some mappings (e.g. a period or
    /// reference value). Returning `None` means this observation has no reference.
    /// The gate will reject any declaration that names a mapping requiring a
    /// reference value when this returns `None`.
    fn reference_value(&self) -> Option<f64> {
        None
    }
}

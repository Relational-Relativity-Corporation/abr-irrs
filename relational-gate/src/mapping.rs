/// Authorized Mappings — the M in the relational treatment.
///
/// An `AuthorizedMapping<V>` has two functions:
///
///   `execute(primary, reference) -> Result<V, MappingError>`
///     The gate calls this to produce the admitted value. The Generator never
///     computes this; only the gate calls it.
///
///   `to_scalar(v) -> f64`
///     Extracts the scalar representation of V for topology verification
///     (the interior-locus condition). Declared alongside the mapping.

#[derive(Debug)]
pub enum MappingError {
    InvalidInput     { reason: &'static str },
    MissingReference,
    DomainViolation  { value: f64, detail: &'static str },
}

impl std::fmt::Display for MappingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MappingError::InvalidInput { reason } =>
                write!(f, "mapping input invalid: {}", reason),
            MappingError::MissingReference =>
                write!(f, "mapping requires a reference value; none was supplied"),
            MappingError::DomainViolation { value, detail } =>
                write!(f, "domain violation at {}: {}", value, detail),
        }
    }
}

pub struct AuthorizedMapping<V> {
    pub name:        &'static str,
    pub description: &'static str,
    /// Gate calls this to independently produce V from raw observable inputs.
    pub execute:  Box<dyn Fn(f64, Option<f64>) -> Result<V, MappingError> + Send + Sync>,
    /// Gate calls this to extract the scalar for topology verification.
    pub to_scalar: Box<dyn Fn(&V) -> f64 + Send + Sync>,
}

impl<V> AuthorizedMapping<V> {
    pub fn new(
        name:        &'static str,
        description: &'static str,
        execute:     impl Fn(f64, Option<f64>) -> Result<V, MappingError> + Send + Sync + 'static,
        to_scalar:   impl Fn(&V) -> f64 + Send + Sync + 'static,
    ) -> Self {
        AuthorizedMapping {
            name,
            description,
            execute:   Box::new(execute),
            to_scalar: Box::new(to_scalar),
        }
    }
}

impl<V> std::fmt::Debug for AuthorizedMapping<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AuthorizedMapping {{ name: {:?} }}", self.name)
    }
}

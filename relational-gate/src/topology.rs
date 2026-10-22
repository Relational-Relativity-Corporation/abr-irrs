/// Topology Specification — domain-agnostic
///
/// A `TopologySpec` is instantiated from an `EstablishedRegion`.
/// It carries the declared topology ID and declared edges.
/// It does NOT hold mappings — those are registered with the gate.
/// This separation means a topology is a structural declaration;
/// the gate is the executor.

use crate::region::EstablishedRegion;

/// A declared edge: a directed relation between two observable families.
/// Direction and provenance are required; neither is optional.
#[derive(Debug, Clone)]
pub struct DeclaredEdge {
    pub from_family:  &'static str,
    pub to_family:    &'static str,
    pub provenance:   &'static str,
    pub mapping_name: &'static str,
}

/// A topology instantiated from a verified region.
pub struct TopologySpec {
    pub id:     &'static str,
    pub region: EstablishedRegion,
    pub edges:  Vec<DeclaredEdge>,
}

impl TopologySpec {
    pub fn new(
        id:     &'static str,
        region: EstablishedRegion,
        edges:  Vec<DeclaredEdge>,
    ) -> Self {
        TopologySpec { id, region, edges }
    }

    pub fn verify_id(&self, id: &str) -> Result<(), TopologyError> {
        if self.id != id {
            return Err(TopologyError::IdMismatch {
                expected: self.id,
                received: id.to_string(),
            });
        }
        Ok(())
    }

    /// Verify that a gate-computed value is interior to the declared extents.
    pub fn verify_interior(&self, value: f64) -> Result<(), TopologyError> {
        if !self.region.extents.is_interior(value) {
            return Err(TopologyError::ValueOutsideExtents {
                value,
                lower: self.region.extents.lower,
                upper: self.region.extents.upper,
            });
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum TopologyError {
    IdMismatch { expected: &'static str, received: String },
    ValueOutsideExtents { value: f64, lower: f64, upper: f64 },
}

impl std::fmt::Display for TopologyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopologyError::IdMismatch { expected, received } =>
                write!(f, "topology ID mismatch: expected '{}', received '{}'", expected, received),
            TopologyError::ValueOutsideExtents { value, lower, upper } =>
                write!(f, "value {} ∉ interior of ({}, {})", value, lower, upper),
        }
    }
}

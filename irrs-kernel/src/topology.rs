/// Kernel V8 — Declared Relational Topology
///
/// A topology is instantiated only from an EstablishedRegion.
/// It carries the declared operator parameters, edge provenance,
/// and M-mapping registry.
///
/// C3: Kernel selection follows the declared relational topology.
/// No operator parameter is silently supplied by a default.

use crate::region::EstablishedRegion;

/// An M-mapping authorized for this topology.
/// Generators supply the mapping name; the gate independently executes it.
/// C2: The gate executes the mapping — not the Generator.
#[derive(Debug, Clone)]
pub struct MMapping {
    pub name: &'static str,
    pub description: &'static str,
    /// pub so irrs-admission can execute the mapping. Application code cannot
    /// reach this field — it has no dependency on irrs-kernel.
    pub execute: fn(f64, f64) -> f64,
}

/// The edge between two observable families, with declared provenance.
/// No edge without declared direction and provenance.
#[derive(Debug, Clone)]
pub struct DeclaredEdge {
    pub from_family: &'static str,
    pub to_family: &'static str,
    pub provenance: &'static str,
    pub mapping_name: &'static str,
}

/// Operator parameters that must be explicitly provided.
/// No missing value is silently replaced with zero or a default.
#[derive(Debug, Clone)]
pub struct OperatorParameters {
    pub period_s: f64,
    pub reference_locus: f64,
    pub declared_topology_id: &'static str,
}

impl OperatorParameters {
    pub fn new(
        period_s: f64,
        reference_locus: f64,
        declared_topology_id: &'static str,
    ) -> Result<Self, TopologyError> {
        if period_s <= 0.0 {
            return Err(TopologyError::InvalidParameter {
                name: "period_s",
                reason: "must be positive",
            });
        }
        if reference_locus <= 0.0 {
            return Err(TopologyError::InvalidParameter {
                name: "reference_locus",
                reason: "must be positive",
            });
        }
        Ok(OperatorParameters {
            period_s,
            reference_locus,
            declared_topology_id,
        })
    }
}

/// A topology instantiated from a verified region.
/// The region establishment record is preserved in the topology.
pub struct KernelTopology {
    pub region: EstablishedRegion,
    pub parameters: OperatorParameters,
    pub edges: Vec<DeclaredEdge>,
    pub m_mappings: Vec<MMapping>,
}

impl KernelTopology {
    pub fn new(
        region: EstablishedRegion,
        parameters: OperatorParameters,
        edges: Vec<DeclaredEdge>,
        m_mappings: Vec<MMapping>,
    ) -> Self {
        KernelTopology { region, parameters, edges, m_mappings }
    }

    /// Retrieve an authorized M-mapping by name.
    /// Returns None if no mapping with that name is declared.
    pub fn get_mapping(&self, name: &str) -> Option<&MMapping> {
        self.m_mappings.iter().find(|m| m.name == name)
    }

    /// Verify that a declared topology ID matches this topology's declared ID.
    pub fn verify_topology_id(&self, id: &str) -> Result<(), TopologyError> {
        if self.parameters.declared_topology_id != id {
            return Err(TopologyError::TopologyMismatch {
                expected: self.parameters.declared_topology_id,
                received: id.to_string(),
            });
        }
        Ok(())
    }

    /// Verify that a phase value (as returned by the gate's execution of an
    /// M-mapping) lies within the declared topology extents.
    pub fn verify_phase_in_topology(&self, phase: f64) -> Result<(), TopologyError> {
        if !self.region.extents.is_interior(phase) {
            return Err(TopologyError::PhaseOutsideTopology {
                phase,
                lower: self.region.extents.lower,
                upper: self.region.extents.upper,
            });
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum TopologyError {
    InvalidParameter { name: &'static str, reason: &'static str },
    TopologyMismatch { expected: &'static str, received: String },
    PhaseOutsideTopology { phase: f64, lower: f64, upper: f64 },
}

impl std::fmt::Display for TopologyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopologyError::InvalidParameter { name, reason } =>
                write!(f, "invalid operator parameter '{}': {}", name, reason),
            TopologyError::TopologyMismatch { expected, received } =>
                write!(f, "topology mismatch: expected '{}', received '{}'", expected, received),
            TopologyError::PhaseOutsideTopology { phase, lower, upper } =>
                write!(f, "phase {} is outside declared topology extents ({}, {})", phase, lower, upper),
        }
    }
}

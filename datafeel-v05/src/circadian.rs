/// DataFeel circadian declarations.
///
/// This module wires the general-purpose relational-gate to the DataFeel
/// domain. It:
///   - Defines the observable source type for a host clock reading.
///   - Implements `Observable` for it.
///   - Declares the circadian topology and the authorized ratio-projection mapping.
///   - Provides the declaration builder the Generator calls.
///
/// Nothing here executes mathematics. Everything here declares it.

use relational_gate::{
    Observable, AuthorizedMapping, MappingError,
    RegionBuilder, ObservableExtent, CrossFamilyOrientation, RegionalAdmissionRule,
    TopologySpec, DeclaredEdge,
    AdmissionGate, DeclarationPackage, ProvenanceChain,
};

// ── Observable source type ──────────────────────────────────────────────────

/// A raw host-clock observation: elapsed seconds since session start.
#[derive(Debug, Clone)]
pub struct HostClockObservation {
    pub source_id: &'static str,
    pub elapsed_s: f64,
    pub period_s:  f64,     // reference value for the ratio mapping
}

impl Observable for HostClockObservation {
    fn source_id(&self)      -> &str       { self.source_id }
    fn primary_value(&self)  -> f64        { self.elapsed_s }
    fn reference_value(&self)-> Option<f64>{ Some(self.period_s) }
}

// ── Admitted value type ─────────────────────────────────────────────────────

/// The domain value produced by the ratio_projection mapping.
/// This is what `AdmittedObservation<CircadianPhase>` carries.
/// It can only exist if the gate admitted it.
#[derive(Debug, Clone, Copy)]
pub struct CircadianPhase(f64); // invariant: strictly interior to (0, 1)

impl CircadianPhase {
    pub fn value(self) -> f64 { self.0 }
}

// ── Topology declaration ─────────────────────────────────────────────────────

/// Build and return the `AdmissionGate` for the DataFeel circadian V1 topology.
///
/// `initial_elapsed_s`: first host-clock observation, used to establish the
///   regional locus. Must produce a phase strictly interior to (0, 1).
/// `period_s`: the declared circadian period. Must be positive; not defaulted.
pub fn build_circadian_gate(
    initial_elapsed_s: f64,
    period_s: f64,
) -> Result<AdmissionGate<HostClockObservation, CircadianPhase>, String> {
    // Regional M prerequisite (C3): all four fields required, no defaults.
    let extents = ObservableExtent::new(0.0, 1.0, "phase_fraction")
        .map_err(|e| format!("{}", e))?;

    let region = RegionBuilder::new()
        .name("circadian_v1")
        .extents(extents)
        .orientation(CrossFamilyOrientation {
            preserved: "circadian phase ratio (elapsed / period)",
            discarded: "absolute elapsed time and absolute period magnitude",
        })
        .admission_rule(RegionalAdmissionRule {
            name:        "circadian_phase_interior",
            description: "Phase must be strictly interior to (0, 1)",
        })
        .establish((initial_elapsed_s / period_s).fract())
        .map_err(|e| format!("{}", e))?;

    let topology = TopologySpec::new(
        "circadian_v1",
        region,
        vec![
            DeclaredEdge {
                from_family:  "host_clock",
                to_family:    "circadian_phase",
                provenance:   "elapsed_s → fract(elapsed/period) → phase ∈ (0,1)",
                mapping_name: "ratio_projection",
            },
        ],
    );

    // Authorized mapping registry (C2): gate executes these; Generator names them.
    let mappings: Vec<AuthorizedMapping<CircadianPhase>> = vec![
        AuthorizedMapping::new(
            "ratio_projection",
            "Maps (elapsed_s, period_s) → CircadianPhase via fract(elapsed/period)",
            |primary, reference| {
                let period = reference.ok_or(MappingError::MissingReference)?;
                if period <= 0.0 {
                    return Err(MappingError::InvalidInput { reason: "period_s must be positive" });
                }
                Ok(CircadianPhase((primary / period).fract()))
            },
            |phase| phase.value(),  // scalar extractor for topology verification
        ),
    ];

    Ok(AdmissionGate::new(topology, mappings))
}

// ── Declaration builder ──────────────────────────────────────────────────────

/// Build a declaration package for a circadian phase observation.
///
/// The Generator calls this. It supplies raw values and mapping names.
/// It does NOT compute elapsed / period.
pub fn declare_phase(
    elapsed_s: f64,
    period_s:  f64,
) -> DeclarationPackage<HostClockObservation> {
    DeclarationPackage {
        source: HostClockObservation {
            source_id: "system_monotonic",
            elapsed_s,
            period_s,
        },
        mapping_name: "ratio_projection",
        topology_id:  "circadian_v1",
        provenance: ProvenanceChain::new()
            .link("host_clock_observation",       "Raw elapsed seconds from host monotonic clock")
            .link("period_declaration",           "Declared 24-hour circadian period in seconds")
            .link("ratio_projection_authorized",  "Gate will apply M mapping 'ratio_projection'"),
    }
}

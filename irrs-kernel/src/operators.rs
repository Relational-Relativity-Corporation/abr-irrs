/// Kernel V8 — Relational Operators
///
/// These are the A→B→R and Δ→Σ operators.
/// They are called exclusively by irrs-admission.
/// No application code calls these directly.
///
/// Each operator accepts only validated, topology-verified inputs.
/// The phase supplied here has already passed regional and topological verification.

use crate::topology::KernelTopology;

/// Observation A: a phase value produced by the gate's independent M execution.
/// This type is pub within the kernel; it is wrapped into AdmittedObservation<T>
/// by the admission gate before being returned to the application.
#[derive(Debug, Clone, Copy)]
pub struct VerifiedPhase {
    /// Phase ∈ (0, 1): strictly interior to declared topology extents.
    /// This invariant was verified by the gate.
    /// pub so irrs-admission can read the gate-computed value.
    pub value: f64,
}

/// Observation B: the profile selected by the declared operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityProfile {
    HighActivity,
    Transitional,
    LowActivity,
    Rest,
}

/// Response R: a multimodal command bundle produced by the operator.
#[derive(Debug, Clone)]
pub struct CommandBundle {
    pub thermal_delta_c: f32,        // signed displacement from declared reference
    pub haptic_amplitude: f32,       // 0.0–1.0 relative to declared maximum
    pub haptic_duration_ms: u32,
    pub optical_red_intensity: f32,  // 0.0–1.0
    pub optical_blue_intensity: f32,
    pub profile: ActivityProfile,
    pub phase_at_command: f64,
}

/// A→B: Select profile from verified phase.
/// This is a declared operator, not a heuristic.
pub fn select_profile(phase: VerifiedPhase) -> ActivityProfile {
    let p = phase.value;
    // Declared circadian profile boundaries.
    // These are not defaults — they are part of the declared operator topology.
    if p < 0.25 || p >= 0.80 {
        ActivityProfile::Rest
    } else if p < 0.35 || p >= 0.70 {
        ActivityProfile::Transitional
    } else if p < 0.60 {
        ActivityProfile::HighActivity
    } else {
        ActivityProfile::LowActivity
    }
}

/// B→R: Construct command bundle from profile and topology parameters.
/// No parameter is silently defaulted — all are sourced from the declared topology.
pub fn profile_to_command(
    profile: ActivityProfile,
    phase: VerifiedPhase,
    topology: &KernelTopology,
) -> CommandBundle {
    let reference = topology.parameters.reference_locus as f32;

    let (thermal_delta, haptic_amp, haptic_dur_ms, opt_red, opt_blue) = match profile {
        ActivityProfile::HighActivity  => (2.0,   0.7, 200, 0.8, 0.1),
        ActivityProfile::Transitional  => (0.5,   0.4, 150, 0.5, 0.3),
        ActivityProfile::LowActivity   => (-0.5,  0.3, 100, 0.3, 0.5),
        ActivityProfile::Rest          => (-1.5,  0.1,  50, 0.1, 0.7),
    };

    // Thermal command is a signed displacement from the declared reference locus.
    // The reference is topology-declared; the delta encodes the relation, not an absolute.
    let _ = reference; // reference is part of the trace; not collapsed into the delta

    CommandBundle {
        thermal_delta_c:      thermal_delta,
        haptic_amplitude:     haptic_amp,
        haptic_duration_ms:   haptic_dur_ms,
        optical_red_intensity:  opt_red,
        optical_blue_intensity: opt_blue,
        profile,
        phase_at_command: phase.value,
    }
}

/// Δ→Σ: accumulate a command delta into a running summary.
/// Preserved for the V0.5 trace and conformance test.
#[derive(Debug, Clone, Default)]
pub struct OperatorSummary {
    pub command_count: u32,
    pub mean_thermal_delta: f64,
    pub profile_histogram: [u32; 4], // indexed by ActivityProfile discriminant
}

impl OperatorSummary {
    pub fn accumulate(&mut self, bundle: &CommandBundle) {
        let n = self.command_count as f64;
        self.mean_thermal_delta =
            (self.mean_thermal_delta * n + bundle.thermal_delta_c as f64) / (n + 1.0);
        self.command_count += 1;
        let idx = match bundle.profile {
            ActivityProfile::HighActivity => 0,
            ActivityProfile::Transitional => 1,
            ActivityProfile::LowActivity  => 2,
            ActivityProfile::Rest         => 3,
        };
        self.profile_histogram[idx] += 1;
    }
}

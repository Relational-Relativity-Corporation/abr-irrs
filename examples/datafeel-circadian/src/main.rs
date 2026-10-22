/// DataFeel circadian example — relational gate edition.
///
/// Demonstrates the abr-irrs relational gate with a circadian phase topology.
/// This is an example application, not a production deployment.
///
/// Dependency chain:
///   datafeel-circadian → relational-gate  (enforcement primitive)
///   datafeel-circadian → irrs-kernel      (frozen domain operators)
///
/// The application:
///   1. Declares a topology and registers authorized mappings via relational-gate.
///   2. Submits DeclarationPackages to the gate.
///   3. Receives AdmittedObservation<CircadianPhase> values.
///   4. Passes admitted values to Kernel V8 operators.
///
/// Step 2 cannot be bypassed. Step 4 operates on values the gate produced.

mod circadian;

use relational_gate::{AdmissionOutcome, OperatorTrace};
use irrs_kernel::operators::{VerifiedPhase, select_profile, profile_to_command, OperatorSummary};

fn main() {
    println!("DataFeel circadian example — relational-gate edition");
    println!("======================================================");

    // Build the gate. This is the only place topology is declared for this run.
    let period_s          = 24.0 * 3600.0_f64;
    let initial_elapsed_s =  6.0 * 3600.0_f64;  // 25% into cycle

    let gate = match circadian::build_circadian_gate(initial_elapsed_s, period_s) {
        Ok(g)  => {
            println!("\n[Gate ready]");
            println!("  Topology: circadian_v1  |  Period: {}h", period_s / 3600.0);
            g
        }
        Err(e) => {
            eprintln!("[GATE BUILD FAILED]  {}", e);
            std::process::exit(1);
        }
    };

    let mut trace:   OperatorTrace<circadian::CircadianPhase> = OperatorTrace::new();
    let mut summary: OperatorSummary = OperatorSummary::default();

    // Twenty-point sweep
    println!("\n{:<5} {:<8} {:<16} {:<10} {:<8} {:<9}",
        "Pt", "Phase", "Profile", "Thermal Δ", "Hap", "Red/Blue");
    println!("{}", "─".repeat(60));

    for i in 0..20 {
        let elapsed_s = (i as f64 / 20.0) * period_s;
        let package   = circadian::declare_phase(elapsed_s, period_s);

        match gate.submit(package) {

            AdmissionOutcome::Executed { observation, trace: record } => {
                // The admitted observation carries a CircadianPhase the gate produced.
                // We wrap it in VerifiedPhase for the frozen kernel operators.
                let phase_val = observation.value().value();
                let vphase    = VerifiedPhase { value: phase_val };

                // Kernel V8: A→B→R
                let kernel_topo = build_kernel_topology(period_s, phase_val);
                let profile     = select_profile(vphase);
                let command     = profile_to_command(profile, vphase, &kernel_topo);

                summary.accumulate(&command);
                trace.record_execution(record);

                println!("{:<5} {:<8.4} {:<16} {:>+9.1}°  {:<8.2} {:.2}/{:.2}",
                    i + 1,
                    phase_val,
                    profile_label(profile),
                    command.thermal_delta_c,
                    command.haptic_amplitude,
                    command.optical_red_intensity,
                    command.optical_blue_intensity,
                );
            }

            AdmissionOutcome::Rejected { error, topology_id, mapping_name } => {
                println!("{:<5} [REJECTED]  topo={} mapping={}",
                    i + 1, topology_id, mapping_name);
                println!("      {}", error);
                trace.record_rejection();
            }

            AdmissionOutcome::Divergent { observation, trace: record, divergence } => {
                let phase_val = observation.value().value();
                println!("{:<5} {:<8.4} [DIVERGENT expected={:.4} got={:.4}: {}]",
                    i + 1, phase_val,
                    divergence.expected, divergence.got, divergence.description);
                trace.record_execution(record);
            }
        }
    }

    // Conformance summary
    let cs = trace.conformance_summary();
    println!("\n[Conformance]  {}", cs);
    println!("[Δ→Σ summary]  {} commands  |  mean thermal Δ: {:+.2}°C",
        summary.command_count, summary.mean_thermal_delta);
    println!("[Trace]        {} records held", trace.execution_count());

    // Demonstrate that the gate computed the values — show provenance for pt 1
    if let Some(r) = trace.records().first() {
        println!("\n[Provenance — first trace record]");
        println!("  source:       {}", r.provenance.source_id);
        println!("  mapping:      {}", r.provenance.mapping_applied);
        println!("  primary in:   {:.2}s elapsed", r.provenance.primary_input);
        println!("  reference in: {:.0}s period",
            r.provenance.reference_input.unwrap_or(0.0));
        println!("  gate result:  {:.6}  ← gate computed this, not the Generator",
            r.provenance.gate_computed_result);
    }
}

fn profile_label(p: irrs_kernel::operators::ActivityProfile) -> &'static str {
    match p {
        irrs_kernel::operators::ActivityProfile::HighActivity => "HighActivity",
        irrs_kernel::operators::ActivityProfile::Transitional => "Transitional",
        irrs_kernel::operators::ActivityProfile::LowActivity  => "LowActivity",
        irrs_kernel::operators::ActivityProfile::Rest         => "Rest",
    }
}

/// Minimal kernel topology for the frozen operator call.
/// In full integration this is constructed once and shared.
fn build_kernel_topology(period_s: f64, locus: f64) -> irrs_kernel::topology::KernelTopology {
    use irrs_kernel::{
        region::{RegionBuilder, ObservableExtent, CrossFamilyOrientation, RegionalAdmissionRule},
        topology::{KernelTopology, OperatorParameters, MMapping, DeclaredEdge},
    };
    let extents = ObservableExtent::new(0.0, 1.0, "phase_fraction").unwrap();
    let region  = RegionBuilder::new()
        .name("circadian_v1")
        .extents(extents)
        .orientation(CrossFamilyOrientation {
            preserved: "circadian phase ratio",
            discarded: "absolute elapsed time",
        })
        .admission_rule(RegionalAdmissionRule {
            name: "circadian_phase_interior",
            description: "phase ∈ (0,1)",
        })
        .establish(locus)
        .expect("region already verified by gate");
    let params = OperatorParameters::new(period_s, locus, "circadian_v1").unwrap();
    KernelTopology::new(
        region, params,
        vec![DeclaredEdge {
            from_family: "host_clock", to_family: "circadian_phase",
            provenance: "gate-admitted", mapping_name: "ratio_projection",
        }],
        vec![MMapping {
            name: "ratio_projection",
            description: "admitted by gate",
            execute: |e, p| (e / p).fract(),
        }],
    )
}

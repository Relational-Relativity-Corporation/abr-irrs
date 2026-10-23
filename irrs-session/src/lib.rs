/// IRRS Session — Interactive D/M-Driven Relational Computation
///
/// Provides the minimal interactive session layer over Kernel V8:
///
///   1. Accept a bounded D and M (declare_topology, declare_field,
///      declare_rho, declare_cc).
///   2. Check admission prerequisites (admit).
///   3. Execute the applicable frozen Kernel V8 operators (execute).
///   4. Expose the mathematical trace and parameter provenance (trace_report).
///
/// This crate does not modify the kernel. All operator mathematics are
/// executed by `irrs-kernel` unchanged.
///
/// Domain-specificity: ρ_base, χ₀, and cc are domain-specific by reason
/// of complexity. See `irrs-kernel` documentation for the structural
/// explanation. This session layer enforces the distinction between
/// illustrative and observationally established parameters.

pub mod admission;
pub mod session;

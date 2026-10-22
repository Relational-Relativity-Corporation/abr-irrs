# abr-irrs

**Integrated Relational Reference System — Admission-Gated Build**

Metatron Dynamics × DataFeel  
Relational Relativity Corporation

---

## What this is

`abr-irrs` is a Rust workspace that enforces relational mathematics by construction. It makes it a **compile error** to pass a value into a declared relational treatment unless that value was produced by a gate that independently executed the authorized mapping from raw observations.

The central enforcement primitive is `AdmittedObservation<T>`. Its constructor is `pub(crate)` — invisible outside `relational-gate`. No application code can manufacture one. The type system closes the bypass.

This addresses a specific failure mode in automated builds: a system can generate scheduling algorithms, label them relational, and return a successful result — without ever touching the relational primitive. `abr-irrs` makes that impossible by requiring `AdmittedObservation<T>` as the input type for all downstream relational treatments. If the gate did not run, the code does not compile.

---

## Architecture

Three crates, one enforcement chain:

```
relational-gate   ←   irrs-kernel   ←   datafeel-v05
  (general)           (domain)           (application)
```

### `relational-gate`

General-purpose enforcement crate. No domain knowledge, no external dependencies.

Exports the core types:

| Type | Role |
|------|------|
| `Observable` | Trait for raw source types. Provides `primary_value()` and optional `reference_value()`. |
| `AuthorizedMapping<V>` | A named mapping the gate is authorized to execute. Two functions: `execute` (gate calls this from raw inputs) and `to_scalar` (for topology verification). |
| `DeclarationPackage<O>` | What the Generator submits: raw observations + mapping names. Never pre-computed results. |
| `AdmissionGate<O, V>` | Generic gate. Configurable per domain. The only code path that produces `AdmittedObservation<V>`. |
| `AdmittedObservation<V>` | **The enforcement primitive.** Constructor is `pub(crate)`. Cannot be built outside `relational-gate`. |
| `AdmissionOutcome<V>` | `Executed`, `Rejected`, or `Divergent`. |
| `RegionBuilder` | Constructs an `EstablishedRegion`. All four regional M prerequisite fields required — no defaults. |
| `TopologySpec` | Instantiated only from an `EstablishedRegion`. |
| `OperatorTrace<V>` | Traces executed operations. Rejected observations produce no entry; count preserved separately. |

### `irrs-kernel`

Frozen Kernel V8. Domain operators for the DataFeel circadian application. Not modified.

Exports the A→B→R operator chain:

- `VerifiedPhase` → `ActivityProfile` via `select_profile()`  
- `ActivityProfile` → `CommandBundle` via `profile_to_command()`

### `datafeel-v05`

Application. Wires `relational-gate` to the DataFeel circadian domain.

- Declares the `circadian_v1` topology with the `ratio_projection` mapping
- Submits raw host-clock observations to the gate as `DeclarationPackage` values
- Receives `AdmittedObservation<CircadianPhase>` values from the gate
- Passes admitted values to Kernel V8 operators

---

## The Regional M Prerequisite

Before a topology can be instantiated, a region must be established. `RegionBuilder` requires all four fields — no field has a default:

1. **Region identification** — a declared name
2. **Observable extents** — the lower and upper bounds of the topology
3. **Cross-family orientation** — what the mapping preserves and what it discards
4. **Regional admission rule** — the named rule governing interior membership

In addition, the `establish(locus)` call verifies the **interior-locus condition**: the provided observable locus must be strictly interior to the declared extents. If it is not, the build fails before a topology exists.

---

## The Gate's Seven Steps

When the application calls `gate.submit(package)`, the gate:

1. Validates the declaration structure (topology ID, mapping name, provenance chain — none defaulted)
2. Verifies the topology ID matches the gate's declared topology
3. Locates the named authorized mapping in its registry
4. **Independently executes the mapping** from raw `primary_value` and `reference_value` — the Generator never submits a pre-computed result
5. Verifies the result scalar is interior to the declared topology extents
6. Constructs `AdmittedObservation<V>` via the private constructor — this is the type-system enforcement moment
7. Returns the outcome and trace record

---

## Admission Outcomes

| Outcome | Meaning |
|---------|---------|
| `Executed` | Admitted. Mapping executed. Trace produced. |
| `Rejected` | Not admitted for evaluation. Structural or mathematical rejection. The physical relation was not evaluated — this is not a statement about whether the relation holds. |
| `Divergent` | Admitted and executed. Result diverged from a declared expected value. Preserved alongside the trace, never suppressed. |

`Rejected` ≠ `Divergent`. A rejected observation was never evaluated. A divergent result was evaluated and preserved.

---

## The Circadian Example

The DataFeel circadian topology treats phase as a **relation** — the ratio of elapsed time to declared period — not as an absolute value.

```
elapsed_s         period_s
    ↓                 ↓
    └──── ratio_projection ────→ CircadianPhase(fract(elapsed/period))
                                        ↓
                               AdmittedObservation<CircadianPhase>
                                        ↓
                              Kernel V8 A→B→R operators
```

The Generator (`datafeel-v05`) submits raw elapsed seconds and a declared period. It does not compute the ratio. The gate computes it, verifies the result is interior to `(0, 1)`, and produces the `AdmittedObservation<CircadianPhase>`.

Point 1 of the sweep (elapsed = 0, phase = 0.0) is correctly **rejected** by the interior-locus condition — a phase of exactly 0.0 is not interior to the open interval `(0, 1)`.

---

## Building and Running

```bash
# Clone
git clone https://github.com/Relational-Relativity-Corporation/abr-irrs.git
cd abr-irrs

# Build all crates
cargo build

# Run the 20-point circadian sweep
cargo run -p datafeel-v05
```

Expected output includes:
- Gate readiness confirmation with declared period
- 20-point sweep table: phase, activity profile, thermal delta, haptic amplitude, optical intensities
- Conformance summary: executed / rejected / divergent counts
- Provenance record for the first trace entry, showing the gate-computed result

---

## What the enforcer prevents

Without this architecture, an automated system generating DataFeel commands can:

- Compute a circadian phase by any method and pass it directly to operators
- Apply scheduling heuristics that have no declared relational basis
- Label outputs "relational" without any topology having been declared

With this architecture, none of those paths compile. `VerifiedPhase` and the downstream kernel operators require `AdmittedObservation<CircadianPhase>`, and that type cannot be constructed except by the gate.

---

## Extending to a new domain

To apply `relational-gate` to a new observable domain:

1. Implement `Observable` for your raw source type
2. Define your admitted value type
3. Call `RegionBuilder` with all four prerequisite fields and a verified locus
4. Register one or more `AuthorizedMapping<V>` instances with both `execute` and `to_scalar`
5. Construct `AdmissionGate::new(topology, mappings)`
6. Submit `DeclarationPackage` values to the gate; match on `AdmissionOutcome`

The gate is fully generic. It has no knowledge of DataFeel, circadian rhythms, or any other domain.

---

## Repository structure

```
abr-irrs/
├── Cargo.toml                    # Workspace root
├── relational-gate/
│   └── src/
│       ├── lib.rs                # Public interface
│       ├── observable.rs         # Observable trait
│       ├── admitted.rs           # AdmittedObservation (enforcement primitive)
│       ├── mapping.rs            # AuthorizedMapping<V>
│       ├── declaration.rs        # DeclarationPackage<O>
│       ├── gate.rs               # AdmissionGate — seven-step submit
│       ├── region.rs             # RegionBuilder, EstablishedRegion
│       ├── topology.rs           # TopologySpec
│       └── trace.rs              # OperatorTrace, ConformanceSummary
├── irrs-kernel/
│   └── src/
│       ├── operators.rs          # VerifiedPhase, ActivityProfile, CommandBundle
│       ├── region.rs             # Kernel region types (frozen)
│       └── topology.rs           # KernelTopology (frozen)
└── datafeel-v05/
    └── src/
        ├── circadian.rs          # Domain declarations: topology, mapping, observable
        └── main.rs               # 20-point sweep, A→B→R operator chain
```

---

*Metatron Dynamics × DataFeel — Relational Relativity Corporation*

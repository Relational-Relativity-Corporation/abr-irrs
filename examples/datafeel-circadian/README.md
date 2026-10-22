# datafeel-circadian

DataFeel circadian phase example for IRRS — demonstrates the relational gate with a circadian phase topology.

This is an **example application**, not a library or production deployment. It exists to show how a domain builder wires `relational-gate` to a specific problem space.

## What this demonstrates

- Defining an observable source type (`HostClockObservation`)
- Declaring a topology and registering authorized mappings
- Submitting `DeclarationPackage`s to the gate
- Receiving `AdmittedObservation<CircadianPhase>` values the gate produced
- Passing admitted values to frozen Kernel V8 operators (A→B→R chain)

## Declaration

The declaration for this topology lives alongside the example:

```
examples/datafeel-circadian/eval/declaration.json
```

This file is supplied to `irrs-eval` to produce the evaluation record. It is the domain-specific content; `irrs-eval` itself is domain-independent.

To evaluate this declaration locally:

```sh
cargo run -p irrs-eval -- evaluate \
  --declaration examples/datafeel-circadian/eval/declaration.json \
  --output eval/record.json
```

## Running the example

```sh
cargo run -p datafeel-circadian
```

This runs a twenty-point circadian sweep, printing phase, activity profile, thermal delta, haptic amplitude, and optical intensities for each admitted observation.

## Topology

| Field | Value |
|-------|-------|
| Topology ID | `circadian_v1` |
| Region | `(0, 1) phase_fraction` |
| Primary observable | `elapsed_s` — host monotonic clock |
| Declared parameter | `period_s` — 86400 s (24 h) |
| Authorized mapping | `ratio_projection` — `fract(elapsed_s / period_s)` |
| Derived value | `CircadianPhase` — produced by gate, not Generator |

## Extension

To add a new topology, create a new `examples/<domain>/` directory:

1. Define an observable source type implementing `relational_gate::Observable`
2. Declare the topology with `RegionBuilder` and `TopologySpec`
3. Register authorized mappings with `AuthorizedMapping::new(...)`
4. Provide `examples/<domain>/eval/declaration.json`
5. Evaluate with `cargo run -p irrs-eval -- evaluate --declaration examples/<domain>/eval/declaration.json`

---

*Metatron Dynamics × DataFeel — Relational Relativity Corporation*

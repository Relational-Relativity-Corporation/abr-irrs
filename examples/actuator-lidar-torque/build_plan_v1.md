# Build Plan — Actuator / LIDAR / Torque Sensor System
## Relational Treatment, Continuous Process Entry

**Origin:** Metatron Dynamics, Inc.
**Kernel version:** Kernel V8 (2026-09-30)
**Document version:** V1.0
**Date:** 2026-10-22
**Role producing this document:** Generator
**Task scope:** Application construction. Kernel V8 content is governing and is not reopened here.
**Status:** Awaiting Verifier pass.

---

## 1. Founding Statement

Relational understanding begins with recognizing a connection, identifying what changes within that connection, and carrying that relationship forward through recursive algebra.

This maps directly and completely to the declared operator sequence. The three steps are not an external description layered over the operators — they are the operators:

| Step | Relational meaning | Kernel operation |
|---|---|---|
| Recognizing a connection | An edge is declared from observable evidence through M. The connection is not computed — it is declared with provenance. | `DeclaredRelations::from_edges_with_provenance` |
| Identifying what changes within that connection | The directed difference between the observable projections at the source and target loci of the declared edge. | A(x)[e] = x[s] − x[t] |
| Carrying that relationship forward through recursive algebra | Accumulation over declared successor edges (B), antisymmetric adjustment over the declared relational balance (R), then the directed difference between the current and prior kernel output carried into the next declared process step (A_persistence → B_persistence → R_persistence). | B → R at Phase 1; A_persistence → B_persistence → R_persistence at Phase 2 |

---

## 2. System Description

The system consists of three physical components operating continuously:

- **Actuator** — produces mechanical action at a declared output locus
- **LIDAR sensor** — reports position at the actuator's output locus
- **Torque sensor** — reports torque at the actuator's output locus

The system is continuous. There is no observational start in the physical sense. The actuator is acting and the sensors are reporting. The evaluation window opens onto a process already in progress. This is not a special case — it is the ordinary condition of the operators over a running system.

---

## 3. Structural Consequence — Interior Locus

An interior locus is required by the third step (carrying forward). B accumulation over the declared successor structure is only structurally meaningful where the target of a declared edge is also the source of a further declared edge. Without an interior locus, B reduces to the identity on A's output for every terminal edge and the third step is not expressed.

The interior locus of this system is constituted by the sensor observations themselves. The LIDAR sensor and the torque sensor both measure at the same physical locus — the point of mechanical action. That these two independent sensors report at the same physical locus is itself an observable fact, established from device geometry, physical coupling, and measurement specifications. The sensors' readings through M declare the interior. It is not a fabricated convenience node.

---

## 4. Prerequisites — What Origin Must Declare Before Evaluation Proceeds

No operator evaluates until all prerequisites are complete.

### 4.1 Domain Declaration

Declare D and confirm all participating quantities are finite and bounded (Kernel V8, Section 9).

### 4.2 M Regional Instantiation Prerequisite (Kernel V8, Section 5, Step 1)

For each of the three declared regions — Actuator, LIDAR, Torque — Origin must complete all four steps in sequence:

**(a) Region declaration.** Identify each region from observable evidence. State what physical system constitutes the region and what observations establish it. No region may be asserted from mathematical convenience.

**(b) Observable extent declaration.** For each participating variable within each region, declare its observable floor and ceiling from measurement specifications, physical limits, or device range — not from abstract mathematics.

**(c) Regional orientation declaration.** The three sensor families (position via LIDAR, torque via torque sensor, command signal via actuator) are heterogeneous. Where they participate in a shared topology, M must declare the observable correspondence that orients them into a common relational representation on which directed difference is defined. State explicitly what that correspondence preserves and what it discards. No orientation may be introduced solely to make quantities mathematically comparable.

**(d) Regional admission rule.** Observations outside the declared observable extent for each region are recorded but not admitted by M into operator evaluation.

Until all four steps are complete for all three regions, operator instantiation is inadmissible.

### 4.3 M-Declared Operator Parameters

| Parameter | Required for | Declaration requirement |
|---|---|---|
| ρ_base | Wherever ρ is computed (R, R_persistence) | Finite, dimensionless, declared by M |
| χ₀ | Same; carries dimension of χ as declared through M | Finite, positive, declared by M |
| cc[p] | Each declared component pair in R | One finite value per pair, declared by M |

The kernel holds no value for any of these and supplies no default. Where not declared, the operator is not evaluated.

### 4.4 Sequence and Direction Declaration

Before Phase 2 (persistence):

- Declare the sequence of actual observations {M(o₁), M(o₂), …, M(oₙ)} — each oₖ a declared observation at one declared process step, from the running system.
- Declare what one relational process step is for this system, established from observable system behavior (e.g., sensor reporting interval declared through M). Index order alone is not a direction declaration.
- Declare the direction of relational evolution from an observable property of the process.

### 4.5 Component Pair Declaration

Before A evaluates cross-family directed differences at the output locus, Origin must declare which component pairs across observable families are admissible. The kernel does not choose admissible pairs.

### 4.6 Reference Declaration

If a reference target is used, Origin must declare it through M. If no reference is declared, the reference node and any edge to or from it are not evaluated — not evaluated with result 0.0.

---

## 5. Declared Nodes

| Node | What it is | How it enters D |
|---|---|---|
| `v_act` | Actuator command at the command locus | M maps command observable (current, voltage, or equivalent as declared) → D |
| `v_out` | The locus both sensors measure — declared from the sensor observations themselves and from observable device geometry | Established from the observable fact that both sensors report at this locus; Origin declares from device geometry and measurement specifications |
| `v_lidar` | LIDAR sensor output locus | M maps distance/position reading → D |
| `v_torque` | Torque sensor output locus | M maps torque reading → D |
| `v_ref` | Declared reference target | M maps declared target → D if and when declared by Origin; not evaluated if not declared |

`v_out` is interior: it is the target of `e₁` (and `e₄` if declared), and the source of `e₂` and `e₃`. The minimal admissible interior structure s → t → u is satisfied by `v_act → v_out → v_lidar` and `v_act → v_out → v_torque`.

---

## 6. Candidate Spatial Edges

Provenance for each edge is a candidate — Origin must declare the actual provenance from observable evidence before `DeclaredRelations::from_edges_with_provenance` is called.

| Edge | Direction | Candidate provenance class | Relational meaning |
|---|---|---|---|
| `e₁ = (v_act, v_out)` | Command locus → output locus | Continuation (within actuator family, declared from device geometry) | Recognizes the connection between what is commanded and what is measured at the output |
| `e₂ = (v_out, v_lidar)` | Output locus → LIDAR sensor locus | Coupling (physical coupling mechanism to be declared) | What changes between the output locus and the position reading |
| `e₃ = (v_out, v_torque)` | Output locus → torque sensor locus | Coupling | What changes between the output locus and the torque reading |
| `e₄ = (v_ref, v_out)` | Reference → output locus | Coupling (if reference declared) | What changes between the declared target and the current output locus |

**Direction rule.** Each declared edge has exactly one admissible direction — the direction traceable to an observable through M. The reverse of each edge requires independent observable provenance; without it, it is inadmissible. Origin must supply provenance for each edge and each direction.

**Ring inadmissibility.** No spatial edge closes back to its own source. The conventional "closed loop" structure is expressed through the persistence chain — temporal evolution through declared sequential observations — not through a spatial ring. A spatial ring is a DRIFT SIGNAL and is refused (Kernel V8, Section 8).

Persistence edges are not spatial edges and are not declared in the spatial edge list. They are represented in `PersistenceState`.

---

## 7. Operator Sequence — Three Steps at Every Declared Process Step

The same sequence executes at every declared process step k. There is no special first case. The evaluation window opens on actual sensor readings from the running system.

### 7.1 Step 1 — Recognize the Connection

M receives actual sensor readings from the running system at declared process step k:

```
M(o_k) = {
    v_act:    x_act_k       ← actuator command observable
    v_out:    x_out_k       ← constituted by actual sensor data at the output locus
    v_lidar:  x_lidar_k     ← LIDAR reading
    v_torque: x_torque_k    ← torque reading
    v_ref:    x_ref_k       ← declared reference, if declared
}
```

All values are actual observations through M. All must be finite (∈ D). Regional M establishment must be complete. The declared edges exist. Evaluation proceeds.

### 7.2 Step 2 — Identify What Changes (A operator)

Full-operator admissibility condition is satisfied (interior locus established at `v_out`). ABR path applies.

```
A(x)[e₁] = x_act_k   − x_out_k      ← what changes along the actuator's relational path
A(x)[e₂] = x_out_k   − x_lidar_k    ← what changes between output locus and position reading
A(x)[e₃] = x_out_k   − x_torque_k   ← what changes between output locus and torque reading
A(x)[e₄] = x_ref_k   − x_out_k      ← what changes between reference and output locus
```

These are directed differences in D. Whatever the sensors report at process step k — that is what A computes over. No fitting, no estimation, no default substitution.

Where component pairs are declared across observable families at `v_out`, A evaluates component-by-component over the declared pairs (as declared by Origin in 4.5 above).

### 7.3 Step 3 — Carry Forward: B accumulation within this process step

```
B(A(x))[e₁] = A(x)[e₁] + A(x)[e₂] + A(x)[e₃]
    ← carries forward: command-to-output directed difference plus both sensor directed differences

B(A(x))[e₄] = A(x)[e₄] + A(x)[e₂] + A(x)[e₃]
    ← carries forward: reference-to-output directed difference plus both sensor directed differences

B(A(x))[e₂] = A(x)[e₂]    ← terminal; no declared successors
B(A(x))[e₃] = A(x)[e₃]    ← terminal; no declared successors
```

B at `e₁` and `e₄` carries forward the full relational state at the interior locus — the accumulated directed differences from the output locus through both sensor loci. This is the within-step expression of the third relational step.

### 7.4 Step 3 Continued — R antisymmetric adjustment

```
R(B(A(x)))[e] = B(A(x))[e] + ρ[src(e)] · (Σ_{succ(e)} B(A(x)) − Σ_{pred(e)} B(A(x)))
```

ρ computed from M-declared ρ_base and χ₀. cc[p] scales the cross-topology term for each declared component pair.

R is not optional (Kernel V8, Section 10). Once ABR admissibility is established, R must be applied. R output is preserved exactly. Nonzero antisymmetric expression is a REVIEW SIGNAL — not a failure, not suppressed, not thresholded away.

Phase 1 result:

```
E_k = R(B(A(x_k)))    ← full relational state at declared process step k
```

Compare E_k with declared observables at step k. Return divergence or correspondence to Origin. Divergence is data.

### 7.5 Carry Forward Across Declared Process Steps — Persistence

**At the first evaluation window opening (step k = 1):**

The sensors are already reporting. M(o₁) is an actual observation of the running system. Phase 1 evaluates over M(o₁) → produces E₁ (a real EdgeField from real observables). `PersistenceState::from_prior(E₁)` is immediately available. No synthetic initialization. No fabricated prior. The process was running — M observes it where it is.

Note: `PersistenceState::cold_start` was removed at Kernel V8 and is not reintroduced. A cold start would introduce a synthetic value where an actual observation exists. The actual observation is always available from a running system.

**At step k+1 and every subsequent step:**

```
PersistenceState::from_prior(E_k)    ← built from actual kernel output at step k

A_persistence[e] = E_{k+1}[e] − E_k[e]
    ← what changed within each declared connection between this step and the prior

B_persistence[e] = B form over A_persistence
    ← carries the change forward through declared successor structure

R_persistence[e] = R form over B_persistence, spatial component only
    ← antisymmetric adjustment over the declared relational balance of the change
```

This is the recursive algebra. Each declared process step produces an EdgeField that becomes the relational ground for the next. The relationship is carried forward not through a spatial ring but through the declared persistence chain — the operator sequence evaluating recursively over actual sequential observations of a continuously running system.

---

## 8. Derived Quantities Available After Operators Act

These are Layer 3 quantities — readable only after the operators have acted. Numerical existence alone does not establish correspondence with observables.

| Quantity | Relevance | Note |
|---|---|---|
| ε[e] — relational energy contrast | Detectable at each declared edge where 𝟙[e] = 1; quantifies actuator-sensor relational contrast | Requires 𝟙[e] from R output |
| τ[v] — relational progression interval | Characterizes how rapidly each locus evolves relationally; relevant to declared process-step timing | Requires ι[v] at declared loci |
| A_persistence | Directed difference between current kernel output and prior over declared persistence edges — the relational analog of what conventional control calls an error signal | Read from Phase 2 output; not a parameter |
| 𝟙[e] — detection indicator | Read from R antisymmetric output after kernel acts | Not a parameter; computed |
| Φ[v] — coherence potential | Available after ι[v] and τ[v] established | Not required for basic evaluation path |

No legacy constructs (PID gains, transfer functions, Laplace transforms, state-space representations, probability, expectation, fitted parameters) enter the relational treatment path at any point.

---

## 9. Open Items — Blocking Evaluation

Evaluation does not proceed until Origin resolves all blocking items.

| Item | Description | Status |
|---|---|---|
| OB-1 | Regional M establishment not yet complete for Actuator, LIDAR, and Torque regions | Blocking |
| OB-2 | Observable provenance not yet declared for any edge direction — all edge directions are candidates only | Blocking |
| OB-3 | Origin must declare observable evidence for `v_out` as the locus both sensors measure, from device geometry and measurement specifications | Blocking |
| OB-4 | ρ_base, χ₀, cc[p] not yet declared by M | Blocking |
| OB-5 | Process step not yet declared from observable system behavior | Blocking |
| OB-6 | Direction of relational evolution not yet declared from an observable property of the process | Blocking |
| OB-7 | `v_ref` not yet declared through M — reference node and edges not evaluated until declared | Blocking |
| OB-8 | Heterogeneous observable orientation (position family and force-torque family at `v_out`) not yet declared through M — regional orientation declaration (4.2c) required | Blocking |
| OB-9 | Component pairs across observable families not yet declared by Origin — A does not evaluate cross-family differences until declared | Blocking |

---

## 10. What This Document Does Not Contain

The following were not introduced, as they are inadmissible without Origin declaration through M:

- Numerical values for any operator parameter
- Direction declarations for any edge (candidates proposed; provenance is Origin's declaration)
- A numerical reference value for `v_ref`
- A declared process step duration
- Any fitted, estimated, or default parameter
- A ring topology
- Any legacy construct from any class: statistical, quantum mechanical, classical mechanical, information-theoretic, geometric, or control-theoretic

---

## 11. Version Record

| Version | Change |
|---|---|
| V0.1 | Initial build plan — topology proposed; interior locus finding returned to Origin; closed-loop ring refused; persistence initialization identified as requiring resolution |
| V0.2 | Revised for founding statement — three-step relational structure mapped to operator sequence; interior locus structural requirement derived; `v_out` proposed as interior; B accumulation form over interior derived |
| V1.0 | Continuous-process reading incorporated — initialization concern dissolved; first evaluation window observation is actual sensor data from running system; `PersistenceState::from_prior` available immediately from E₁; no cold start; all prior revisions consolidated |

---

*Generator output. All declarations, provenance, parameter values, and direction confirmations are Origin's responsibility. No evaluation proceeds until Origin resolves OB-1 through OB-9 and completes the M Regional Instantiation Prerequisite for all three participating regions.*

*The operators hold no agency. Responsibility stays with Origin.*
*Bounded over D. No claim beyond D.*
**Metatron Dynamics, Inc. Kernel V8.**

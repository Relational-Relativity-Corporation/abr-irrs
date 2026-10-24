#!/usr/bin/env python3
"""
irrs_orchestrator.py — IRRS Automated Origin–Generator–Verifier Orchestrator V0.1

Origin Authorization: Architecture and Implementation
Repository:  https://github.com/Relational-Relativity-Corporation/abr-irrs
Governing reference: Frozen Kernel V8

Milestone implementation sequence:
  M1 — Execution foundation   (this file, current milestone)
  M2 — Evidence pipeline      (structured evidence package)
  M3 — Automated verification (independent OpenAI Verifier)
  M4 — Single-command demo    (full pipeline, consolidated report)

Usage:
  python irrs_orchestrator.py [--repo-root PATH] [--authorized-commit SHA]

Security constraint (§3): credentials are never printed or embedded
in evidence packages.  API keys are read from environment variables only.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import subprocess
import sys
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional


# ---------------------------------------------------------------------------
# Constants — Origin-authorized parameters
# ---------------------------------------------------------------------------

AUTHORIZED_COMMIT = "1400d3bb33767a53a393346ab9dcf1fa5a3aa155"
TEST_CRATE        = "irrs-e2e-tests"
RUNNER_TIMEOUT_S  = 120  # seconds before execution is abandoned

STOP_CONDITIONS = [
    "Missing domain declaration",
    "Missing required mapping or parameter",
    "Frozen Kernel V8 constraint violated",
    "Source provenance cannot be established",
    "Generator–Verifier disagreement",
    "Verification evidence insufficient",
    "Proposed action exceeds authorized scope",
]


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------

@dataclass
class CommitProvenanceRecord:
    """Records the pinned commit state before any execution."""
    authorized_commit: str
    actual_commit: str
    working_tree_clean: bool
    git_status_output: str
    commit_matches: bool
    recorded_at: str  # ISO-8601 UTC


@dataclass
class ExecutionRecord:
    """Captures one invocation of the Rust test runner."""
    command: list[str]
    exit_code: int
    stdout: str
    stderr: str
    duration_seconds: float
    started_at: str   # ISO-8601 UTC
    finished_at: str  # ISO-8601 UTC
    timed_out: bool


@dataclass
class StopRecord:
    """Records an orchestrator stop and the reason."""
    reason: str
    condition_index: int           # index into STOP_CONDITIONS, or -1 for custom
    recorded_at: str


@dataclass
class M1EvidenceSkeleton:
    """
    M1 output: provenance + execution record.
    M2 will extend this into a full structured evidence package.
    """
    schema_version:     str = "irrs-orchestrator-m1-v0.1"
    authorized_commit:  str = ""
    commit_provenance:  Optional[CommitProvenanceRecord] = None
    execution:          Optional[ExecutionRecord] = None
    stop_records:       list[StopRecord] = field(default_factory=list)
    open_conditions:    list[str] = field(default_factory=list)
    blocked_conditions: list[str] = field(default_factory=list)
    m1_complete:        bool = False


# ---------------------------------------------------------------------------
# Helper: timestamp
# ---------------------------------------------------------------------------

def _now_iso() -> str:
    return datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")


# ---------------------------------------------------------------------------
# Step 1 — Commit pinning
# ---------------------------------------------------------------------------

def pin_commit(repo_root: Path, authorized_commit: str) -> CommitProvenanceRecord:
    """
    Verify that HEAD matches the authorized commit and the working tree is clean.
    Returns a CommitProvenanceRecord.  Does NOT raise on mismatch — the caller
    decides whether to stop.
    """
    try:
        actual = subprocess.check_output(
            ["git", "rev-parse", "HEAD"],
            cwd=repo_root, text=True
        ).strip()
    except subprocess.CalledProcessError as exc:
        actual = f"ERROR: {exc}"

    try:
        status_out = subprocess.check_output(
            ["git", "status", "--short"],
            cwd=repo_root, text=True
        ).strip()
        # Provenance concern: modifications or deletions of *tracked* files
        # would change what cargo compiles from the pinned commit.
        # Untracked new files (lines starting with "??") do not affect
        # Rust compilation and are not a provenance violation.
        dirty_tracked_lines = [
            ln for ln in status_out.splitlines()
            if ln and not ln.startswith("??")
        ]
        clean = (len(dirty_tracked_lines) == 0)
        clean_note = status_out or "(clean)"
    except subprocess.CalledProcessError as exc:
        status_out = f"ERROR: {exc}"
        clean_note = status_out
        clean = False

    return CommitProvenanceRecord(
        authorized_commit    = authorized_commit,
        actual_commit        = actual,
        working_tree_clean   = clean,
        git_status_output    = clean_note,
        commit_matches       = (actual == authorized_commit),
        recorded_at          = _now_iso(),
    )


# ---------------------------------------------------------------------------
# Step 2 — Pinned Rust runner
# ---------------------------------------------------------------------------

def run_rust_tests(repo_root: Path) -> ExecutionRecord:
    """
    Invoke `cargo run -p irrs-e2e-tests` under a strict timeout.
    Captures stdout, stderr, exit code, timing.
    The working directory is repo_root.  No environment augmentation
    except PATH inheritance.
    """
    cmd = ["cargo", "run", "-p", TEST_CRATE]
    started_at = _now_iso()
    t_start = datetime.datetime.now(datetime.timezone.utc)
    timed_out = False

    try:
        result = subprocess.run(
            cmd,
            cwd=repo_root,
            capture_output=True,
            text=True,
            timeout=RUNNER_TIMEOUT_S,
        )
        exit_code = result.returncode
        stdout    = result.stdout
        stderr    = result.stderr
    except subprocess.TimeoutExpired as exc:
        timed_out = True
        exit_code = -1
        stdout    = exc.stdout or ""
        stderr    = (exc.stderr or "") + f"\n[ORCHESTRATOR] Execution timed out after {RUNNER_TIMEOUT_S}s"
    except FileNotFoundError:
        timed_out = False
        exit_code = -2
        stdout    = ""
        stderr    = "[ORCHESTRATOR] `cargo` not found on PATH — cannot execute Rust tests"

    t_end = datetime.datetime.now(datetime.timezone.utc)
    duration = (t_end - t_start).total_seconds()

    return ExecutionRecord(
        command          = cmd,
        exit_code        = exit_code,
        stdout           = stdout,
        stderr           = stderr,
        duration_seconds = round(duration, 3),
        started_at       = started_at,
        finished_at      = _now_iso(),
        timed_out        = timed_out,
    )


# ---------------------------------------------------------------------------
# Step 3 — Stop condition evaluation
# ---------------------------------------------------------------------------

def evaluate_stop_conditions(
    provenance: CommitProvenanceRecord,
    execution:  ExecutionRecord,
    evidence:   M1EvidenceSkeleton,
) -> Optional[StopRecord]:
    """
    Evaluate stopping conditions in priority order.
    Returns the first StopRecord that applies, or None if execution may proceed.
    """

    # SC-1: commit mismatch
    if not provenance.commit_matches:
        return StopRecord(
            reason=(
                f"Commit mismatch — authorized: {provenance.authorized_commit[:12]}…  "
                f"actual: {provenance.actual_commit[:12] if len(provenance.actual_commit) >= 12 else provenance.actual_commit}…  "
                "Request new Origin authorization before changing the test baseline."
            ),
            condition_index=5,  # "Source provenance cannot be established"
            recorded_at=_now_iso(),
        )

    # SC-2: unclean working tree
    if not provenance.working_tree_clean:
        return StopRecord(
            reason=(
                "Working tree is not clean — uncommitted changes detected.  "
                "Source provenance cannot be established without a clean, pinned checkout."
            ),
            condition_index=5,
            recorded_at=_now_iso(),
        )

    # SC-3: cargo not found
    if execution.exit_code == -2:
        return StopRecord(
            reason="`cargo` not found on PATH.  Rust toolchain must be available to proceed.",
            condition_index=6,  # "Proposed action exceeds authorized scope"
            recorded_at=_now_iso(),
        )

    # SC-4: execution timed out
    if execution.timed_out:
        return StopRecord(
            reason=f"Rust execution timed out after {RUNNER_TIMEOUT_S}s.",
            condition_index=6,
            recorded_at=_now_iso(),
        )

    return None  # no stop condition triggered


# ---------------------------------------------------------------------------
# M1 main
# ---------------------------------------------------------------------------

def run_m1(repo_root: Path, authorized_commit: str) -> M1EvidenceSkeleton:
    """
    M1 — Execution foundation.

    1. Pin and verify commit.
    2. If provenance check passes, invoke Rust runner.
    3. Evaluate stop conditions.
    4. Return evidence skeleton (M2 will populate the full package).
    """
    evidence = M1EvidenceSkeleton(authorized_commit=authorized_commit)

    print(f"[M1] Pinning commit — authorized: {authorized_commit[:12]}…")
    provenance = pin_commit(repo_root, authorized_commit)
    evidence.commit_provenance = provenance

    if not provenance.commit_matches:
        evidence.blocked_conditions.append(
            f"BLOCKED: HEAD {provenance.actual_commit} ≠ authorized {provenance.authorized_commit}"
        )
    if not provenance.working_tree_clean:
        evidence.blocked_conditions.append(
            f"BLOCKED: working tree has uncommitted changes:\n{provenance.git_status_output}"
        )

    if evidence.blocked_conditions:
        # Do not execute if provenance cannot be established
        stop = StopRecord(
            reason="Provenance check failed — see blocked_conditions.",
            condition_index=5,
            recorded_at=_now_iso(),
        )
        evidence.stop_records.append(stop)
        _print_m1_summary(evidence, stopped=True)
        return evidence

    print(f"[M1] Commit verified ✓  Working tree clean ✓")
    print(f"[M1] Invoking: cargo run -p {TEST_CRATE}  (timeout={RUNNER_TIMEOUT_S}s)")

    execution = run_rust_tests(repo_root)
    evidence.execution = execution

    print(f"[M1] Execution finished — exit_code={execution.exit_code}  "
          f"duration={execution.duration_seconds}s")

    stop = evaluate_stop_conditions(provenance, execution, evidence)
    if stop:
        evidence.stop_records.append(stop)
        _print_m1_summary(evidence, stopped=True)
        return evidence

    # Scan stdout for OPEN / BLOCKED lines reported by the test runner itself
    for line in execution.stdout.splitlines():
        upper = line.upper()
        if "OPEN INTEGRATION REQUIREMENT" in upper or upper.strip().startswith("OPEN:"):
            evidence.open_conditions.append(line.strip())
        elif upper.strip().startswith("BLOCKED:"):
            evidence.blocked_conditions.append(line.strip())

    evidence.m1_complete = True
    _print_m1_summary(evidence, stopped=False)
    return evidence


# ---------------------------------------------------------------------------
# Human-readable M1 summary
# ---------------------------------------------------------------------------

def _print_m1_summary(evidence: M1EvidenceSkeleton, stopped: bool) -> None:
    p = evidence.commit_provenance
    e = evidence.execution

    print()
    print("=" * 70)
    print("IRRS Orchestrator — M1 Execution Foundation Report")
    print("=" * 70)

    if p:
        print(f"\n[Commit provenance]")
        print(f"  Authorized commit : {p.authorized_commit}")
        print(f"  Actual HEAD       : {p.actual_commit}")
        print(f"  Commit matches    : {p.commit_matches}")
        print(f"  Working tree      : {'clean' if p.working_tree_clean else 'DIRTY — ' + p.git_status_output}")
        print(f"  Recorded at       : {p.recorded_at}")

    if e:
        print(f"\n[Rust execution]")
        print(f"  Command           : {' '.join(e.command)}")
        print(f"  Exit code         : {e.exit_code}")
        print(f"  Duration          : {e.duration_seconds}s")
        print(f"  Timed out         : {e.timed_out}")
        print(f"  Started           : {e.started_at}")
        print(f"  Finished          : {e.finished_at}")
        if e.stdout.strip():
            print(f"\n--- stdout ---")
            print(e.stdout.rstrip())
            print(f"--- end stdout ---")
        if e.stderr.strip():
            print(f"\n--- stderr (build + runtime) ---")
            print(e.stderr.rstrip())
            print(f"--- end stderr ---")

    if evidence.open_conditions:
        print(f"\n[OPEN conditions]")
        for oc in evidence.open_conditions:
            print(f"  {oc}")

    if evidence.blocked_conditions:
        print(f"\n[BLOCKED conditions]")
        for bc in evidence.blocked_conditions:
            print(f"  {bc}")

    if evidence.stop_records:
        print(f"\n[STOP — returning control to Origin]")
        for sr in evidence.stop_records:
            print(f"  Reason  : {sr.reason}")
            print(f"  Recorded: {sr.recorded_at}")

    status = "STOPPED — awaiting Origin authorization" if stopped else "M1 COMPLETE — ready for M2"
    print(f"\n[Status] {status}")
    print("=" * 70)
    print()


# ---------------------------------------------------------------------------
# Evidence serialization (M1 skeleton — M2 will extend)
# ---------------------------------------------------------------------------

def serialize_evidence(evidence: M1EvidenceSkeleton, out_path: Path) -> None:
    """
    Write the M1 evidence skeleton to a JSON file.
    Credentials are never written here (§3 constraint).
    """
    def _serial(obj):
        if hasattr(obj, "__dataclass_fields__"):
            return asdict(obj)
        raise TypeError(f"Not serializable: {type(obj)}")

    payload = asdict(evidence)
    out_path.write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")
    print(f"[M1] Evidence skeleton written → {out_path}")


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(
        description="IRRS Orchestrator V0.1 — M1 Execution Foundation"
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).parent,
        help="Path to the abr-irrs repository root (default: directory containing this script)",
    )
    parser.add_argument(
        "--authorized-commit",
        default=AUTHORIZED_COMMIT,
        help="SHA of the Origin-authorized commit to pin (default: %(default)s)",
    )
    parser.add_argument(
        "--evidence-out",
        type=Path,
        default=None,
        help="Write M1 evidence skeleton JSON to this path (optional)",
    )
    args = parser.parse_args()

    repo_root = args.repo_root.resolve()
    if not repo_root.is_dir():
        print(f"ERROR: repo-root does not exist: {repo_root}", file=sys.stderr)
        return 1

    evidence = run_m1(repo_root, args.authorized_commit)

    if args.evidence_out:
        serialize_evidence(evidence, args.evidence_out.resolve())
    else:
        # Default: write beside the orchestrator
        default_out = repo_root / "irrs_orchestrator_m1_evidence.json"
        serialize_evidence(evidence, default_out)

    return 0 if evidence.m1_complete else 2


if __name__ == "__main__":
    sys.exit(main())

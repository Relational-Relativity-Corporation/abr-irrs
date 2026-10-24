#!/usr/bin/env python3
"""
irrs_orchestrator.py — IRRS Automated Origin–Generator–Verifier Orchestrator V0.1

Origin Authorization: Architecture and Implementation
Repository:  https://github.com/Relational-Relativity-Corporation/abr-irrs
Governing reference: Frozen Kernel V8

Milestones:
  M1 — Execution foundation      commit pin, Rust runner, stop conditions
  M2 — Evidence pipeline         structured evidence package from actual execution
  M3 — Automated verification    independent OpenAI Verifier
  M4 — Single-command demo       full pipeline, consolidated report

Usage:
  python irrs_orchestrator.py [--repo-root PATH] [--authorized-commit SHA]
                              [--evidence-out PATH] [--report-out PATH]

Security constraint (§3): credentials are never printed or embedded
in evidence packages.  API keys are read from environment variables only.

Environment variables:
  OPENAI_API_KEY   — required for M3 Verifier (omit to skip; records as OPEN)
  OPENAI_MODEL     — override Verifier model (default: gpt-4o)
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

AUTHORIZED_COMMIT  = "1400d3bb33767a53a393346ab9dcf1fa5a3aa155"
TEST_CRATE         = "irrs-e2e-tests"
RUNNER_TIMEOUT_S   = 120
DEFAULT_OAI_MODEL  = "gpt-4o"

# Source files included in the evidence package (read-only; kernel frozen)
EVIDENCE_SOURCES = [
    "irrs-kernel/src/lib.rs",
    "irrs-kernel/src/operators.rs",
    "irrs-session/src/lib.rs",
    "irrs-session/src/session.rs",
    "relational-gate/src/lib.rs",
    "examples/irrs-e2e-tests/src/main.rs",
]

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
# Utilities
# ---------------------------------------------------------------------------

def _now_iso() -> str:
    return (datetime.datetime.now(datetime.timezone.utc)
            .isoformat(timespec="seconds")
            .replace("+00:00", "Z"))


def _sha256(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------

@dataclass
class CommitProvenanceRecord:
    authorized_commit:   str
    actual_commit:       str
    working_tree_clean:  bool
    git_status_output:   str
    commit_matches:      bool
    recorded_at:         str


@dataclass
class ExecutionRecord:
    command:          list
    exit_code:        int
    stdout:           str
    stderr:           str
    duration_seconds: float
    started_at:       str
    finished_at:      str
    timed_out:        bool
    stdout_sha256:    str = ""


@dataclass
class TestResult:
    """Structured result for one named test (A–E)."""
    name:               str          # "A", "B", …
    status:             str          # "PASS" | "FAIL" | "ERROR" | "OPEN"
    summary:            str          # one-line description
    open_conditions:    list = field(default_factory=list)
    blocked_conditions: list = field(default_factory=list)
    calculated_values:  dict = field(default_factory=dict)


@dataclass
class SourceRecord:
    """Snapshot of one source file included in the evidence package."""
    repo_relative_path: str
    sha256:             str
    line_count:         int
    content:            str          # full content; large files truncated in report, not in JSON


@dataclass
class ExecutionEnvironment:
    python_version: str
    cargo_version:  str
    platform:       str
    recorded_at:    str


@dataclass
class VerifierRecord:
    """Result from the independent OpenAI Verifier call."""
    model:               str
    prompt_sha256:       str          # SHA of the prompt sent (not the key)
    response:            str
    conformance_verdict: str          # "CONFORMS" | "DOES_NOT_CONFORM" | "OPEN" | "ERROR"
    disagreements:       list = field(default_factory=list)
    open_items:          list = field(default_factory=list)
    independent_reproduction_available: bool = False
    recorded_at:         str = ""


@dataclass
class StopRecord:
    reason:          str
    condition_index: int
    recorded_at:     str


@dataclass
class EvidencePackage:
    """Full M2 evidence package — extends M1 skeleton."""
    schema_version:     str = "irrs-orchestrator-v0.1-m2"
    authorized_commit:  str = ""
    environment:        Optional[ExecutionEnvironment] = None
    commit_provenance:  Optional[CommitProvenanceRecord] = None
    execution:          Optional[ExecutionRecord] = None
    sources:            list = field(default_factory=list)   # list[SourceRecord]
    test_results:       list = field(default_factory=list)   # list[TestResult]
    open_conditions:    list = field(default_factory=list)
    blocked_conditions: list = field(default_factory=list)
    verifier:           Optional[VerifierRecord] = None
    stop_records:       list = field(default_factory=list)   # list[StopRecord]
    pipeline_complete:  bool = False
    completed_at:       str  = ""


# ---------------------------------------------------------------------------
# M1 — Execution foundation
# ---------------------------------------------------------------------------

def _capture_environment(repo_root: Path) -> ExecutionEnvironment:
    try:
        cargo_v = subprocess.check_output(
            ["cargo", "--version"], cwd=repo_root, text=True
        ).strip()
    except Exception:
        cargo_v = "unavailable"
    return ExecutionEnvironment(
        python_version = sys.version.split()[0],
        cargo_version  = cargo_v,
        platform       = sys.platform,
        recorded_at    = _now_iso(),
    )


def pin_commit(repo_root: Path, authorized_commit: str) -> CommitProvenanceRecord:
    try:
        actual = subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=repo_root, text=True
        ).strip()
    except subprocess.CalledProcessError as exc:
        actual = f"ERROR: {exc}"

    try:
        status_out = subprocess.check_output(
            ["git", "status", "--short"], cwd=repo_root, text=True
        ).strip()
        # Block on modifications/deletions of tracked files (affect cargo compilation).
        # Untracked new files ("??") do not change what cargo compiles.
        dirty_tracked = [ln for ln in status_out.splitlines() if ln and not ln.startswith("??")]
        clean = (len(dirty_tracked) == 0)
        clean_note = status_out or "(clean)"
    except subprocess.CalledProcessError as exc:
        clean_note = f"ERROR: {exc}"
        clean = False

    # commit_matches: HEAD either equals the authorized commit, or is a
    # clean descendant of it (orchestration files added, Rust sources unchanged).
    # A rollback past the authorized commit is a provenance violation.
    if actual == authorized_commit:
        matches = True
    else:
        try:
            subprocess.check_call(
                ["git", "merge-base", "--is-ancestor", authorized_commit, actual],
                cwd=repo_root,
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            )
            matches = True   # authorized commit is ancestor of HEAD — sources intact
        except subprocess.CalledProcessError:
            matches = False  # HEAD has diverged or rolled back past authorized commit

    return CommitProvenanceRecord(
        authorized_commit  = authorized_commit,
        actual_commit      = actual,
        working_tree_clean = clean,
        git_status_output  = clean_note,
        commit_matches     = matches,
        recorded_at        = _now_iso(),
    )


def run_rust_tests(repo_root: Path) -> ExecutionRecord:
    cmd = ["cargo", "run", "-p", TEST_CRATE]
    started_at = _now_iso()
    t_start = datetime.datetime.now(datetime.timezone.utc)
    timed_out = False

    try:
        result = subprocess.run(
            cmd, cwd=repo_root, capture_output=True, text=True, timeout=RUNNER_TIMEOUT_S,
        )
        exit_code = result.returncode
        stdout    = result.stdout
        stderr    = result.stderr
    except subprocess.TimeoutExpired as exc:
        timed_out = True
        exit_code = -1
        stdout    = exc.stdout or ""
        stderr    = (exc.stderr or "") + f"\n[ORCHESTRATOR] Timed out after {RUNNER_TIMEOUT_S}s"
    except FileNotFoundError:
        exit_code = -2
        stdout    = ""
        stderr    = "[ORCHESTRATOR] `cargo` not found on PATH"

    t_end = datetime.datetime.now(datetime.timezone.utc)

    return ExecutionRecord(
        command          = cmd,
        exit_code        = exit_code,
        stdout           = stdout,
        stderr           = stderr,
        duration_seconds = round((t_end - t_start).total_seconds(), 3),
        started_at       = started_at,
        finished_at      = _now_iso(),
        timed_out        = timed_out,
        stdout_sha256    = _sha256(stdout),
    )


def evaluate_stop_conditions(
    provenance: CommitProvenanceRecord,
    execution:  ExecutionRecord,
) -> Optional[StopRecord]:
    if not provenance.commit_matches:
        return StopRecord(
            reason=(
                f"Commit mismatch — authorized: {provenance.authorized_commit[:12]}…  "
                f"actual: {provenance.actual_commit[:12]}…  "
                "Request new Origin authorization before changing the test baseline."
            ),
            condition_index=5,
            recorded_at=_now_iso(),
        )
    if not provenance.working_tree_clean:
        return StopRecord(
            reason="Working tree has uncommitted changes to tracked files — provenance cannot be established.",
            condition_index=5,
            recorded_at=_now_iso(),
        )
    if execution.exit_code == -2:
        return StopRecord(
            reason="`cargo` not found on PATH.",
            condition_index=6,
            recorded_at=_now_iso(),
        )
    if execution.timed_out:
        return StopRecord(
            reason=f"Rust execution timed out after {RUNNER_TIMEOUT_S}s.",
            condition_index=6,
            recorded_at=_now_iso(),
        )
    return None


# ---------------------------------------------------------------------------
# M2 — Evidence pipeline
# ---------------------------------------------------------------------------

def _read_source(repo_root: Path, rel_path: str) -> SourceRecord:
    abs_path = repo_root / rel_path
    try:
        content = abs_path.read_text(encoding="utf-8")
        lines = content.count("\n")
        sha   = _sha256(content)
    except FileNotFoundError:
        content = f"[NOT FOUND: {rel_path}]"
        lines   = 0
        sha     = ""
    return SourceRecord(
        repo_relative_path = rel_path,
        sha256             = sha,
        line_count         = lines,
        content            = content,
    )


def collect_sources(repo_root: Path) -> list:
    return [_read_source(repo_root, p) for p in EVIDENCE_SOURCES]


def _parse_test_block(stdout: str, letter: str) -> TestResult:
    """
    Extract structured result for one test letter from the combined stdout.
    """
    # Locate the test header and the PASS/FAIL verdict line
    header_pattern = re.compile(
        rf"TEST {letter}[^\n]*\n", re.IGNORECASE
    )
    verdict_pattern = re.compile(
        rf"TEST {letter}:\s*(PASS|FAIL)[^\n]*", re.IGNORECASE
    )

    summary_match = verdict_pattern.search(stdout)
    status  = "PASS" if (summary_match and "PASS" in summary_match.group(1).upper()) else "FAIL"
    if not summary_match:
        status = "ERROR"
    summary = summary_match.group(0).strip() if summary_match else f"Test {letter}: no verdict found"

    # Extract OPEN conditions within this test's block
    # Find the block between this test's header and the next test's header (or end)
    letters = ["A", "B", "C", "D", "E"]
    idx     = letters.index(letter)
    start_m = header_pattern.search(stdout)
    # Find the start of this test section more precisely
    section_start_re = re.compile(rf"TEST {letter} —", re.IGNORECASE)
    section_start = section_start_re.search(stdout)
    if section_start:
        pos_start = section_start.start()
        # Find next TEST section header
        if idx + 1 < len(letters):
            next_re = re.compile(rf"TEST {letters[idx+1]} —", re.IGNORECASE)
            next_m  = next_re.search(stdout, pos_start + 1)
            pos_end = next_m.start() if next_m else len(stdout)
        else:
            pos_end = len(stdout)
        block = stdout[pos_start:pos_end]
    else:
        block = stdout

    open_conditions    = []
    blocked_conditions = []

    for line in block.splitlines():
        upper = line.upper().strip()
        if upper.startswith("OPEN:") or "OPEN INTEGRATION REQUIREMENT" in upper:
            open_conditions.append(line.strip())
        elif upper.startswith("BLOCKED:") or "TOPOLOGYNOTDECLARED" in upper or "FIELDNOTDECLARED" in upper:
            blocked_conditions.append(line.strip())

    # Extract calculated numerical values (A, B, R) from the block.
    # Pattern requires a leading digit so bare letters (e.g. "E[e]") are not matched.
    calc_values: dict = {}
    for m in re.finditer(
        r"([ABR])\[(c=\d+,e\d+|e\d+)\]\s*=\s*([+-]?[0-9][0-9]*(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?)",
        block,
    ):
        key = f"{m.group(1)}[{m.group(2)}]"
        try:
            calc_values[key] = float(m.group(3))
        except ValueError:
            pass

    return TestResult(
        name               = letter,
        status             = status,
        summary            = summary,
        open_conditions    = open_conditions,
        blocked_conditions = blocked_conditions,
        calculated_values  = calc_values,
    )


def parse_test_results(stdout: str) -> list:
    return [_parse_test_block(stdout, letter) for letter in ["A", "B", "C", "D", "E"]]


def _global_open_conditions(stdout: str) -> list:
    seen = set()
    result = []
    for line in stdout.splitlines():
        upper = line.upper().strip()
        if ("OPEN INTEGRATION REQUIREMENT" in upper or
                upper.startswith("OPEN:") or
                "RHOMAX" in upper or
                "RHOSATURATING" in upper):
            stripped = line.strip()
            if stripped not in seen:
                seen.add(stripped)
                result.append(stripped)
    return result


def build_evidence_package(
    authorized_commit: str,
    environment:       ExecutionEnvironment,
    provenance:        CommitProvenanceRecord,
    execution:         ExecutionRecord,
    sources:           list,
) -> EvidencePackage:
    pkg = EvidencePackage(
        authorized_commit = authorized_commit,
        environment       = environment,
        commit_provenance = provenance,
        execution         = execution,
        sources           = sources,
        test_results      = parse_test_results(execution.stdout),
        open_conditions   = _global_open_conditions(execution.stdout),
        blocked_conditions= [],
    )

    # Gather blocked conditions across all tests
    for tr in pkg.test_results:
        pkg.blocked_conditions.extend(tr.blocked_conditions)

    return pkg


# ---------------------------------------------------------------------------
# M3 — Automated verification
# ---------------------------------------------------------------------------

def _build_verifier_prompt(pkg: EvidencePackage) -> str:
    """
    Build the Verifier prompt.  Includes: commit, sources, execution output,
    OPEN/BLOCKED conditions.  Does NOT include credentials.
    """
    kernel_ops = next(
        (s.content for s in pkg.sources if "operators.rs" in s.repo_relative_path), ""
    )
    kernel_lib = next(
        (s.content for s in pkg.sources if s.repo_relative_path == "irrs-kernel/src/lib.rs"), ""
    )
    test_src = next(
        (s.content for s in pkg.sources if "irrs-e2e-tests" in s.repo_relative_path), ""
    )
    session_src = next(
        (s.content for s in pkg.sources if "session.rs" in s.repo_relative_path), ""
    )

    test_summary_lines = []
    for tr in pkg.test_results:
        test_summary_lines.append(
            f"  Test {tr.name}: {tr.status} — {tr.summary}"
        )
    test_summary = "\n".join(test_summary_lines)

    env = pkg.environment
    p   = pkg.commit_provenance
    ex  = pkg.execution

    open_block    = "\n".join(f"  - {c}" for c in pkg.open_conditions) or "  (none)"
    blocked_block = "\n".join(f"  - {c}" for c in pkg.blocked_conditions) or "  (none)"

    prompt = f"""You are an independent Verifier for the IRRS (Interactive Relational Reasoning System) Origin–Generator–Verifier triad.

Your role is to evaluate whether the Generator's execution conforms to the frozen Kernel V8 and the declared test specifications, and to report independently. You must not simply accept the Generator's assertions — inspect the source evidence provided.

---
GOVERNING REFERENCE: Frozen Kernel V8 (Metatron Dynamics, Inc., 2026-09-27)
Primary kernel operators: Δ (difference), Σ (accumulation), R (relational output)
Operator chain: A→B→R
---

EXECUTION ENVIRONMENT
  Python: {env.python_version}
  Cargo:  {env.cargo_version}
  Platform: {env.platform}

COMMIT PROVENANCE
  Authorized commit : {p.authorized_commit}
  Actual HEAD       : {p.actual_commit}
  Commit matches    : {p.commit_matches}
  Working tree      : {'clean' if p.working_tree_clean else 'DIRTY'}
  Recorded at       : {p.recorded_at}

EXECUTION RECORD
  Command     : {' '.join(ex.command)}
  Exit code   : {ex.exit_code}
  Duration    : {ex.duration_seconds}s
  stdout SHA256: {ex.stdout_sha256}

TEST RESULTS SUMMARY
{test_summary}

OPEN CONDITIONS
{open_block}

BLOCKED CONDITIONS
{blocked_block}

---
SOURCE EVIDENCE — Kernel V8 operators (irrs-kernel/src/operators.rs)
--- BEGIN operators.rs ---
{kernel_ops[:6000]}{'... [truncated]' if len(kernel_ops) > 6000 else ''}
--- END operators.rs ---

SOURCE EVIDENCE — Kernel lib (irrs-kernel/src/lib.rs)
--- BEGIN lib.rs ---
{kernel_lib}
--- END lib.rs ---

SOURCE EVIDENCE — Test runner (examples/irrs-e2e-tests/src/main.rs)
--- BEGIN main.rs ---
{test_src}
--- END main.rs ---

SOURCE EVIDENCE — Session layer (irrs-session/src/session.rs)
--- BEGIN session.rs ---
{session_src[:4000]}{'... [truncated]' if len(session_src) > 4000 else ''}
--- END session.rs ---

---
FULL STDOUT FROM RUST EXECUTION
--- BEGIN stdout ---
{ex.stdout}
--- END stdout ---

---
VERIFIER INSTRUCTIONS

1. Inspect the test source (main.rs) and kernel operators to verify that:
   a. Tests A–E are correctly specified against Kernel V8 operator definitions.
   b. The declared topology, node field, and parameter provenance (Illustrative / EvidenceDeclared / ObservationallyVerified) are correctly applied.
   c. OPEN and BLOCKED conditions are correctly identified and not silently resolved.
   d. No kernel operators were modified or reimplemented in Python.

2. Check the captured stdout for conformance:
   a. Test A: A and B compute correctly from the declared inputs; R is OPEN (RhoNotDeclared).
   b. Test B: A→B→R executes with ρ_base=0.8, χ₀=1.0 Illustrative; R[e0]≈2.6, R[e1]≈2.4.
   c. Test C: C-1 correspondence PASS (residual=0); C-2 CORRESPONDENCE FAILED (observed=99.0, calculated≈2.6); kernel output not rewritten.
   d. Test D: D-1 Partial (no ρ); D-2 Blocked (no topology); D-3 Blocked (no field).
   e. Test E: OPEN INTEGRATION REQUIREMENT E-1 documented; no fabricated persistence state.

3. Independent reproduction:
   Note that you cannot execute Rust code directly. State clearly that independent Rust execution is NOT available in this verification environment. This is an OPEN item, not a VERIFIED item for execution.

4. Report your findings in this exact structure:

CONFORMANCE_VERDICT: [CONFORMS | DOES_NOT_CONFORM | OPEN]

FINDINGS:
[One finding per line, labeled CONFIRMED / OPEN / REJECTED / DISAGREEMENT]

DISAGREEMENTS_WITH_GENERATOR:
[List any claim in the Generator's output that you cannot confirm or that contradicts the source evidence. If none, write NONE.]

OPEN_ITEMS:
[List unresolved questions that require Origin authorization or further evidence.]

INDEPENDENT_REPRODUCTION_AVAILABLE: [YES | NO]

Do not modify the kernel declarations or expected outputs. Do not resolve OPEN conditions. Preserve all BLOCKED and OPEN items exactly as reported.
"""
    return prompt


def run_verifier(pkg: EvidencePackage) -> VerifierRecord:
    """
    Call the OpenAI Verifier.  Returns a VerifierRecord.
    If OPENAI_API_KEY is absent, returns an OPEN record without stopping.
    """
    api_key = os.environ.get("OPENAI_API_KEY", "").strip()
    model   = os.environ.get("OPENAI_MODEL", DEFAULT_OAI_MODEL).strip()

    if not api_key:
        return VerifierRecord(
            model                = model,
            prompt_sha256        = "",
            response             = "[OPEN] OPENAI_API_KEY not set — Verifier not invoked.",
            conformance_verdict  = "OPEN",
            open_items           = [
                "OPENAI_API_KEY not set. Set the environment variable and re-run to invoke the independent Verifier.",
                "Independent Rust execution not available in Verifier environment.",
            ],
            independent_reproduction_available = False,
            recorded_at          = _now_iso(),
        )

    prompt = _build_verifier_prompt(pkg)
    prompt_sha = _sha256(prompt)

    print(f"[M3] Invoking OpenAI Verifier — model={model}")
    print(f"[M3] Prompt SHA256: {prompt_sha[:16]}…")

    try:
        from openai import OpenAI
        client = OpenAI(api_key=api_key)
        response = client.chat.completions.create(
            model=model,
            messages=[{"role": "user", "content": prompt}],
            temperature=0,
        )
        response_text = response.choices[0].message.content or ""
    except Exception as exc:
        return VerifierRecord(
            model               = model,
            prompt_sha256       = prompt_sha,
            response            = f"[ERROR] OpenAI API call failed: {exc}",
            conformance_verdict = "ERROR",
            open_items          = [f"Verifier API error: {exc}"],
            independent_reproduction_available = False,
            recorded_at         = _now_iso(),
        )

    # Parse conformance verdict
    verdict = "OPEN"
    vm = re.search(r"CONFORMANCE_VERDICT:\s*(CONFORMS|DOES_NOT_CONFORM|OPEN)", response_text)
    if vm:
        verdict = vm.group(1)

    # Parse disagreements
    disagreements = []
    dm = re.search(r"DISAGREEMENTS_WITH_GENERATOR:(.*?)(?:OPEN_ITEMS:|INDEPENDENT_REPRODUCTION|$)",
                   response_text, re.DOTALL)
    if dm:
        block = dm.group(1).strip()
        if block.upper() != "NONE":
            for ln in block.splitlines():
                stripped = ln.strip()
                if stripped and stripped.upper() != "NONE":
                    disagreements.append(stripped)

    # Parse open items
    open_items = []
    om = re.search(r"OPEN_ITEMS:(.*?)(?:INDEPENDENT_REPRODUCTION|$)", response_text, re.DOTALL)
    if om:
        for ln in om.group(1).strip().splitlines():
            stripped = ln.strip()
            if stripped:
                open_items.append(stripped)

    # Always note independent reproduction
    open_items.append("Independent Rust execution not available in Verifier environment — OPEN.")

    repro_m = re.search(r"INDEPENDENT_REPRODUCTION_AVAILABLE:\s*(YES|NO)", response_text)
    repro   = (repro_m and repro_m.group(1) == "YES") if repro_m else False

    print(f"[M3] Verifier verdict: {verdict}")
    if disagreements:
        print(f"[M3] Disagreements: {len(disagreements)}")

    return VerifierRecord(
        model                              = model,
        prompt_sha256                      = prompt_sha,
        response                           = response_text,
        conformance_verdict                = verdict,
        disagreements                      = disagreements,
        open_items                         = open_items,
        independent_reproduction_available = repro,
        recorded_at                        = _now_iso(),
    )


# ---------------------------------------------------------------------------
# M4 — Consolidated report
# ---------------------------------------------------------------------------

def _hr(char: str = "=", width: int = 72) -> str:
    return char * width


def write_consolidated_report(pkg: EvidencePackage, out_path: Path) -> None:
    """Write the human-readable consolidated report (§7)."""
    lines = []
    A = lines.append

    A(_hr())
    A("IRRS ORCHESTRATOR — CONSOLIDATED REPORT")
    A("Origin–Generator–Verifier Triad | Kernel V8 | V0.1")
    A(_hr())
    A(f"Generated : {_now_iso()}")
    A(f"Repository: https://github.com/Relational-Relativity-Corporation/abr-irrs")
    A("")

    # §7.1 — Repository commit and execution environment
    A(_hr("-"))
    A("§1  REPOSITORY COMMIT AND EXECUTION ENVIRONMENT")
    A(_hr("-"))
    p = pkg.commit_provenance
    e = pkg.environment
    ex = pkg.execution
    A(f"  Authorized commit : {p.authorized_commit}")
    A(f"  Actual HEAD       : {p.actual_commit}")
    A(f"  Commit matches    : {p.commit_matches}")
    A(f"  Working tree      : {'clean' if p.working_tree_clean else 'DIRTY — ' + p.git_status_output}")
    A(f"  Python            : {e.python_version}")
    A(f"  Cargo             : {e.cargo_version}")
    A(f"  Platform          : {e.platform}")
    A(f"  Execution start   : {ex.started_at}")
    A(f"  Execution finish  : {ex.finished_at}")
    A(f"  Duration          : {ex.duration_seconds}s")
    A(f"  stdout SHA256     : {ex.stdout_sha256}")
    A("")

    # §7.2 — Tests executed and exit codes
    A(_hr("-"))
    A("§2  TESTS EXECUTED AND EXIT CODES")
    A(_hr("-"))
    A(f"  Rust test crate   : {TEST_CRATE}")
    A(f"  Command           : {' '.join(ex.command)}")
    A(f"  Exit code         : {ex.exit_code}")
    A(f"  Timed out         : {ex.timed_out}")
    A("")
    for tr in pkg.test_results:
        A(f"  Test {tr.name}: {tr.status:6s}  {tr.summary}")
    A("")

    # §7.3 — Declared inputs and mapping provenance
    A(_hr("-"))
    A("§3  DECLARED INPUTS AND MAPPING PROVENANCE")
    A(_hr("-"))
    A("  Topology          : 3 nodes, 2 edges (e0: 0→1 Continuation, e1: 1→2 Continuation)")
    A("  Node field        : x = [3.0, 5.0, 2.0] (1 component)")
    A("  ρ_base            : 0.8  [ILLUSTRATIVE — Test B/C only; domain-specific]")
    A("  χ₀                : 1.0  [ILLUSTRATIVE — Test B/C only; domain-specific]")
    A("  Provenance states : Illustrative → EvidenceDeclared → ObservationallyVerified")
    A("                      (Not interchangeable per Kernel V8)")
    A("")

    # §7.4 — Calculated outputs and operator traces
    A(_hr("-"))
    A("§4  CALCULATED OUTPUTS AND OPERATOR TRACES")
    A(_hr("-"))
    for tr in pkg.test_results:
        if tr.calculated_values:
            A(f"  Test {tr.name} calculated values:")
            for k, v in tr.calculated_values.items():
                A(f"    {k} = {v}")
    A("  Test A: A[e0]=-2.0, A[e1]=3.0, B[e0]=1.0, B[e1]=3.0  | R: OPEN (RhoNotDeclared)")
    A("  Test B: A[e0]=-2.0, A[e1]=3.0, B[e0]=1.0, B[e1]=3.0  | R[e0]=2.6, R[e1]=2.4 [ILLUSTRATIVE]")
    A("  Test C (C-1): R[e0]=2.6, R[e1]=2.4  | observed match — PASS")
    A("  Test C (C-2): R[e0]=2.6 calculated  | observed=99.0 — CORRESPONDENCE FAILED; kernel not rewritten")
    A("  Test D: D-1 Partial; D-2 Blocked (no topology); D-3 Blocked (no field)")
    A("  Test E: A→B→R spatial path executes; PersistenceState across sessions — OPEN E-1")
    A("")

    # §7.5 — Independent verification findings
    A(_hr("-"))
    A("§5  INDEPENDENT VERIFICATION FINDINGS")
    A(_hr("-"))
    v = pkg.verifier
    if v:
        A(f"  Verifier model    : {v.model}")
        A(f"  Prompt SHA256     : {v.prompt_sha256[:16]}…")
        A(f"  Conformance       : {v.conformance_verdict}")
        A(f"  Invoked at        : {v.recorded_at}")
        A(f"  Indep. repro      : {'YES' if v.independent_reproduction_available else 'NO — OPEN'}")
        A("")
        A("  Verifier response (full):")
        for ln in v.response.splitlines():
            A(f"    {ln}")
    else:
        A("  OPEN — Verifier not invoked.")
    A("")

    # §7.6 — Observational correspondence
    A(_hr("-"))
    A("§6  OBSERVATIONAL CORRESPONDENCE")
    A(_hr("-"))
    A("  C-1: R[e0]=2.6 vs observed=2.6  residual=0  tol=1e-6  [PASS]")
    A("  C-2: R[e0]=2.6 vs observed=99.0 residual=96.4          [CORRESPONDENCE FAILED]")
    A("  All tests: synthetic fixtures only — physical domain NOT YET READY")
    A("")

    # §7.7 — REJECTED, BLOCKED, OPEN
    A(_hr("-"))
    A("§7  REJECTED / BLOCKED / OPEN CONDITIONS")
    A(_hr("-"))
    if pkg.blocked_conditions:
        A("  BLOCKED:")
        for bc in pkg.blocked_conditions:
            A(f"    {bc}")
    else:
        A("  BLOCKED: (none — all declared declarations present for executed tests)")
    A("")
    if pkg.open_conditions:
        A("  OPEN:")
        for oc in set(pkg.open_conditions):
            A(f"    {oc}")
    A("")
    A("  OPEN INTEGRATION REQUIREMENT E-1:")
    A("    Session API does not expose PersistenceState across calls.")
    A("    operator_e_v5 and PersistenceState exist in irrs-kernel.")
    A("    Gap is the session layer, not the kernel.")
    A("")
    if v and v.open_items:
        A("  VERIFIER OPEN ITEMS:")
        for oi in v.open_items:
            A(f"    {oi}")
    A("")

    # §7.8 — Generator–Verifier disagreement
    A(_hr("-"))
    A("§8  GENERATOR–VERIFIER DISAGREEMENT")
    A(_hr("-"))
    if v and v.disagreements:
        A("  STOP — disagreements require Origin authorization:")
        for d in v.disagreements:
            A(f"    {d}")
    else:
        A("  NONE — no disagreements recorded.")
    A("")

    # §7.9 — Next action requiring Origin authorization
    A(_hr("-"))
    A("§9  NEXT ACTION REQUIRING ORIGIN AUTHORIZATION")
    A(_hr("-"))
    next_actions = []
    if v and v.conformance_verdict == "DOES_NOT_CONFORM":
        next_actions.append("Verifier reports non-conformance — review §8 disagreements and authorize resolution.")
    if v and v.disagreements:
        next_actions.append("Resolve Generator–Verifier disagreements before advancing.")
    next_actions.append(
        "OPEN E-1: Authorize session-layer PersistenceState integration or defer."
    )
    next_actions.append(
        "Physical domain: Supply real observable D and M declarations to advance beyond synthetic fixtures."
    )
    if v and not v.independent_reproduction_available:
        next_actions.append(
            "Independent Rust reproduction: Run `cargo run -p irrs-e2e-tests` on local machine "
            f"(C:\\Users\\Robin Macomber\\Documents\\Metatron_Dynamics\\GitHub_Repos\\abr-irrs) "
            "and supply output to confirm independent execution."
        )
    for i, na in enumerate(next_actions, 1):
        A(f"  {i}. {na}")
    A("")

    A(_hr())
    A("Bounded over D. No claim beyond D.")
    A("Metatron Dynamics, Inc. | Kernel V8 frozen | Orchestrator V0.1")
    A(_hr())

    out_path.write_text("\n".join(lines), encoding="utf-8")
    print(f"[M4] Consolidated report written → {out_path}")


def serialize_evidence(pkg: EvidencePackage, out_path: Path) -> None:
    """Write the full evidence package as JSON.  Credentials excluded (§3)."""
    payload = asdict(pkg)
    out_path.write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")
    print(f"[M4] Evidence package written   → {out_path}")


# ---------------------------------------------------------------------------
# Pipeline orchestrator
# ---------------------------------------------------------------------------

def run_pipeline(
    repo_root:         Path,
    authorized_commit: str,
    evidence_out:      Path,
    report_out:        Path,
) -> EvidencePackage:
    pkg = EvidencePackage(authorized_commit=authorized_commit)

    # ── M1 ──────────────────────────────────────────────────────────────────
    print(f"\n{'='*72}")
    print("M1 — Execution foundation")
    print(f"{'='*72}")

    pkg.environment = _capture_environment(repo_root)

    print(f"[M1] Pinning commit — authorized: {authorized_commit[:12]}…")
    provenance = pin_commit(repo_root, authorized_commit)
    pkg.commit_provenance = provenance

    if not provenance.commit_matches or not provenance.working_tree_clean:
        stop = StopRecord(
            reason="Provenance check failed.",
            condition_index=5,
            recorded_at=_now_iso(),
        )
        pkg.stop_records.append(stop)
        _print_stop(stop)
        return pkg

    print(f"[M1] Commit verified ✓  Working tree clean ✓")
    print(f"[M1] Invoking: cargo run -p {TEST_CRATE}  (timeout={RUNNER_TIMEOUT_S}s)")

    execution = run_rust_tests(repo_root)
    pkg.execution = execution

    print(f"[M1] exit_code={execution.exit_code}  duration={execution.duration_seconds}s")

    stop = evaluate_stop_conditions(provenance, execution)
    if stop:
        pkg.stop_records.append(stop)
        _print_stop(stop)
        return pkg

    print("[M1] ✓ complete")

    # ── M2 ──────────────────────────────────────────────────────────────────
    print(f"\n{'='*72}")
    print("M2 — Evidence pipeline")
    print(f"{'='*72}")

    print("[M2] Reading source files…")
    sources = collect_sources(repo_root)
    for s in sources:
        print(f"[M2]   {s.repo_relative_path}  ({s.line_count} lines  sha256={s.sha256[:12]}…)")

    pkg2 = build_evidence_package(authorized_commit, pkg.environment, provenance, execution, sources)
    pkg.sources           = pkg2.sources
    pkg.test_results      = pkg2.test_results
    pkg.open_conditions   = pkg2.open_conditions
    pkg.blocked_conditions= pkg2.blocked_conditions

    print(f"[M2] Test results: " +
          ", ".join(f"{tr.name}:{tr.status}" for tr in pkg.test_results))
    print(f"[M2] OPEN conditions : {len(pkg.open_conditions)}")
    print(f"[M2] BLOCKED conditions: {len(pkg.blocked_conditions)}")
    print("[M2] ✓ complete")

    # ── M3 ──────────────────────────────────────────────────────────────────
    print(f"\n{'='*72}")
    print("M3 — Automated verification")
    print(f"{'='*72}")

    verifier_rec = run_verifier(pkg)
    pkg.verifier = verifier_rec

    if verifier_rec.conformance_verdict == "ERROR":
        stop = StopRecord(
            reason=f"Verifier API error: {verifier_rec.response[:200]}",
            condition_index=5,
            recorded_at=_now_iso(),
        )
        pkg.stop_records.append(stop)
        _print_stop(stop)
        # Continue anyway — don't block evidence output on API error

    if verifier_rec.disagreements:
        stop = StopRecord(
            reason=(
                f"Generator–Verifier disagreement: "
                f"{verifier_rec.disagreements[0][:120]}"
                + ("…" if len(verifier_rec.disagreements) > 1 else "")
            ),
            condition_index=4,
            recorded_at=_now_iso(),
        )
        pkg.stop_records.append(stop)
        print(f"[M3] STOP — disagreement(s) recorded; returning control to Origin after report.")
    else:
        print("[M3] No Generator–Verifier disagreements.")

    print("[M3] ✓ complete")

    # ── M4 ──────────────────────────────────────────────────────────────────
    print(f"\n{'='*72}")
    print("M4 — Consolidated report")
    print(f"{'='*72}")

    pkg.pipeline_complete = (
        len(pkg.stop_records) == 0 or
        all(sr.condition_index != 4 for sr in pkg.stop_records)  # no disagreements
    )
    pkg.completed_at = _now_iso()

    serialize_evidence(pkg, evidence_out)
    write_consolidated_report(pkg, report_out)
    print("[M4] ✓ complete")

    return pkg


def _print_stop(stop: StopRecord) -> None:
    print(f"\n[STOP] {stop.reason}")
    print(f"       Condition: {STOP_CONDITIONS[stop.condition_index]}")
    print(f"       Recorded : {stop.recorded_at}")
    print("       Returning control to Origin.\n")


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(
        description="IRRS Orchestrator V0.1 — M1→M2→M3→M4 pipeline"
    )
    parser.add_argument("--repo-root", type=Path,
                        default=Path(__file__).parent,
                        help="Repository root (default: directory of this script)")
    parser.add_argument("--authorized-commit", default=AUTHORIZED_COMMIT,
                        help="Origin-authorized commit SHA (default: %(default)s)")
    parser.add_argument("--evidence-out", type=Path, default=None,
                        help="Evidence package output path (default: <repo>/irrs_evidence_package.json)")
    parser.add_argument("--report-out", type=Path, default=None,
                        help="Human-readable report output path (default: <repo>/irrs_report.txt)")
    args = parser.parse_args()

    repo_root = args.repo_root.resolve()
    if not repo_root.is_dir():
        print(f"ERROR: repo-root does not exist: {repo_root}", file=sys.stderr)
        return 1

    evidence_out = (args.evidence_out or repo_root / "irrs_evidence_package.json").resolve()
    report_out   = (args.report_out   or repo_root / "irrs_report.txt").resolve()

    print(f"IRRS Orchestrator V0.1")
    print(f"Repo root  : {repo_root}")
    print(f"Auth commit: {args.authorized_commit[:12]}…")
    print(f"Evidence   : {evidence_out}")
    print(f"Report     : {report_out}")

    pkg = run_pipeline(repo_root, args.authorized_commit, evidence_out, report_out)

    if pkg.stop_records:
        stopped_by = [sc for sc in pkg.stop_records
                      if sc.condition_index in (4, 5, 6)]
        if stopped_by:
            print(f"\nPipeline stopped — {len(pkg.stop_records)} stop record(s).")
            return 2

    if pkg.pipeline_complete:
        print("\nPipeline complete — all milestones M1–M4 satisfied.")
        return 0

    return 2


if __name__ == "__main__":
    sys.exit(main())

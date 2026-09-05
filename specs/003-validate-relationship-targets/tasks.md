---
id: TASK-003
title: "Relationship target validation implementation tasks"
type: implementation-tasks
status: implemented
created: 2026-09-05
updated: 2026-09-05
owner: TBD
parent: US-003
depends_on: [TASK-002]
requires: [REQ-003, DES-003, ADR-005, ADR-006]
blockers: []
related:
  - EPIC-002
  - US-003
  - REQ-003
  - DES-003
  - DES-002
  - ADR-002
  - ADR-003
  - ADR-004
  - ADR-005
  - ADR-006
approval:
  approved_by: Project owner
  approved_on: 2026-09-05
---

# Tasks

<!-- Tasks turn the approved design into an ordered implementation plan. They
must not silently change the requirements or design. -->

## Implementation Approach

Extend the existing layered Rust validator with one relationship-target
vertical slice. Reuse `ArtifactIdentitySource` for active and historical
candidate discovery, keep relationship rules pure in the domain crate, and
reuse `ValidationReport` for structural and relationship findings. Preserve the
active-only `ArtifactSource` behavior and the existing filesystem discovery
boundary.

Follow the constitutional `RED -> GREEN -> REFACTOR -> TRACE` loop. Write and
observe failing tests before each corresponding production implementation. Use
the existing workspace crates, parser dependencies, in-memory source doubles,
and quality tooling. Do not add a crate, dependency, persistence, schema, CLI,
serialization, mutation, lifecycle promotion, reciprocal-link, or graph-cycle
behavior.

## Ordered Tasks

- [x] **TASK-003-1 (RED): Specify pure relationship rules in domain tests.**
  - Outcome: Add traceable domain tests for the empty and null relationship
    boundary, malformed and unresolved values, valid target resolution, missing
    targets, wrong-kind mappings, every expected-kind rule, any-kind fields,
    historical target eligibility at the normalized-input boundary, one
    diagnostic per invalid entry, complete continuation, duplicate target IDs,
    and deterministic diagnostic ordering.
  - Dependencies: TASK-002.
  - Traceability: `REQ-003` FR-002 through FR-006 and FR-008 through FR-010;
    scenarios `Valid relationships resolve across current and historical
    artifacts`, `Empty relationship collections do not create target failures`,
    `Missing targets are reported for every invalid relationship entry`,
    `Wrong-kind targets remain diagnosable`, `Relationship failures do not
    suppress unrelated artifacts`, `Structural relationship failures are not
    duplicated as target failures`, and `Relationship validation is deterministic
    and read-only`.
  - Constraints: Keep tests pure and filesystem-free. Use existing `rstest`,
    `proptest`, and domain fixtures. Assert observable diagnostics and report
    behavior, not private implementation details. Do not add a dependency.
  - Verification: Focused domain tests fail for the expected missing
    `validate_relationships` behavior or API, not because fixtures or the test
    harness are invalid.

- [x] **TASK-003-2 (RED): Specify relationship application orchestration.**
  - Outcome: Add application tests through the public API using a hand-written
    `ArtifactIdentitySource` fake. Cover empty success, valid relationships,
    missing and wrong-kind diagnostics, one finding per invalid entry, complete
    unrelated-result retention, structural-diagnostic preservation, per-file
    source diagnostics, deterministic repeated reports, and terminal
    repository-level discovery failure.
  - Dependencies: TASK-003-1.
  - Traceability: `REQ-003` FR-004 through FR-010; scenarios `Missing targets
    are reported for every invalid relationship entry`, `Wrong-kind targets
    remain diagnosable`, `Relationship failures do not suppress unrelated
    artifacts`, `Structural relationship failures are not duplicated as target
    failures`, and `Relationship validation is deterministic and read-only`.
  - Constraints: Test through the application public API and the existing
    `ArtifactIdentitySource` contract. Do not change the active `ArtifactSource`
    tests or introduce a second source port.
  - Verification: Focused application tests fail with the expected missing
    relationship use case or public API before production implementation is
    added.

- [x] **TASK-003-3 (RED): Specify filesystem scope and integrated acceptance behavior.**
  - Outcome: Add failing adapter and integrated tests using temporary local
    repositories. Cover active, archived, and superseded targets; every
    relationship field used by the approved scenarios; template and supporting
    file exclusion; empty collections; malformed, empty, and unresolved values;
    multiple invalid entries; unrelated valid artifacts; identical repeated
    results; unchanged source bytes and lifecycle statuses; and the existing
    active-only source boundary.
  - Dependencies: TASK-003-2.
  - Traceability: Every scenario in
    `specs/003-validate-relationship-targets/scenarios.feature`; `REQ-003`
    FR-001 through FR-010; `DES-003` filesystem, compatibility, and data-flow
    contracts.
  - Constraints: Use the real public filesystem and application APIs. Do not
    test through a CLI because CLI behavior and serialization are deferred to
    EPIC-004. Do not require a new adapter component.
  - Verification: Focused adapter and acceptance tests fail for the expected
    absence of relationship evaluation while existing active-only tests retain
    their baseline behavior.

- [x] **TASK-003-4 (GREEN): Implement the pure relationship evaluator.**
  - Outcome: Add the domain relationship module and public
    `validate_relationships` function. Build a deterministic recognized-ID
    target index, scan applicable fields in fixed order, apply expected-kind and
    any-kind rules, skip structural-only values, and emit actionable
    `ARTIFACT.RELATIONSHIP.MISSING_TARGET` and
    `ARTIFACT.RELATIONSHIP.WRONG_KIND` diagnostics with one finding per invalid
    entry.
  - Dependencies: TASK-003-1.
  - Traceability: `REQ-003` FR-002 through FR-006 and FR-008 through FR-010;
    `DES-003` proposed design, interfaces, and state flow.
  - Constraints: Keep domain code synchronous, pure, deterministic, and free of
    filesystem, parser, serialization, async, and framework dependencies. Reuse
    existing domain identifiers, metadata values, diagnostics, and ordering
    semantics. Do not reparse malformed snapshots or implement reciprocal-link
    or cycle rules.
  - Verification: TASK-003-1 domain tests pass without weakened assertions;
    public function documentation and `R-SDD-02` traceability are present;
    domain layering and lint rules remain satisfied.

- [x] **TASK-003-5 (GREEN): Implement relationship application orchestration.**
  - Outcome: Add the documented `RelationshipValidator<S>` use case, export it
    from the application crate, consume `ArtifactIdentitySource`, create base
    structural results, merge relationship diagnostics by source path, retain
    candidate-level failures, and return the existing ordered
    `ValidationReport`.
  - Dependencies: TASK-003-2 and TASK-003-4.
  - Traceability: `REQ-003` FR-001, FR-004 through FR-010; `DES-003`
    interfaces and data/state flow; `ADR-005` and `ADR-006` layer and source
    boundaries.
  - Constraints: Use constructor injection and domain/application types only in
    the use case and port. Propagate typed repository discovery errors. Do not
    change `Validator`, `ArtifactSource`, or active-only behavior. Do not add a
    new trait, crate, dependency, or serialization surface.
  - Verification: TASK-003-2 application tests pass; source failures remain
    diagnostics or typed discovery errors according to the established contract;
    public items have documentation and error contracts.

- [x] **TASK-003-6 (REFACTOR AND TRACE): Complete contract and acceptance coverage.**
  - Outcome: Run all approved US-003 scenarios through the real filesystem source
    and application use case; add or refine property and contract coverage for
    deterministic ordering, complete diagnostic retention, and read-only
    behavior; curate exports, docs, helpers, and fixtures; and remove any
    duplication without changing approved behavior.
  - Dependencies: TASK-003-3 and TASK-003-5.
  - Traceability: Every `REQ-003` functional and quality requirement, every
    scenario in `scenarios.feature`, `R-SDD-02`, `R-TST-01`, `R-TST-03`,
    `R-TST-05`, `R-TST-14`, `R-TST-16`, `R-TST-17`, `R-TST-18`, `R-DIR-01`,
    `R-SEP-02`, `R-TRT-01`, `R-TRT-20`, and `R-DOC-01`.
  - Constraints: Assert observable contracts rather than implementation calls.
    Preserve existing identity and active-validation behavior. Do not add CLI,
    serialization, persistence, lifecycle, schema, reciprocal, or cycle checks.
  - Verification: All focused domain, application, adapter, contract, and
    acceptance tests pass; two identical validations return equal ordered
    reports; before/after repository snapshots prove no artifact content,
    lifecycle status, or persisted result changed; dependency direction remains
    `application -> domain` and adapter -> application/domain.

- [x] **TASK-003-7 (VERIFY): Execute and record complete quality-gate evidence.**
  - Outcome: Execute the constitutional Rust quality suite and record observed
    output, test counts, coverage, dependency audit, and environment limitations
    in this task document for `verify-feature`.
  - Dependencies: TASK-003-6.
  - Traceability: `REQ-003` quality requirements; `CONSTITUTION.md`
    R-AGT-07, R-SDD-02, R-TST-01, R-TST-03, R-TST-14, R-TST-16, R-TST-17,
    R-TST-18, R-TST-19, R-DIR-01, R-SEP-02, R-TRT-01, R-TRT-20, R-DOC-01,
    and R-TOOL-04.
  - Verification: Run `cargo xtask ci`, reproducing `cargo fmt --all --
    --check`, `cargo clippy --workspace --all-targets --all-features -- -D
    warnings`, `cargo test --workspace --all-features`, rustdoc with
    `RUSTDOCFLAGS="-D warnings"`, `cargo deny check`, workspace coverage at
    least 85%, domain coverage at least 95%, and release tests. Also run
    `cargo machete` and `cargo check --manifest-path fuzz/Cargo.toml`; attempt
    the available fuzz-target smoke command and record if `cargo-fuzz` is not
    installed.

## Test And Verification Plan

- [x] Domain unit tests: Field applicability, expected-kind policy, any-kind
  references, recognized active and historical target IDs, missing and
  wrong-kind findings, empty/null/malformed/unresolved boundaries, one finding
  per entry, complete continuation, duplicate-ID interaction, and stable
  ordering.
- [x] Domain property tests: Equal snapshot inputs produce equal ordered
  diagnostics; adding unrelated valid snapshots retains existing findings; the
  pure evaluator performs no I/O or mutation.
- [x] Application integration tests: In-memory historical source, empty and
  valid reports, missing and wrong-kind aggregation, structural findings,
  unrelated results, candidate failures, typed discovery failure, and repeated
  identical reports.
- [x] Filesystem integration tests: Active/archive/superseded discovery through
  the existing source, template/supporting exclusion, all relationship fields,
  malformed and unresolved values, read-only repository snapshots, and stable
  repeated results.
- [x] Scenario-equivalent acceptance tests: All nine approved scenario headings
  in `scenarios.feature`, including all four wrong-kind examples, are covered
  by traceable domain, application, and real-filesystem tests.
- [x] Fuzz checks: The existing parser fuzz target compiles and
  `cargo fuzz --version` is available. A one-run fuzz execution was attempted
  but could not build because the pinned stable toolchain lacks the nightly-only
  sanitizer option; no fuzz execution result is claimed.
- [x] Rust quality gates: `cargo xtask ci` passes with workspace line coverage at
  least 85% and domain line coverage at least 95%; `cargo machete` reports no
  unused dependencies; release tests pass.
- [x] CLI and serialization checks: Not applicable to this slice; deferred to
  EPIC-004.
- [x] .NET and frontend gates: Not applicable; this feature changes only the
  Rust workspace and its specifications.

## Verification Evidence

- Date: 2026-09-05.
- Repository HEAD at verification: `ae3e0812b89e9afdbed1c4258356feb3f6cf4085`;
  implementation changes remain uncommitted.
- RED phase: focused domain, application, and filesystem relationship tests
  failed before the new `validate_relationships` and `RelationshipValidator`
  APIs existed, with unresolved-import compiler errors.
- GREEN and refactor phase: 10 domain relationship tests, 6 application
  relationship tests, and 6 filesystem relationship tests passed; clippy and
  formatting checks passed after the final changes.
- `cargo xtask ci`: passed, including formatting, clippy, workspace tests,
  rustdoc with warnings denied, dependency audit, coverage, and release tests.
- `cargo test --workspace --all-features`: passed; all workspace unit,
  integration, relationship, identity, filesystem, and documentation tests
  passed.
- `cargo llvm-cov --workspace --fail-under-lines 85`: passed at 92.89% line
  coverage.
- `cargo llvm-cov --package sdd-domain --fail-under-lines 95`: passed at 97.07%
  line coverage.
- `cargo machete`: passed with no unused dependencies.
- `cargo check --manifest-path fuzz/Cargo.toml`: passed.
- `cargo fuzz --version`: passed with `cargo-fuzz 0.13.2`.
- `cargo fuzz run parse_artifact -- -runs=1`: attempted but blocked because
  rustup has only stable `1.95.0` installed and cargo-fuzz requires nightly
  sanitizer flags for this run; no fuzz execution was claimed.

## Rollout And Recovery

### Rollout

- Add the pure relationship evaluator and application use case as a library-only
  extension to the existing workspace.
- Reuse the existing active-plus-historical identity source; do not change
  active-only validation callers or filesystem source behavior.
- No database, file-format, lifecycle, schema, persisted-result, deployment,
  network, retry, or background-job migration is needed.
- The validator reads a repository snapshot and returns a complete report or a
  typed repository-discovery error.

### Recovery

- Per-file read, parse, recognition, or relationship failures require no
  rollback: preserve diagnostics and continue processing all other candidates.
- A repository-level discovery failure is terminal for that invocation; correct
  repository access and rerun. Do not claim partial success.
- Validation performs no mutation, so an interrupted run needs no data cleanup.
  Rerunning against the same repository state is safe and should produce the
  same ordered result.
- If implementation verification fails, revert the relationship implementation
  and its same-commit specification/task changes together. Do not modify
  artifact contents or lifecycle statuses as recovery work.

## Definition Of Done

- [x] All ordered tasks are complete with observed RED and GREEN evidence.
- [x] Every `REQ-003` functional and quality requirement is covered by a
  traceable passing test.
- [x] All approved scenario behaviors are covered by traceable tests through the
  real adapter and application relationship path where filesystem behavior is
  applicable.
- [x] Active, archived, superseded, empty, missing, wrong-kind, excluded,
  malformed, unresolved, unrelated, deterministic, and read-only paths are
  verified.
- [x] Every invalid concrete relationship entry receives one actionable
  diagnostic on its referencing artifact, with expected and actual kind context
  for wrong-kind findings.
- [x] Structural diagnostics remain attributable to EPIC-001 and are not
  duplicated by relationship target validation.
- [x] The active-only `ArtifactSource` behavior remains unchanged, and the
  existing historical source boundary is reused without a new port.
- [x] Domain purity, dependency direction, public documentation, typed errors,
  and no-unsafe/no-new-dependency constraints pass review.
- [x] `cargo xtask ci`, coverage thresholds, dependency audit, unused-
  dependency analysis, release tests, and fuzz-target compilation pass with
  observed output recorded for `verify-feature`; the nightly-only fuzz execution
  limitation is explicitly recorded.
- [x] No CLI, serialization, persistence, lifecycle, schema, reciprocal-link,
  cycle, or release behavior is implemented by this feature.
- [x] Relevant specifications remain current in the implementation change set.

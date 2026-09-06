---
id: TASK-005
title: "Relationship cycle detection implementation tasks"
type: implementation-tasks
status: implemented
created: 2026-09-07
updated: 2026-09-07
owner: TBD
parent: US-005
depends_on: [TASK-004]
requires: [REQ-005, DES-005, ADR-005, ADR-006]
blockers: []
related:
  - EPIC-002
  - US-005
  - REQ-005
  - DES-005
  - REQ-004
  - DES-004
  - REQ-003
  - DES-003
  - REQ-002
  - DES-002
  - ADR-002
  - ADR-003
  - ADR-004
  - ADR-005
  - ADR-006
approval:
  approved_by: Project owner
  approved_on: 2026-09-07
---

# Tasks

<!-- Tasks turn the approved design into an ordered implementation plan. Each
task should have a concrete completion check and should not silently change the
requirements or design. -->

## Implementation Approach

Implement one cycle-detection vertical slice in the existing layered Rust
workspace. Add a pure domain evaluator over normalized snapshots, then add an
opt-in application validator that composes structural, target, reciprocal, and
cycle findings over one active-plus-historical candidate set. Reuse the existing
filesystem source, parser boundary, in-memory source double, diagnostic model,
and `ValidationReport` ordering.

Follow the constitutional `RED -> GREEN -> REFACTOR -> TRACE` loop. All RED
tests must fail because the specified cycle API or behavior is absent, not
because the test harness is invalid. GREEN implementation must preserve the
existing active-only `Validator`, target-only `RelationshipValidator`, and
reciprocal-only `ReciprocalRelationshipValidator` behavior.

Do not add a crate, workspace member, dependency, port, architectural layer,
persistence, schema, CLI, serialization, lifecycle mutation, network access, or
automatic repair behavior.

## Ordered Tasks

- [x] **TASK-005-1 (RED): Specify pure cycle rules in domain tests.**
  - Outcome: Add traceable domain tests for empty and clean graphs, eligible
    parent cycles, `depends_on`/`requires`/`blockers` cycles, supersession
    normalization, target-valid self-loops, wrong-kind self-loops, mixed-family
    isolation, ineligible values, target-valid non-reciprocal entries, historical
    states, excluded nodes, source-owned diagnostics, rotation deduplication,
    overlapping cycles, complete continuation, cycle result status, and stable
    ordering.
  - Dependencies: `TASK-004`.
  - Traceability: `REQ-005` FR-001 through FR-014; scenarios `A valid repository
    with no cycles succeeds`, `An eligible parent cycle is reported within the
    parent graph`, `Each dependency relationship field participates in cycle
    detection`, `Supersession fields are normalized to one logical direction`,
    `A target-valid self-loop is reported`, `A wrong-kind self-loop is not
    duplicated as a cycle`, `Different relationship families do not form a cycle
    together`, `A directed cycle is reported once regardless of traversal
    rotation`, `Distinct overlapping cycles receive distinct findings`,
    `Ineligible relationship values retain existing diagnostics without cycle
    findings`, `A target-valid non-reciprocal relationship remains eligible`,
    `Historical recognized artifacts participate while excluded files do not`,
    `Cycle findings do not suppress valid unrelated artifacts`, and `Cycle
    validation is deterministic and read-only`.
  - Constraints: Keep tests filesystem-free and use normalized domain snapshots
    only. Use the existing `rstest` dependency for the dependency-field and
    invalid-value matrices, and `proptest` for rotation canonicalization and
    ordering properties. Assert diagnostics, cycle identities, and result values
    rather than private graph representation. Include test comments or names
    citing `REQ-005` and the covered `FR-*` identifiers.
  - Verification: Focused domain tests fail with the expected unresolved
    `validate_relationship_cycles` API or missing cycle behavior, while existing
    domain tests remain the baseline.

- [x] **TASK-005-2 (RED): Specify cycle-aware application orchestration.**
  - Outcome: Add application integration tests through the public API for the
    new `CycleValidator<S>`. Cover empty and clean reports, parent/dependency/
    supersession findings, merged structural/target/reciprocal/cycle findings,
    source-owned diagnostics, valid unrelated result retention, historical
    candidates, typed repository discovery failure, one source discovery, and
    repeated identical reports.
  - Dependencies: `TASK-005-1`.
  - Traceability: `REQ-005` FR-001, FR-006, and FR-010 through FR-014; scenarios
    `A valid repository with no cycles succeeds`, `A target-valid non-reciprocal
    relationship remains eligible`, `Historical recognized artifacts
    participate while excluded files do not`, `Cycle findings do not suppress
    valid unrelated artifacts`, and `Cycle validation is deterministic and
    read-only`; `DES-005` application interfaces and data flow.
  - Constraints: Use a hand-written `ArtifactIdentitySource` fake and only the
    application public API. Verify that one discovery result feeds structural,
    target, reciprocal, and cycle evaluation. Preserve typed discovery errors and
    do not add a new process-boundary port.
  - Verification: Focused application tests fail with the expected missing
    `CycleValidator` or public API before production implementation is added;
    existing US-003 and US-004 application tests remain unchanged.

- [x] **TASK-005-3 (RED): Specify filesystem and scenario-equivalent behavior.**
  - Outcome: Add failing adapter integration tests using temporary repositories
    and the real `FilesystemArtifactSource`. Cover all 14 approved Gherkin
    scenario headings, all three dependency-field examples, all four ineligible
    value examples, active/archive/superseded participation, template and
    supporting-file exclusion, source and existing diagnostic preservation,
    complete result retention, stable ordering, repeated reports, and unchanged
    source bytes and lifecycle metadata.
  - Dependencies: `TASK-005-2`.
  - Traceability: Every scenario in `specs/005-detect-relationship-cycles/
    scenarios.feature`; `REQ-005` FR-001 through FR-014; `DES-005` filesystem,
    compatibility, data-flow, and recovery contracts; `ADR-005` and `ADR-006`
    discovery and parser boundaries.
  - Constraints: Use temporary local files and the public application and
    adapter APIs. Do not test through a CLI because CLI behavior and
    serialization are deferred to EPIC-004. Do not change canonical discovery
    policy. Each test keeps Arrange / Act / Assert sections visible and cites the
    corresponding specification IDs.
  - Verification: Focused adapter tests fail for the expected absence of cycle
    evaluation, while existing active-only, target-only, and reciprocal behavior
    remains the compatibility baseline.

- [x] **TASK-005-4 (GREEN): Implement the pure domain cycle evaluator.**
  - Outcome: Add and deliberately export
    `domain::validate_relationship_cycles`. Reuse or factor the existing
    relationship eligibility boundary without changing target-validation
    behavior. Build ordered eligible edges for independent parent, dependency,
    and supersession families; normalize supersession direction; enumerate all
    distinct simple directed cycles; canonicalize traversal rotations; preserve
    distinct overlapping cycles; handle eligible self-loops; and emit one
    `ARTIFACT.RELATIONSHIP.CYCLE` diagnostic per participating source entry per
    cycle identity.
  - Dependencies: `TASK-005-1` and `TASK-005-3`.
  - Traceability: `REQ-005` FR-001 through FR-011; `DES-005` proposed design,
    domain components, interfaces, cycle identity, data flow, performance, and
    risk contracts.
  - Constraints: Keep the domain synchronous, pure, deterministic, and free of
    filesystem, parser, serialization, async, and framework dependencies. Use
    existing standard-library ordered collections and domain identifiers. Do not
    add a graph crate or change the `Diagnostic` process boundary. Keep malformed,
    empty, unresolved, missing-target, and wrong-kind values outside the graph;
    allow target-valid non-reciprocal entries; do not emit findings for mixed
    relationship families. Add no unsafe code and preserve public documentation.
  - Verification: TASK-005-1 domain tests pass without weakened assertions;
    cycle identities are rotation-stable, overlapping cycles remain distinct,
    diagnostic counts and source paths are correct, and domain lint,
    dependency-direction, and coverage checks remain satisfied.

- [x] **TASK-005-5 (GREEN): Implement cycle-aware application orchestration.**
  - Outcome: Add and export the documented
    `application::CycleValidator<S>`. Discover candidates once through
    `ArtifactIdentitySource`, preserve per-file source failures, create base
    structural results, run target, reciprocal, and cycle evaluators over the
    same snapshots, merge all findings by source path, and return the established
    ordered `ValidationReport`.
  - Dependencies: `TASK-005-2` and `TASK-005-4`.
  - Traceability: `REQ-005` FR-001, FR-006, and FR-010 through FR-014;
    `DES-005` interfaces, responsibilities, data/state flow, compatibility,
    security, and operations; `ADR-005` and `ADR-006` layering and source
    boundaries.
  - Constraints: Use constructor injection and existing domain/application
    types. Propagate typed repository discovery errors. Keep existing
    `Validator`, `ArtifactSource`, `RelationshipValidator`,
    `ReciprocalRelationshipValidator`, and filesystem discovery behavior
    compatible. Do not add a trait, crate, dependency, serialization surface,
    persistence, or CLI. Document every new public item and its error contract.
  - Verification: TASK-005-2 application tests pass; structural, target,
    reciprocal, cycle, and source diagnostics remain attributable to their
    original paths; valid unrelated results remain represented; one discovery
    feeds all evaluation; public API and lint checks pass.

- [x] **TASK-005-6 (REFACTOR AND TRACE): Complete contract and acceptance coverage.**
  - Outcome: Run every approved US-005 scenario through `CycleValidator` and the
    real filesystem source; refine tests for all dependency fields, supersession
    normalization, parent-family defense, target-valid and wrong-kind self-loops,
    mixed-family isolation, invalid-value ownership, non-reciprocal eligibility,
    active and historical scope, excluded files, source ownership, complete
    continuation, canonical cycle identity, overlapping cycles, deterministic
    ordering, and read-only behavior. Curate exports and helpers and remove
    duplication without changing approved behavior.
  - Dependencies: `TASK-005-3` and `TASK-005-5`.
  - Traceability: Every `REQ-005` functional and quality requirement and every
    scenario in `scenarios.feature`; `R-SDD-02`, `R-SDD-05`, `R-AGT-03`,
    `R-TST-01`, `R-TST-03`, `R-TST-04`, `R-TST-05`, `R-TST-07`, `R-TST-10`,
    `R-TST-16`, `R-TST-17`, `R-TST-18`, `R-DIR-01`, `R-DIR-02`, `R-SEP-01`,
    `R-SEP-02`, `R-SEP-03`, `R-SEP-07`, `R-SEP-08`, `R-TRT-01`, `R-TRT-04`,
    `R-TRT-16`, `R-TRT-20`, `R-DOC-01`, and `R-DOC-03`.
  - Constraints: Assert observable contracts rather than implementation calls.
    Preserve existing identity, active validation, target validation, and
    reciprocity behavior. Do not add CLI, serialization, persistence,
    lifecycle, schema, release, or automatic repair behavior. If implementation
    reveals a requirements or design mismatch, stop and revise those artifacts
    before changing code.
  - Verification: All focused domain, application, adapter, property, and
    scenario-equivalent tests pass; repeated validation returns equal ordered
    reports; before/after repository snapshots prove no source or lifecycle
    mutation; every new public item is documented; dependency direction remains
    `application -> domain` and adapter -> application/domain.

- [x] **TASK-005-7 (VERIFY): Execute and record complete quality-gate evidence.**
  - Outcome: Execute the constitutional Rust quality suite and record observed
    command output, test counts, coverage, dependency audit, release tests,
    property-test results, and environment limitations in this task document for
    `verify-feature`.
  - Dependencies: `TASK-005-6`.
  - Traceability: `REQ-005` quality requirements; `DES-005` verification
    approach; `CONSTITUTION.md` R-AGT-07, R-SDD-02, R-SDD-05, R-TST-01,
    R-TST-03, R-TST-05, R-TST-14, R-TST-16, R-TST-17, R-TST-18, R-DIR-01,
    R-DIR-02, R-SEP-02, R-TRT-01, R-TRT-20, R-DOC-01, R-DOC-03, and
    R-TOOL-04.
  - Verification: Run `cargo xtask ci`, reproducing formatting, clippy,
    workspace tests, rustdoc with warnings denied, dependency audit, workspace
    coverage at least 85%, domain coverage at least 95%, and release tests.
    Also run `cargo machete`, `cargo check --manifest-path fuzz/Cargo.toml`,
    `cargo fuzz --version`, and attempt the available fuzz-target smoke command.
    Record any nightly-only fuzz limitation without claiming an unexecuted fuzz
    result. Confirm no new dependencies, warnings, lint suppressions, or
    constitutional deviations were introduced.

## Implementation Evidence

- Verified on 2026-09-07 against working-tree base commit
  `6a247ce37d973894fc1eea98b900290e1c221d02`.
- RED observed before implementation: `cargo test -p sdd-domain --test
  cycles` failed because `domain::validate_relationship_cycles` was absent;
  `cargo test -p sdd-application --test cycles` failed because
  `application::CycleValidator` was absent.
- GREEN focused tests passed: 12 domain cycle tests, 5 application cycle tests,
  and 2 filesystem cycle tests.
- `cargo xtask ci` passed: formatting, clippy, 124 workspace tests, rustdoc with
  warnings denied, `cargo deny check`, workspace coverage `93.61%`, domain
  coverage `96.04%`, and release tests.
- Additional checks passed: `cargo machete`, `cargo check --manifest-path
  fuzz/Cargo.toml`, and `cargo fuzz --version` (`cargo-fuzz 0.13.2`).
- Fuzz smoke command `cargo fuzz run parse_artifact -- -runs=1` was attempted
  and could not run because the pinned stable `1.95.0` toolchain rejects the
  required nightly `-Zsanitizer=address` option.
- No new crate, workspace member, dependency, port, architectural layer,
  unsafe code, lint suppression, serialization, persistence, or CLI behavior
  was introduced.
- The approved parent-cycle scenario is covered at the parent-edge and family
  isolation boundary; the current artifact-kind rules make a concrete eligible
  parent cycle unreachable, as documented in `design.md`.

## Test And Verification Plan

- [x] Domain tests: Pure normalized-snapshot coverage for all graph families,
  dependency fields, supersession direction, target eligibility, self-loops,
  invalid values, non-reciprocal entries, historical states, excluded nodes,
  canonical cycle identity, overlapping cycles, source ownership, complete
  continuation, result status, and deterministic ordering.
- [x] Domain property tests: Equivalent traversal rotations produce one cycle
  identity; distinct directed edge sets remain distinct; equivalent snapshot
  orderings produce equal ordered diagnostics; and the pure evaluator performs no
  I/O or mutation.
- [x] Application integration tests: Empty and clean reports, cycle failures,
  merged structural/target/reciprocal/cycle diagnostics, source failures,
  unrelated-result retention, one source discovery, typed discovery failure, and
  repeated identical reports.
- [x] Filesystem integration tests: Active/archive/superseded scope, template and
  supporting-file exclusion, all relationship families, invalid-value ownership,
  overlapping findings, deterministic ordering, source-byte snapshots, and
  lifecycle-state snapshots.
- [x] Scenario-equivalent acceptance tests: All 14 approved scenario headings in
  `scenarios.feature`, including the three dependency-field examples and four
  ineligible-value examples, are covered by traceable tests through the
  application and real filesystem source.
- [x] Compatibility checks: Existing active-only, relationship-target-only, and
  reciprocal-only tests continue to pass without changed expectations.
- [x] Rust quality gates: Run `cargo xtask ci`, which includes format, clippy,
  workspace tests, documentation, dependency audit, workspace coverage at least
  85%, domain coverage at least 95%, and release tests. Record observed output
  during verification; do not claim results during planning.
- [x] Additional checks: Run `cargo machete`, `cargo check --manifest-path
  fuzz/Cargo.toml`, `cargo fuzz --version`, and the available parser fuzz-target
  smoke command. Record stable-toolchain limitations explicitly.
- [x] CLI and serialization checks: Not applicable to this slice; deferred to
  EPIC-004.
- [x] .NET and frontend gates: Not applicable; this feature changes the Rust
  workspace and specifications only.

## Rollout And Recovery

### Rollout

- Add the pure cycle evaluator and opt-in application validator as a library-only
  extension to the existing workspace.
- Reuse the existing active-plus-historical `ArtifactIdentitySource`; do not
  change active-only callers, parser dependencies, or filesystem discovery
  policy.
- No database, file-format, lifecycle, schema, persisted-result, deployment,
  network, retry, or background-job migration is needed.
- The cycle-aware validator reads one candidate snapshot and returns a complete
  report or a typed repository-discovery error. CLI exposure remains deferred.

### Recovery

- Per-file read, parse, recognition, structural, target, reciprocal, or cycle
  failures require no rollback: preserve diagnostics and continue processing all
  other candidates.
- A repository-level discovery failure is terminal for that invocation; correct
  repository access and rerun without claiming partial success.
- Validation performs no mutation, so an interrupted run needs no data cleanup.
  Rerunning against the same repository state is safe and must produce the same
  ordered report.
- If implementation exposes a behavior mismatch, stop and revise the approved
  requirements or design before continuing. Do not make code the source of
  truth.
- If verification fails, retain the observed failure, correct the implementation
  or specification as appropriate, and rerun the affected RED/GREEN and quality
  gates. Do not weaken assertions or bypass a gate.

## Definition Of Done

- [x] All ordered tasks are complete with observed RED and GREEN evidence.
- [x] Every `REQ-005` functional and quality requirement is covered by a
  traceable passing test.
- [x] All approved scenario behaviors are covered by traceable domain,
  application, and filesystem tests where applicable.
- [x] Parent, dependency, and supersession cycles are evaluated in independent
  directed graphs.
- [x] `depends_on`, `requires`, and `blockers` all participate in dependency
  cycle detection, and supersession fields are normalized correctly.
- [x] Target-valid self-loops are reported; malformed, empty, unresolved,
  missing-target, and wrong-kind values do not receive cycle findings.
- [x] Existing non-reciprocal findings do not suppress eligible cycle detection,
  and existing structural and target diagnostics are not duplicated incorrectly.
- [x] Rotation-equivalent cycles are reported once, distinct overlapping cycles
  remain distinct, and every participating source entry receives the required
  per-cycle diagnostic.
- [x] Active, archived, and superseded recognized artifacts participate while
  templates and supporting files remain excluded.
- [x] Cycle findings do not suppress valid unrelated artifacts or relationships,
  and complete validation continues after failures.
- [x] Diagnostics are stable, source-owned, actionable, and use
  `ARTIFACT.RELATIONSHIP.CYCLE` with the required cycle context.
- [x] Repeated validation produces identical ordered reports without repository or
  lifecycle mutation and without network access.
- [x] Existing identity, active-only, target-only, and reciprocal-only behavior
  remains compatible.
- [x] No new port, crate, dependency, architectural layer, unsafe code, CLI,
  serialization, persistence, lifecycle, schema, release, or repair behavior was
  introduced.
- [x] Domain purity, dependency direction, constructor injection, typed errors,
  public documentation, and lint constraints pass review.
- [x] `cargo xtask ci`, coverage thresholds, dependency audit, unused-dependency
  analysis, release tests, and fuzz-target checks have observed evidence recorded
  for `verify-feature`; environment limitations are explicitly recorded.
- [x] Relevant specifications remain current in the implementation change set.

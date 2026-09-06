---
id: TASK-004
title: "Reciprocal relationship validation implementation tasks"
type: implementation-tasks
status: implemented
created: 2026-09-06
updated: 2026-09-06
owner: TBD
parent: US-004
depends_on: [TASK-003]
requires: [REQ-004, DES-004, ADR-005, ADR-006]
blockers: []
related:
  - EPIC-002
  - US-004
  - REQ-004
  - DES-004
  - DES-003
  - DES-002
  - ADR-002
  - ADR-003
  - ADR-004
  - ADR-005
  - ADR-006
approval:
  approved_by: Project owner
  approved_on: 2026-09-06
---

# Tasks

<!-- Tasks turn the approved design into an ordered implementation plan. Each
task should have a concrete completion check and should not silently change the
requirements or design. -->

## Implementation Approach

Extend the existing layered Rust validator with one reciprocal-consistency
vertical slice. Add pure domain membership evaluation and a separate
application entry point that composes target and reciprocal findings over one
active-plus-historical candidate set. Reuse the existing filesystem discovery,
parser boundary, in-memory source double, diagnostic types, and
`ValidationReport` ordering.

Follow the constitutional `RED -> GREEN -> REFACTOR -> TRACE` loop. The RED
tasks must fail because the specified reciprocal API and behavior are absent,
not because the test harness is invalid. The GREEN tasks must preserve the
US-003 target-only `validate_relationships` and `RelationshipValidator`
behavior. Do not add a crate, workspace member, dependency, port, persistence,
schema, CLI, serialization, lifecycle, or cycle-detection behavior.

## Ordered Tasks

- [x] **TASK-004-1 (RED): Specify pure reciprocal rules in domain tests.**
  - Outcome: Add traceable domain tests for reciprocal `related` membership,
    both `supersedes`/`superseded_by` mappings, active and historical targets,
    missing counterparts, reverse links to a different artifact, empty and
    malformed values, unresolved and missing targets, wrong-kind supersession
    targets, duplicate occurrences, changed list order, multiple independent
    findings, source-owned diagnostics, and deterministic diagnostic ordering.
  - Dependencies: `TASK-003`.
  - Traceability: `REQ-004` FR-001 through FR-006 and FR-008 through FR-010;
    scenarios `Reciprocal related links resolve across current and historical
    artifacts`, `A missing related counterpart is reported on its source`, `A
    different reverse related link does not satisfy the original link`,
    `Reciprocal supersession links resolve across historical artifacts`,
    `Missing supersession counterparts are reported on their source`,
    `Excluded files cannot satisfy a reciprocal counterpart`, `Empty
    relationship collections do not create reciprocity failures`, `Earlier
    structural and target failures are not duplicated`, `Relationship
    membership ignores order and duplicate occurrences`, and `Reciprocity
    validation is deterministic and read-only`.
  - Constraints: Keep tests pure and filesystem-free. Use existing domain
    identifiers, metadata values, `rstest`, and `proptest` where an algebraic
    ordering or duplicate-insensitivity property is useful. Assert observable
    diagnostics and values rather than private index representation. Do not
    weaken or remove existing US-003 assertions.
  - Verification: Focused domain tests fail with unresolved reciprocal API or
    behavior, while existing domain tests continue to compile and run up to
    the expected missing implementation.

- [x] **TASK-004-2 (RED): Specify reciprocal application orchestration.**
  - Outcome: Add application integration tests through the public API for the
    new reciprocal validator. Cover empty success, reciprocal success,
    non-reciprocal failure, both supersession directions, combined target and
    reciprocity diagnostics, preservation of structural and per-file source
    diagnostics, valid unrelated result retention, one source diagnostic per
    unmatched entry, deterministic repeated reports, and terminal repository
    discovery failure.
  - Dependencies: `TASK-004-1`.
  - Traceability: `REQ-004` FR-002 through FR-010; scenarios `Reciprocal
    related links resolve across current and historical artifacts`, `A missing
    related counterpart is reported on its source`, `Missing supersession
    counterparts are reported on their source`, `Multiple reciprocity failures
    do not suppress valid unrelated artifacts`, `Earlier structural and target
    failures are not duplicated`, and `Reciprocity validation is deterministic
    and read-only`.
  - Constraints: Use a hand-written `ArtifactIdentitySource` fake and only the
    application public API. Verify that one source discovery result feeds both
    target and reciprocity evaluation. Do not modify the active-only
    `ArtifactSource` contract or add a second process-boundary port.
  - Verification: Focused application tests fail with the expected missing
    reciprocal validator or public API before production implementation is
    added; existing US-003 application tests remain unchanged.

- [x] **TASK-004-3 (RED): Specify filesystem scope and scenario-equivalent behavior.**
  - Outcome: Add failing filesystem integration tests using temporary local
    repositories. Cover every approved scenario in
    `specs/004-validate-reciprocal-relationships/scenarios.feature`, including
    both Scenario Outline examples, active/archive/superseded counterparts,
    template and supporting-file exclusion, empty collections, malformed and
    target-invalid values, independent failures, valid unrelated artifacts,
    stable ordering, repeated identical reports, and unchanged source bytes and
    lifecycle metadata.
  - Dependencies: `TASK-004-2`.
  - Traceability: Every scenario in `scenarios.feature`; `REQ-004` FR-001
    through FR-010; `DES-004` filesystem, compatibility, and data-flow
    contracts; `ADR-005` and `ADR-006` discovery boundaries.
  - Constraints: Use the real public filesystem and application APIs. Do not
    test through a CLI because CLI behavior and serialization are deferred to
    EPIC-004. Do not add a filesystem component or change canonical discovery
    policy. Each test must keep Arrange / Act / Assert sections visible and
    cite its `REQ-004` and `FR-*` coverage.
  - Verification: Focused adapter and scenario-equivalent tests fail for the
    expected absence of reciprocal evaluation, while existing US-003
    active-only behavior remains the baseline.

- [x] **TASK-004-4 (GREEN): Implement the pure reciprocal evaluator.**
  - Outcome: Add and deliberately export
    `domain::validate_reciprocal_relationships`. Build deterministic indexes
    for recognized target IDs, recognized kinds, and concrete reverse
    memberships. Scan `related`, `supersedes`, and `superseded_by` in fixed
    order; apply the approved reverse mappings; compare membership without
    order or multiplicity; skip values owned by structural or target
    validation; and emit one actionable
    `ARTIFACT.RELATIONSHIP.NON_RECIPROCAL` diagnostic per unmatched eligible
    directed entry.
  - Dependencies: `TASK-004-1` and `TASK-004-3`.
  - Traceability: `REQ-004` FR-001 through FR-006 and FR-008 through FR-010;
    `DES-004` proposed design, interfaces, data flow, and risks.
  - Constraints: Keep domain code synchronous, pure, deterministic, and free
    of filesystem, parser, serialization, async, and framework dependencies.
    Reuse existing domain identifiers, metadata values, diagnostic types, and
    ordering semantics. Preserve `validate_relationships` target-only behavior
    and do not implement graph cycles or identity diagnostics here. Do not add
    a dependency or unsafe code.
  - Verification: TASK-004-1 domain tests pass without weakened assertions;
    public function documentation and `R-SDD-02` traceability are present;
    domain lint and dependency-direction checks remain satisfied.

- [x] **TASK-004-5 (GREEN): Implement reciprocal application orchestration.**
  - Outcome: Add and export the documented
    `application::ReciprocalRelationshipValidator<S>`. Discover candidates
    once through `ArtifactIdentitySource`, create base structural results, run
    both the existing target evaluator and the new reciprocal evaluator over
    the same snapshots, merge findings by source path, retain candidate-level
    failures, and return the established ordered `ValidationReport`.
  - Dependencies: `TASK-004-2` and `TASK-004-4`.
  - Traceability: `REQ-004` FR-001 through FR-010; `DES-004` interfaces,
    component responsibilities, data/state flow, compatibility constraints;
    `ADR-005` and `ADR-006` layering and source boundaries.
  - Constraints: Use constructor injection and existing domain/application
    types. Propagate typed repository discovery errors. Keep the existing
    `Validator`, `ArtifactSource`, `RelationshipValidator`, and active-only
    behavior compatible. Do not add a new trait, crate, dependency, or
    serialization surface. Document every new public item and its error
    contract.
  - Verification: TASK-004-2 application tests pass; structural, target,
    reciprocity, and source diagnostics remain attributable to their original
    paths; valid unrelated results remain represented; public API and lint
    checks pass.

- [x] **TASK-004-6 (REFACTOR AND TRACE): Complete contract and acceptance coverage.**
  - Outcome: Run all approved US-004 scenarios through the new application
    use case and real filesystem source; refine domain and integration tests
    for membership properties, complete diagnostic retention, deterministic
    ordering, and read-only behavior; curate exports and helpers; and remove
    duplication without changing approved behavior.
  - Dependencies: `TASK-004-3` and `TASK-004-5`.
  - Traceability: Every `REQ-004` functional and quality requirement, every
    scenario in `scenarios.feature`, `R-SDD-02`, `R-SDD-05`, `R-AGT-03`,
    `R-TST-01`, `R-TST-03`, `R-TST-04`, `R-TST-05`, `R-TST-07`, `R-TST-10`,
    `R-TST-16`, `R-TST-17`, `R-TST-18`, `R-DIR-01`, `R-DIR-02`, `R-SEP-01`,
    `R-SEP-02`, `R-SEP-03`, `R-SEP-07`, `R-SEP-08`, `R-TRT-01`, `R-TRT-04`,
    `R-TRT-16`, `R-TRT-20`, `R-DOC-01`, and `R-DOC-03`.
  - Constraints: Assert observable contracts rather than implementation
    calls. Preserve existing identity, active validation, and US-003 target
    behavior. Do not add CLI, serialization, persistence, lifecycle, schema,
    release, or cycle checks. Any behavior mismatch requires revising the
    approved specification before code changes continue.
  - Verification: All focused domain, application, adapter, and
    scenario-equivalent tests pass; two validations return equal ordered
    reports; before/after repository snapshots prove no source or lifecycle
    mutation; dependency direction remains `application -> domain` and adapter
    -> application/domain; every new public item is documented.

- [x] **TASK-004-7 (VERIFY): Execute and record complete quality-gate evidence.**
  - Outcome: Execute the constitutional Rust quality suite and record observed
    command output, test counts, coverage, dependency audit, release tests,
    and environment limitations in this task document for `verify-feature`.
  - Dependencies: `TASK-004-6`.
  - Traceability: `REQ-004` quality requirements; `CONSTITUTION.md`
    R-AGT-07, R-SDD-02, R-SDD-05, R-TST-01, R-TST-03, R-TST-05, R-TST-14,
    R-TST-16, R-TST-17, R-TST-18, R-TST-19, R-DIR-01, R-DIR-02, R-SEP-02,
    R-TRT-01, R-TRT-20, R-DOC-01, R-DOC-03, and R-TOOL-04.
  - Verification: Run `cargo xtask ci`, reproducing formatting, clippy,
    workspace tests, rustdoc with warnings denied, dependency audit,
    workspace coverage at least 85%, domain coverage at least 95%, and release
    tests. Also run `cargo machete`,
    `cargo check --manifest-path fuzz/Cargo.toml`, `cargo fuzz --version`, and
    attempt the available fuzz-target smoke command. Record any nightly-only
    fuzz limitation without claiming an unexecuted fuzz result. Confirm no new
    dependencies or warnings were introduced.

## Test And Verification Plan

- [x] Domain unit tests: Reciprocal `related` membership; both supersession
  mappings; active, archived, and superseded recognized targets; missing and
  different reverse targets; empty, malformed, unresolved, missing-target, and
  wrong-kind boundaries; one diagnostic per unmatched entry; continued
  evaluation; duplicate and order-insensitive membership; stable ordering.
- [x] Domain property tests: Reordering or duplicating counterpart values does
  not change findings; equal snapshot inputs produce equal ordered diagnostics;
  adding unrelated valid snapshots retains existing reciprocal findings; the
  pure evaluator performs no I/O or mutation.
- [x] Application integration tests: Empty and reciprocal reports,
  non-reciprocal failures, both supersession directions, combined target and
  reciprocity diagnostics, structural and source-diagnostic preservation,
  unrelated-result retention, typed discovery failure, one source discovery,
  and repeated identical reports.
- [x] Filesystem integration tests: Active/archive/superseded counterparts,
  template/supporting exclusion, all three field mappings, malformed and
  unresolved values, multiple asymmetric entries, valid unrelated artifacts,
  source-byte and lifecycle snapshots, and stable repeated reports.
- [x] Scenario-equivalent acceptance tests: All 11 approved scenario headings
  in `scenarios.feature`, including both `Scenario Outline` examples, are
  covered by traceable tests through the application and real filesystem
  paths.
- [x] Rust quality gates: `cargo xtask ci` passes with workspace line coverage
  at least 85% and domain line coverage at least 95%; `cargo machete` reports
  no unused dependencies; release tests pass; fuzz target compilation and
  available smoke checks are recorded.
- [x] CLI and serialization checks: Not applicable to this slice; deferred to
  EPIC-004.
- [x] .NET and frontend gates: Not applicable; this feature changes only the
  Rust workspace and its specifications.

## Verification Evidence

- Date: 2026-09-06.
- Repository HEAD at verification: `999a594eee244f7a1510f8126d9e9f4fd7586e96`;
  implementation changes remain uncommitted.
- RED phase: focused domain and application tests failed before the reciprocal
  APIs existed with unresolved-import compiler errors.
- GREEN and refactor phase: 8 domain reciprocal tests, 6 application reciprocal
  tests, and 5 filesystem reciprocal tests passed. Existing workspace tests also
  passed.
- `cargo xtask ci`: passed, including formatting, clippy, workspace tests,
  rustdoc with warnings denied, dependency audit, coverage, and release tests.
- `cargo llvm-cov --workspace --fail-under-lines 85`: passed at 92.97% line
  coverage.
- `cargo llvm-cov --package sdd-domain --fail-under-lines 95`: passed at 96.93%
  line coverage.
- `cargo machete`: passed with no unused dependencies.
- `cargo check --manifest-path fuzz/Cargo.toml`: passed.
- `cargo fuzz --version`: passed with `cargo-fuzz 0.13.2`.
- `cargo fuzz run parse_artifact -- -runs=1`: attempted but blocked because the
  environment has stable Rust `1.95.0` and cargo-fuzz requires nightly `-Z`
  sanitizer flags; no fuzz execution result is claimed.

## Rollout And Recovery

### Rollout

- Add the pure reciprocity evaluator and opt-in application use case as a
  library-only extension to the existing workspace.
- Reuse the existing active-plus-historical `ArtifactIdentitySource`; do not
  change active-only callers or filesystem discovery behavior.
- No database, file-format, lifecycle, schema, persisted-result, deployment,
  network, retry, or background-job migration is needed.
- The validator reads one repository candidate snapshot and returns a complete
  report or a typed repository-discovery error. CLI exposure remains deferred.

### Recovery

- Per-file read, parse, recognition, structural, target, or reciprocity
  failures require no rollback: preserve diagnostics and continue processing
  all other candidates.
- A repository-level discovery failure is terminal for that invocation; correct
  repository access and rerun without claiming partial success.
- Validation performs no mutation, so an interrupted run needs no data cleanup.
  Rerunning against the same repository state is safe and must produce the same
  ordered report.
- If implementation verification fails, revert the reciprocal implementation
  and its same-commit specification/task changes together. Do not alter
  artifact content or lifecycle statuses as recovery work.

## Definition Of Done

- [x] All ordered tasks are complete with observed RED and GREEN evidence.
- [x] Every `REQ-004` functional and quality requirement is covered by a
  traceable passing test.
- [x] All approved scenario behaviors are covered by traceable tests through
  the application and real filesystem path where applicable.
- [x] Reciprocal `related` and supersession membership is verified across
  active, archived, and superseded recognized artifacts.
- [x] Empty, malformed, unresolved, missing-target, and wrong-kind values do
  not receive duplicate reciprocity findings.
- [x] Every unmatched eligible directed entry receives exactly one actionable
  source-owned diagnostic with source field, target ID, and reverse field
  context.
- [x] List order and duplicate occurrences do not change reciprocal membership
  or create duplicate findings.
- [x] Multiple failures do not suppress valid unrelated artifact results, and
  validation continues through the complete recognized set.
- [x] Structural and US-003 target diagnostics remain attributable to their
  existing owners and are not duplicated.
- [x] Active-only US-003 behavior remains unchanged, and no new port, crate,
  dependency, architectural layer, unsafe code, CLI, serialization,
  persistence, lifecycle, schema, release, or cycle behavior is introduced.
- [x] Domain purity, dependency direction, constructor injection, typed errors,
  public documentation, and lint constraints pass review.
- [x] `cargo xtask ci`, coverage thresholds, dependency audit, unused-
  dependency analysis, release tests, and fuzz-target checks have observed
  evidence recorded for `verify-feature`; any environment limitation is
  explicitly recorded.
- [x] Relevant specifications remain current in the implementation change set.

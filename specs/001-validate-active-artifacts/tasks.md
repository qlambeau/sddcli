---
id: TASK-001
title: "Active artifact validation implementation tasks"
type: implementation-tasks
status: draft
created: 2026-09-03
updated: 2026-09-03
owner: Project owner
parent: US-001
depends_on: []
requires: [REQ-001, DES-001, ADR-005]
blockers: []
related:
  - REQ-001
  - DES-001
  - ADR-002
  - ADR-003
  - ADR-004
  - ADR-005
---

# Tasks

## Implementation Approach

Implement one vertical validation slice through the approved layered Rust
design. Establish only the workspace needed by this feature, write observable
acceptance and lower-level tests before production behavior, implement the pure
domain rules before orchestration and filesystem parsing, and finish by running
the complete constitutional gate suite with recorded evidence.

The implementation must remain read-only, offline, and bounded to the eight
artifact types and canonical locations in `REQ-001`. CLI commands,
serialization, global identity and relationship checks, promotion, readiness,
persistence, and network integration remain excluded.

## Ordered Tasks

- [ ] **TASK-001-1 (FOUNDATION): Audit dependencies and establish the Rust workspace.**
  - Outcome: Create the pinned Rust toolchain and workspace configuration for
    `domain`, `application`, `adapters/artifact-filesystem`, and `xtask`; commit
    `Cargo.lock`; configure formatting, workspace lints, dependency policy,
    coverage, and the `cargo xtask ci` entry point.
  - Dependencies: None.
  - Constraints: Audit `serde`, the maintained Serde-compatible YAML parser,
    `pulldown-cmark`, `gherkin`, `thiserror`, and constitution-required test and
    analysis tooling for maintenance, licence, transitive weight, duplicate
    versions, advisories, and unsafe footprint before adding them. Keep parser
    dependencies outside `domain`. Stop and revise `DES-001` or `ADR-005` if the
    audit requires a different architecture or dependency class.
  - Verification: `cargo metadata --no-deps` resolves the intended acyclic
    workspace; every member inherits workspace lints; `Cargo.lock` exists; the
    dependency audit has no unapproved licence, advisory, or duplicate-version
    failure.

- [ ] **TASK-001-2 (RED): Add fixture-driven acceptance tests for the approved scenarios.**
  - Outcome: Add fixtures and integration tests for all seven scenarios in
    `scenarios.feature`, covering all eight valid artifact types, invalid
    artifacts, multiple violations, malformed frontmatter, excluded paths, an
    empty active set, deterministic output, and read-only behavior.
  - Dependencies: TASK-001-1.
  - Traceability: `REQ-001` FR-001 through FR-009 and every scenario in
    `scenarios.feature`.
  - Verification: Run the focused acceptance-test command and record the
    expected failing output caused by missing validation behavior, not by a
    broken fixture or test harness.

- [ ] **TASK-001-3 (RED): Specify the domain report and ordering contracts in tests.**
  - Outcome: Add unit and property tests for artifact kinds and IDs,
    diagnostics, warning failure semantics, artifact statuses, overall status,
    complete finding aggregation, fixed type order, ascending ID order, no-ID
    path fallback, and rule-ID/location ordering.
  - Dependencies: TASK-001-2.
  - Traceability: `REQ-001` FR-004, FR-005, FR-007, FR-008, and FR-009.
  - Verification: Run the focused domain tests and record RED output showing
    the report behavior is not implemented.

- [ ] **TASK-001-4 (GREEN): Implement the pure domain report model.**
  - Outcome: Implement the minimum immutable domain types and pure behavior
    needed to aggregate all diagnostics, calculate artifact and overall
    statuses, and produce the required deterministic ordering.
  - Dependencies: TASK-001-3.
  - Constraints: Use no filesystem, parser, async, serialization, or framework
    dependencies in `domain`; keep invalid states unrepresentable and public
    items attributable to `REQ-001`.
  - Verification: The TASK-001-3 tests pass without weakening assertions, and
    domain code passes its focused clippy and documentation checks.

- [ ] **TASK-001-5 (RED): Specify common and type-specific validation rules in tests.**
  - Outcome: Add table-driven tests for all eight artifact types covering
    canonical type values, lifecycle statuses, required metadata and headers,
    required headings and checklist sections, local ID/reference syntax,
    unresolved-value markers, and unreplaced template markers.
  - Dependencies: TASK-001-4.
  - Traceability: `REQ-001` FR-002, FR-003, FR-005, and FR-008.
  - Verification: Run the focused rule tests and record RED output showing each
    required rule family fails before implementation.

- [ ] **TASK-001-6 (GREEN): Implement the pure validation policy.**
  - Outcome: Implement common rules and the distinct PRD, EPIC, user story,
    Gherkin, requirements, design, ADR, and TASK rule sets against normalized
    artifact snapshots, emitting stable namespaced diagnostics for every
    applicable violation.
  - Dependencies: TASK-001-5.
  - Constraints: Do not perform global uniqueness, collisions, cross-artifact
    resolution, promotion, or readiness checks.
  - Verification: The TASK-001-5 tests pass; a multi-violation snapshot returns
    every expected diagnostic exactly once and in the required order.

- [ ] **TASK-001-7 (RED): Specify application orchestration through an in-memory source.**
  - Outcome: Add application tests using an in-memory `ArtifactSource` for an
    empty set, valid and invalid candidate collections, complete aggregation,
    per-file read/parse findings, and repository-level discovery failure.
  - Dependencies: TASK-001-6.
  - Traceability: `REQ-001` FR-004, FR-005, FR-006, and FR-009.
  - Verification: Run the focused application tests and record RED output for
    the missing use case and port contract.

- [ ] **TASK-001-8 (GREEN): Implement the application validation use case and source port.**
  - Outcome: Define the narrow `ArtifactSource` port, its in-memory test double,
    typed operational error, and the use case that validates every supplied
    candidate without aborting on per-artifact failures.
  - Dependencies: TASK-001-7.
  - Constraints: Return a typed operational error only when repository
    discovery cannot be established; preserve per-file failures as path-based
    diagnostic candidates.
  - Verification: The TASK-001-7 tests pass, including empty-set success and
    complete processing after individual candidate failures.

- [ ] **TASK-001-9 (RED): Specify canonical discovery and parser adapter behavior.**
  - Outcome: Add adapter tests and fuzz targets for canonical path discovery,
    all eight artifact formats, source locations, ignored supporting/template/
    archive paths, malformed or unrecognized frontmatter, malformed Markdown
    and Gherkin, unreadable files, and untrusted parser input.
  - Dependencies: TASK-001-8.
  - Traceability: `REQ-001` FR-001, FR-002, FR-003, FR-006, and FR-009.
  - Verification: Run focused adapter tests and parser fuzz-target smoke checks;
    record RED output showing discovery and parsing behavior is absent.

- [ ] **TASK-001-10 (GREEN): Implement the filesystem source and parser adapters.**
  - Outcome: Implement canonical discovery and single-read parsing through the
    approved YAML, Markdown, and Gherkin libraries; map parser data into domain
    snapshots and map unreadable/malformed candidates into source-located
    diagnostics without stopping the run.
  - Dependencies: TASK-001-9.
  - Constraints: Do not execute artifact content, access the network, write
    files, persist results, or leak parser/filesystem types across the
    application boundary.
  - Verification: The TASK-001-9 adapter tests and fuzz-target smoke checks
    pass, and excluded files are never returned as active candidates.

- [ ] **TASK-001-11 (REFACTOR AND TRACE): Complete the vertical slice.**
  - Outcome: Make every acceptance test from TASK-001-2 pass through the real
    filesystem adapter and application use case; remove duplication, curate
    public APIs, document contracts and errors, and preserve the approved
    dependency direction.
  - Dependencies: TASK-001-10.
  - Traceability: Every `REQ-001` `FR-*` identifier and every scenario in
    `scenarios.feature` must cite at least one passing test.
  - Verification: The focused acceptance suite passes; two runs over identical
    fixtures return identical reports; before/after fixture snapshots prove no
    source, lifecycle status, or persisted result changed.

- [ ] **TASK-001-12 (VERIFY): Run and record all quality gates.**
  - Outcome: Execute the complete repository gate suite and record observed
    command output and coverage in this task document for `verify-feature`.
  - Dependencies: TASK-001-11.
  - Verification: `cargo fmt --all -- --check`,
    `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
    `cargo test --workspace --all-features`,
    `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`,
    `cargo deny check`,
    `cargo llvm-cov --workspace --fail-under-lines 85`, and
    `cargo test --release` pass; `cargo xtask ci` reproduces the constitutional
    local gate suite; unused-dependency analysis reports no unused dependency.

## Test And Verification Plan

- [ ] RED evidence is recorded for acceptance, domain report, validation-rule,
  application, and adapter tests before their matching production behavior.
- [ ] Unit tests cover every pure rule, result state, ordering branch, and error
  path, with property tests for deterministic and ordering invariants.
- [ ] Application tests cover orchestration with the in-memory `ArtifactSource`.
- [ ] Adapter tests cover real canonical filesystem layouts and parser
  boundaries without network access.
- [ ] Fuzz targets exercise parsers that consume untrusted artifact bytes.
- [ ] All seven scenarios in `scenarios.feature` pass through the integrated
  adapter and application path.
- [ ] Every specified behavior cites `REQ-001` and a relevant `FR-*` identifier
  in its test name or doc comment.
- [ ] Workspace line coverage is at least 85%, and domain coverage is at least
  95%.
- [ ] The full Rust quality suite is available and passes through
  `cargo xtask ci`.
- [ ] No CLI or serialization assertion is added because those contracts are
  deferred to EPIC-004.

## Rollout And Recovery

### Rollout

- Add library and tooling crates without exposing a user-facing command.
- Commit the pinned toolchain, workspace manifests, dependency lockfile, and
  quality configuration with the feature implementation.
- No schema migration, persisted state migration, retry policy, or deployment
  coordination is required.
- Per-file read or parse failures are returned as diagnostics in the completed
  report; only repository-level discovery failure terminates the operation.

### Recovery

- Roll back the feature source and workspace additions together if validation
  behavior or dependency policy fails verification.
- No data rollback or cleanup is required because validation is read-only and
  persists no results.
- A partial validation run requires no recovery action because it performs no
  mutation; rerunning against the same repository state is safe.

## Definition Of Done

- [ ] All ordered tasks are complete with observed RED and GREEN evidence.
- [ ] Every `REQ-001` functional requirement and every approved scenario is
  covered by a traceable passing test.
- [ ] Valid fixtures for all eight artifact types return `ok` and invalid
  fixtures report every expected diagnostic.
- [ ] Empty, malformed, unreadable, excluded, warning, multi-violation, ordering,
  deterministic, and read-only paths are verified.
- [ ] Dependency direction follows ADR-005 and parser types remain outside the
  domain and application contracts.
- [ ] No behavior from EPIC-002, EPIC-003, or EPIC-004 is implemented.
- [ ] Dependency audit, formatting, linting, tests, documentation, release tests,
  fuzz smoke checks, unused-dependency analysis, and coverage gates pass.
- [ ] Verification commands and observed output are recorded for
  `verify-feature`.
- [ ] Relevant specifications and ADRs remain current.

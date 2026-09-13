---
id: TASK-009
title: "Guarded completion transition implementation tasks"
type: implementation-tasks
status: implemented
created: 2026-09-13
updated: 2026-09-13
promoted_from: in-review
promoted_to: approved
promoted_by: Project owner
promoted_at: 1789327215
owner: TBD
parent: US-009
depends_on: [TASK-008]
requires: [REQ-009, REQ-008, DES-009, DES-008, ADR-002, ADR-007, ADR-009]
blockers: []
related:
  - PRD-001
  - EPIC-003
  - US-009
  - REQ-009
  - REQ-008
  - DES-009
  - DES-008
  - DES-007
  - ADR-002
  - ADR-007
  - ADR-009
approval:
  approved_by: Project owner
  approved_on: 2026-09-13
---

# Tasks

## Implementation Approach

Extend the existing Rust promotion workflow test-first. First add normalized
release-record and archived-packet contracts, then implement parser and
discovery support. Next add pure completion guards and a packet-archival plan,
followed by application orchestration and format-preserving filesystem commits.
Finish with scenario-equivalent acceptance tests, property and compatibility
coverage, fuzz checks, and repository quality gates.

Follow the constitutional `RED -> GREEN -> REFACTOR -> TRACE` loop. Reuse the
existing domain promotion decisions, application ports, source-conflict
protection, packet commit rollback, and test doubles. Do not add a crate,
external dependency, architectural layer, runtime release database, SQL
execution, file movement, release-record creation, CLI serialization, network
access, unsafe code, or AI judgment.

`record-release` remains responsible for compiling release records and moving
feature packets into `specs/archive/`. US-009 only validates the already moved
packet and promotes lifecycle metadata.

## Ordered Tasks

- [x] **TASK-009-1 (RED): Specify release, released-state, and archived-packet contracts.**
  - Outcome: Add failing domain tests for `Release` artifact recognition,
    `REL-NNN` identity and canonical `specs/releases/REL-NNN.md` path,
    `released` lifecycle parsing and terminal behavior, and valid
    `specs/archive/NNN-feature-slug/` packet references.
  - Dependencies: `TASK-008`.
  - Traceability: `REQ-009` FR-002, FR-006, and FR-008; scenarios `Happy: A
    complete release record becomes released`, `Happy: A relocated feature
    packet becomes archived`, and `Boundary: Terminal completion states are
    immutable`; `DES-009` normalized artifact and lifecycle design; `ADR-009`
    decisions 1 and 2.
  - Constraints: Use pure normalized values only. Observe RED before changing
    domain production code. Do not add serialization, filesystem, or database
    dependencies.
  - Verification: Focused domain tests fail because release artifacts,
    `released`, and archived packet references are not represented.

- [x] **TASK-009-2 (GREEN): Implement normalized release and archived-packet models.**
  - Outcome: Extend `ArtifactKind`, stable kind ordering, canonical path and
    identity handling, `LifecycleState`, `state_of`, structural field/heading
    rules, and `ImplementationPacketRef` for release records and relocated
    packets. Add normalized release document data for included user-story
    statuses, verification evidence, and release commit presence.
  - Dependencies: `TASK-009-1`.
  - Traceability: `REQ-009` FR-002, FR-005, FR-006, and FR-008; `DES-009`
    components and interfaces; `ADR-009` decisions 1 through 4.
  - Constraints: Keep the domain pure and free of parser, filesystem,
    serialization, and application types. Preserve existing artifact,
    promotion, readiness, identity, relationship, and packet behavior.
  - Verification: Domain model, structural-rule, identity, packet-reference,
    and existing compatibility tests pass.

- [x] **TASK-009-3 (RED): Specify release parser and discovery behavior.**
  - Outcome: Add failing adapter tests for canonical release discovery,
    non-canonical and template exclusion, release frontmatter and Markdown
    normalization, included-feature rows, verification evidence, release
    commit extraction, malformed release diagnostics, archived packet
    discovery, and repeated deterministic discovery.
  - Dependencies: `TASK-009-2`.
  - Traceability: `REQ-009` FR-005, FR-006, FR-011, and FR-013; scenarios
    `Happy: A complete release record becomes released`, `Happy: A relocated
    feature packet becomes archived`, `Failure: An incomplete release is
    rejected without mutation`, and `Boundary: Repeated failed evaluation is
    deterministic and offline`; `DES-009` parser/discovery responsibilities.
  - Constraints: Exercise existing parser and discovery boundaries. Do not
    execute SQL, move files, or add parser dependencies. Observe RED first.
  - Verification: Focused adapter tests fail because release records and their
    normalized evidence are not discovered or parsed.

- [x] **TASK-009-4 (GREEN): Implement release normalization and discovery.**
  - Outcome: Recognize `specs/releases/REL-NNN.md`, reuse generic frontmatter
    parsing, normalize included feature rows and evidence, preserve parser
    diagnostics, and include archived packet paths in identity discovery.
  - Dependencies: `TASK-009-3`.
  - Traceability: `REQ-009` FR-005, FR-006, FR-007, FR-011, and FR-013;
    `DES-009` parser/discovery design; `ADR-009` decisions 1 through 4.
  - Constraints: Keep adapters thin; source-format translation stays at the
    parser boundary. Templates and non-canonical paths remain excluded. Do not
    change release-record creation or relocation ownership.
  - Verification: Parser/discovery tests pass for valid, malformed, excluded,
    active-release, and archived-packet inputs; existing parser tests remain
    green; the existing parser fuzz target still compiles.

- [x] **TASK-009-5 (RED): Specify pure completion guards and archival planning.**
  - Outcome: Add failing domain tests for reciprocal approved supersession,
    release closure across every included feature, missing evidence and commit
    aggregation, released-record association by `US-NNN`, archive-location
    guards, terminal-state rejection, unsupported/skipped/backward transitions,
    one-target scope, unchanged-on-rejection behavior, and stable diagnostics.
  - Dependencies: `TASK-009-4`.
  - Traceability: `REQ-009` FR-001 through FR-013; all approved scenarios in
    `scenarios.feature`; `DES-009` pure completion policy and packet archival
    flow; `ADR-002`, `ADR-007`, and `ADR-009`.
  - Constraints: Tests use immutable normalized snapshots and no I/O. Write
    tests before production policy. Cover all applicable failures rather than
    weakening assertions or stopping at the first diagnostic.
  - Verification: Focused domain tests fail with expected missing completion
    policy, release association, and archival-planning behavior.

- [x] **TASK-009-6 (GREEN): Implement pure completion policy and packet archival plan.**
  - Outcome: Extend promotion decisions for `released` and terminal behavior,
    derive supersession and release facts from snapshots, aggregate and order
    completion diagnostics, and add a pure archival planner that selects only
    the five colocated packet artifacts.
  - Dependencies: `TASK-009-5`.
  - Traceability: `REQ-009` FR-001 through FR-012; all approved scenarios;
    `DES-009` pure completion policy, release evaluator, and packet archival
    planner; `ADR-009` decisions 4 through 6.
  - Constraints: Domain code remains synchronous, deterministic, and free of
    clocks, filesystem access, parser types, and serialization. Supporting
    ADRs and release records must never become archival participants.
  - Verification: Domain tests pass for successful and rejected supersession,
    release, and archive plans, complete diagnostics, terminal rejection, and
    no mutation of supplied snapshots.

- [x] **TASK-009-7 (RED): Specify application completion orchestration.**
  - Outcome: Add failing public application tests for one-target commands,
    single-artifact supersession/release, packet archival, one discovery pass,
    source-diagnostic preservation, typed target/discovery errors, clock and
    commit non-use on rejected requests, successful outcome mapping, and
    unchanged included-feature/supporting-artifact state.
  - Dependencies: `TASK-009-6`.
  - Traceability: `REQ-009` FR-001, FR-007, FR-009, FR-010, FR-011, and FR-013;
    scenarios `Failure: Invalid supersession prerequisites are aggregated`,
    `Failure: An incomplete release is rejected without mutation`, `Boundary:
    One request cannot batch completion transitions`, and `Boundary: Repeated
    failed evaluation is deterministic and offline`; `DES-009` application
    contracts; constitution rules R-SEP-03, R-TRT-09, R-TRT-16, R-TRT-20,
    R-ERR-03, and R-TST-13.
  - Constraints: Use hand-written in-memory port fakes. Do not add a new
    process-boundary port unless the approved design requires it. Observe RED
    before application production changes.
  - Verification: Focused application tests fail because completion facts,
    target routing, and typed outcome behavior are not integrated.

- [x] **TASK-009-8 (GREEN): Integrate application completion flows.**
  - Outcome: Extend existing single-artifact and packet promotion orchestration
    to derive completion facts from one discovered snapshot set, distinguish
    domain rejection from operational errors, obtain timestamps only after
    acceptance, and commit one artifact or one colocated packet atomically.
  - Dependencies: `TASK-009-7`.
  - Traceability: `REQ-009` FR-001, FR-002, FR-007, FR-009, FR-010, FR-011,
    and FR-013; `DES-009` application single-artifact and packet components;
    `ADR-007` atomic application-owned promotion decision.
  - Constraints: Preserve existing public promotion, readiness, and packet
    APIs unless an additive completion contract is required. Keep handlers
    orchestral; business guards remain in domain. Do not move files or create
    release records.
  - Verification: Application tests pass for success, all-diagnostic failure,
    typed operational failure, one discovery, no clock/commit on rejection, and
    deterministic repeated requests. Existing promotion and packet tests remain
    green.

- [x] **TASK-009-9 (RED): Specify filesystem atomicity and all-scenario acceptance behavior.**
  - Outcome: Add failing adapter and acceptance tests for format-preserving
    release status patches, five-file archive status patches, expected-source
    conflicts, batch rollback, no file movement, terminal status handling,
    supporting ADR preservation, and every approved US-009 scenario.
  - Dependencies: `TASK-009-8`.
  - Traceability: `REQ-009` FR-003 through FR-012; scenarios `Happy: An
    approved artifact is superseded by a valid successor`, `Happy: A complete
    release record becomes released`, `Happy: A relocated feature packet
    becomes archived`, all `Failure:` scenarios, and all `Boundary:`
    scenarios; `DES-009` data flow, operations, and verification approach;
    constitution rules R-SDD-02, R-TST-10, R-TST-14, R-TST-17, and R-TST-26.
  - Constraints: Assert repository bytes, paths, and observable outcomes, not
    private implementation details. Do not weaken a failing scenario to match
    an implementation. No file move may be introduced.
  - Verification: Tests fail only for the unimplemented release/archival
    promotion behavior or missing scenario-equivalent coverage.

- [x] **TASK-009-10 (GREEN): Complete atomic filesystem promotion and acceptance integration.**
  - Outcome: Implement release and archived-packet format-preserving patches,
    source-conflict protection, all-preflighted packet commits, rollback
    recovery, and public acceptance behavior without changing file locations.
  - Dependencies: `TASK-009-9`.
  - Traceability: `REQ-009` FR-004 through FR-009 and FR-011; all approved
    scenarios; `DES-009` filesystem responsibilities and recovery flow;
    `ADR-007` atomic replacement and source protection.
  - Constraints: Use existing standard-library filesystem primitives and ports.
    Do not add a crate, serialization layer, relocation operation, network
    call, or release creation path.
  - Verification: Adapter, application, domain, and acceptance tests pass;
    packet archival changes only the five packet files, preserves supporting
    artifacts, and leaves bytes and paths unchanged on rejected or conflicting
    operations.

- [x] **TASK-009-11 (REFACTOR/TRACE): Complete property, compatibility, and fuzz coverage.**
  - Outcome: Add `proptest` coverage for snapshot-order invariance and repeated
    decision equality, verify stable diagnostic ordering, add trace comments for
    every requirement/scenario test, and preserve all existing validator,
    identity, relationship, readiness, promotion, and parser behavior.
  - Dependencies: `TASK-009-10`.
  - Traceability: `REQ-009` FR-009 through FR-013; `DES-009` verification and
    compatibility requirements; constitution rules R-SDD-02, R-SDD-05,
    R-TST-01 through R-TST-05, R-TST-16 through R-TST-19, R-DOC-01 through
    R-DOC-04, and R-TOOL-02 through R-TOOL-05.
  - Constraints: Refactor only after green behavior is observed. Do not add
    lint suppressions, weaken assertions, claim unavailable fuzz execution, or
    alter approved behavior.
  - Verification: Property tests pass, all public APIs have required docs,
    parser fuzz target compilation remains available, and compatibility tests
    pass without new warnings.

- [x] **TASK-009-12 (VERIFY): Execute and record the complete Rust quality gates.**
  - Outcome: Run repository quality gates and record observed output,
    test counts, coverage, dependency audit, release tests, release/parser
    checks, and toolchain limitations in this task document.
  - Dependencies: `TASK-009-11`.
  - Traceability: All `REQ-009` quality requirements; all US-009 scenarios;
    `DES-009` verification approach; `ADR-002`, `ADR-007`, and `ADR-009`;
    constitution rules R-AGT-05, R-AGT-07, R-SDD-02, R-TST-16 through
    R-TST-19, and R-TOOL-04 through R-TOOL-06.
  - Constraints: Run the existing Rust gates without .NET or frontend checks.
    Record stable/nightly fuzz limitations honestly and do not weaken a gate.
  - Verification: `cargo xtask ci`, `cargo machete`,
    `cargo check --manifest-path fuzz/Cargo.toml`, `cargo fuzz --version`, and
    the available parser fuzz smoke command are observed and recorded.

## Test And Verification Plan

- [x] Domain unit tests cover release artifact contracts, `released` lifecycle
      behavior, reciprocal supersession, release evidence, feature-state
      aggregation, archive association, terminal rejection, single-target
      validation, and deterministic diagnostic ordering.
- [x] Domain property tests cover completion equality under reordered snapshots,
      repeated failed decisions, packet archival participant bounds, and stable
      diagnostics using the existing `proptest` dependency.
- [x] Adapter integration tests cover canonical release discovery, malformed
      release rows, verification and commit extraction, archived packet paths,
      format-preserving patches, source conflicts, rollback, and no movement.
- [x] Application integration tests cover one discovery call, typed errors,
      one target/transition, rejected-request non-mutation, success outcomes,
      and existing in-memory port contracts.
- [x] Acceptance tests cover all ten approved scenarios in
      `specs/009-enforce-guarded-completion-transitions/scenarios.feature`.
- [x] Compatibility tests preserve existing validation, identity, relationship,
      reciprocal, cycle, readiness, single-promotion, packet-promotion, and
      parser behavior.
- [x] Rust quality gates are passing through `cargo xtask ci`; no .NET or
      frontend gates apply to this Rust-only slice.
- [x] Fuzz manifest, parser fuzz target, and available smoke checks are run and
      limitations are recorded explicitly.

## Verification Evidence

Observed on 2026-09-13T19:59:43Z against implementation working tree at
`5385708` (before the implementation commit):

- `cargo xtask ci`: PASS. Workspace coverage was 92.21% regions / 92.28%
  lines; domain coverage was 95.77% regions / 96.25% lines. Workspace tests,
  strict clippy, formatting, release tests, and documentation tests passed.
- `cargo machete`: PASS; no unused dependencies reported.
- `cargo check --manifest-path fuzz/Cargo.toml`: PASS.
- `cargo fuzz --version`: PASS (`cargo-fuzz 0.13.2`).
- `cargo fuzz run parse_artifact -- -runs=100`: attempted, but blocked by the
  stable toolchain because cargo-fuzz requires nightly `-Zsanitizer=address`;
  no fuzz execution is claimed.
- `git diff --check`: PASS.
- .NET and frontend gates were not applicable to this Rust-only feature.

## Rollout And Recovery

### Rollout

- Add release-record recognition and completion guards behind the existing
  application promotion boundaries.
- Keep release-record creation and packet relocation in `record-release`.
- Apply status and promotion-metadata patches only after complete preflight;
  do not move files or create runtime release state.
- Preserve existing active artifact discovery and existing promotion behavior;
  canonical release records and archived packets become recognized inputs.

### Recovery

- Domain prerequisite failures return all deterministic diagnostics and perform
  no write, clock read, or commit.
- Source conflicts abort before replacement and preserve the current source.
- Packet archival preflights every expected source, uses existing temporary
  replacement and rollback behavior, and never relocates files.
- Malformed release records remain source/structural diagnostics and cannot
  satisfy release or archival guards.
- If implementation reveals a mismatch with US-009, REQ-009, DES-009, or
  ADR-009, stop and revise the specification before changing code.
- If a quality gate fails, retain the failure evidence, correct the cause, and
  rerun the affected gate without weakening assertions.

## Definition Of Done

- [x] All ordered tasks are complete with observed RED and GREEN evidence.
- [x] Every `REQ-009` functional and quality requirement is covered by a
      traceable passing test.
- [x] Every approved US-009 scenario is covered by an acceptance test.
- [x] Release records are recognized canonically and normalized deterministically.
- [x] Supersession requires approved reciprocal successor links.
- [x] Release promotion requires implemented or archived included features,
      verification evidence, and a release commit.
- [x] Archive promotion requires a relocated packet and a validated released
      record, and changes no file location.
- [x] Failed completion requests return all diagnostics and mutate nothing.
- [x] Terminal, unsupported, skipped, backward, and batched transitions are
      rejected.
- [x] Supporting ADRs and release records are not mutated by packet archival.
- [x] Existing validation, readiness, identity, relationship, cycle, and
      promotion behavior remains compatible.
- [x] No new crate, workspace member, external dependency, architectural layer,
      unsafe code, lint suppression, or constitutional deviation is introduced.
- [x] Public APIs, errors, and normalized release contracts are documented.
- [x] `cargo xtask ci`, coverage thresholds, dependency audit, release tests,
      unused-dependency analysis, fuzz manifest checks, and available parser fuzz
      smoke checks have observed evidence recorded in this document.

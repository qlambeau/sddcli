---
id: TASK-008
title: "Spec-Ready evaluation implementation tasks"
type: implementation-tasks
status: implemented
created: 2026-09-13
updated: 2026-09-13
promoted_from: in-review
promoted_to: approved
promoted_by: Project owner
promoted_at: 1789321002
owner: TBD
parent: US-008
depends_on: [TASK-007]
requires: [REQ-008, DES-008, ADR-002, ADR-007, ADR-008]
blockers: []
related:
  - PRD-001
  - EPIC-003
  - US-008
  - REQ-008
  - DES-008
  - DES-007
  - ADR-002
  - ADR-007
  - ADR-008
approval:
  approved_by: Project owner
  approved_on: 2026-09-13
---

# Tasks

## Implementation Approach

Extend the existing layered Rust validator with a pure Spec-Ready evaluation
capability. First expand the normalized artifact model and parser boundary for
schema documents and explicit Gherkin scenario coverage. Then implement the
readiness predicate as a pure domain evaluator over immutable snapshots, followed
by application orchestration through the existing identity-source port. Finish
with scenario-equivalent acceptance coverage and repository quality gates.

Follow the constitutional `RED -> GREEN -> REFACTOR -> TRACE` loop. Write and
observe failing tests before each production implementation. Reuse existing
normalized snapshots, packet references, validators, diagnostic ordering,
application ports, and in-memory fakes. Do not add a crate, workspace member,
external dependency, runtime database, SQL execution, migration system, CLI
serialization, release behavior, network access, unsafe code, or AI judgment.

Schema support in this slice means recognizing and validating SDD document
artifacts at `specs/schema/DB-NNN.md` and `specs/schema/TABLE-NNN.md`. It does
not create database state or perform persistence operations.

## Ordered Tasks

- [x] **TASK-008-1 (RED): Specify normalized schema and scenario-coverage contracts.**
  - Outcome: Add domain tests for `Database` and `Table` artifact kinds,
    canonical paths, type tokens, identifier prefixes, stable kind ordering,
    path identity recognition, schema metadata shapes, and the four exact
    scenario coverage prefixes `Happy:`, `Alternate:`, `Failure:`, and
    `Boundary:`.
  - Dependencies: `TASK-007`.
  - Traceability: `REQ-008` FR-002, FR-005, and FR-012; scenarios `Happy: A
    complete approved packet with implemented dependencies is Spec-Ready`,
    `Alternate: Packet context includes supporting artifacts but excludes
    unrelated artifacts`, and `Failure: All applicable unmet conditions are
    reported deterministically`; `DES-008` normalized artifact and Gherkin
    coverage model; `ADR-008` decisions 1 through 5.
  - Constraints: Tests use normalized domain values only. Do not add parser,
    filesystem, application, serialization, or database dependencies. Do not
    change production model code before observing RED.
  - Verification: Focused domain tests fail because schema kinds, schema link
    values, and scenario category values are not yet represented.

- [x] **TASK-008-2 (GREEN): Implement normalized schema and scenario-coverage models.**
  - Outcome: Extend `ArtifactKind`, canonical path and identity handling,
    metadata contracts, document snapshots, and stable ordering for DB/TABLE
    artifacts and scenario coverage categories. Add structural schema rules,
    expected target policies, and bidirectional database/table link diagnostics.
  - Dependencies: `TASK-008-1`.
  - Traceability: `REQ-008` FR-002, FR-003, FR-005, and FR-012; `DES-008`
    normalized artifact and schema model; `ADR-008` decisions 1, 2, and 6.
  - Constraints: Keep the domain pure and free of parser, filesystem, YAML,
    database, and serialization types. Preserve existing eight artifact kinds,
    validator behavior, diagnostic ordering, and no-unsafe-code rules. Add no
    new dependency or crate.
  - Verification: Domain model, structural-rule, identity, relationship, and
    schema-link tests pass; existing domain tests are updated only for the
    intentional ten-kind model and remain behaviorally compatible.

- [x] **TASK-008-3 (RED): Specify schema discovery and Gherkin prefix parsing.**
  - Outcome: Add adapter tests proving canonical DB/TABLE discovery, template
    and non-canonical exclusion, schema frontmatter and Markdown normalization,
    malformed schema diagnostics, exact case-sensitive scenario-prefix
    extraction, unprefixed scenario handling, and malformed-Gherkin behavior.
  - Dependencies: `TASK-008-2`.
  - Traceability: `REQ-008` FR-002, FR-005, and FR-013; scenarios `Happy: A
    complete approved packet with implemented dependencies is Spec-Ready`,
    `Alternate: Packet context includes supporting artifacts but excludes
    unrelated artifacts`, and `Boundary: Repeated evaluation is deterministic
    and read-only`; `DES-008` schema validation integration and Gherkin coverage
    model; `ADR-008` decisions 1 through 5.
  - Constraints: Exercise the existing parser and discovery adapter boundaries.
    Do not create runtime database files, execute DDL, or add parser
    dependencies. Observe RED before implementing parser/discovery changes.
  - Verification: Focused adapter tests fail because DB/TABLE paths are not
    discovered and scenario category membership is not populated.

- [x] **TASK-008-4 (GREEN): Implement schema discovery and Gherkin prefix extraction.**
  - Outcome: Extend canonical path discovery for `specs/schema/DB-NNN.md` and
    `specs/schema/TABLE-NNN.md`; reuse generic frontmatter/Markdown parsing for
    schema documents; and classify Gherkin scenario names using only the exact
    approved prefixes.
  - Dependencies: `TASK-008-3`.
  - Traceability: `REQ-008` FR-002, FR-005, FR-012, and FR-013; `DES-008`
    schema validation integration; `ADR-008` decisions 1 through 4.
  - Constraints: Keep templates and non-canonical supporting files excluded.
    Preserve source diagnostics and existing Gherkin step parsing. Do not
    execute schema content, mutate files, or introduce a dependency.
  - Verification: Filesystem parser/discovery tests pass for valid, malformed,
    excluded, prefixed, unprefixed, and repeated scenario inputs; parser fuzz
    target compilation remains available.

- [x] **TASK-008-5 (RED): Specify pure readiness scope and predicate behavior.**
  - Outcome: Add domain tests for selected packet scope, supporting ADR/DB/TABLE
    inclusion, parent PRD/epic prerequisite context, unrelated exclusion, exact
    `approved` state checks, all normative predicate conditions, missing scenario
    categories, aggregate diagnostics, ready/not-ready classification, and
    read-only result invariants.
  - Dependencies: `TASK-008-4`.
  - Traceability: `REQ-008` FR-001 through FR-005 and FR-009 through FR-013;
    scenarios `Happy: A complete approved packet with implemented dependencies
    is Spec-Ready`, `Alternate: Packet context includes supporting artifacts but
    excludes unrelated artifacts`, `Failure: All applicable unmet conditions
    are reported deterministically`, `Boundary: An artifact beyond approval is
    not currently Spec-Ready`, and `Failure: An invalid packet reference
    returns an operational error`; `DES-008` pure readiness evaluation.
  - Constraints: Use immutable normalized snapshots only. Do not add I/O,
    application ports, parser calls, or filesystem assertions to domain unit
    tests. Every test documents its `REQ-008` or scenario traceability and
    observes RED before production readiness logic exists.
  - Verification: Focused domain tests fail because readiness result types,
    predicate evaluation, scope resolution, and category diagnostics are absent.

- [x] **TASK-008-6 (GREEN): Implement the pure Spec-Ready evaluator.**
  - Outcome: Add readiness result/decision types, stable snapshot indexes,
    packet-context selection, exact lifecycle checks, predicate diagnostics,
    schema-link composition, recursive feature-dependency traversal, dependency
    evidence checks, shared-root memoization, canonical cycle diagnostics, and
    deterministic result ordering.
  - Dependencies: `TASK-008-5`.
  - Traceability: `REQ-008` FR-001 through FR-010 and FR-012 through FR-013;
    all readiness scenarios; `DES-008` pure readiness evaluation, schema
    integration, data flow, and diagnostic contracts; `ADR-002` and `ADR-008`.
  - Constraints: Keep all decisions in the domain and make them deterministic.
    Do not re-read source text, access the filesystem, call a clock, mutate
    snapshots, serialize results, or introduce a readiness status. Implemented
    dependencies require recorded verification evidence without being required
    to remain `approved`.
  - Verification: Domain tests pass for successful readiness, complete failure
    aggregation, exact-state boundaries, supporting-schema links, missing
    categories, recursive dependencies, shared dependency deduplication, cycle
    termination, and repeated equal results. Property tests prove snapshot
    reorder invariance and deterministic cycle/result ordering.

- [x] **TASK-008-7 (RED): Specify application readiness orchestration and errors.**
  - Outcome: Add public application tests for command construction, one
    identity-source discovery, missing-root operational errors, discovery
    failures, ready/not-ready outcome mapping, preservation of source
    diagnostics, deterministic repeated evaluation, and no-write behavior.
  - Dependencies: `TASK-008-6`.
  - Traceability: `REQ-008` FR-001, FR-009, FR-010, FR-011, FR-012, and FR-013;
    scenarios `Happy: A complete approved packet with implemented dependencies
    is Spec-Ready`, `Failure: An invalid packet reference returns an operational
    error`, and `Boundary: Repeated evaluation is deterministic and read-only`;
    `DES-008` application orchestration and interface contracts.
  - Constraints: Use the existing application public API style and a
    hand-written in-memory `ArtifactIdentitySource` fake. Prove discovery occurs
    once and no write-capable port or clock is invoked. Do not add application
    production types before observing RED.
  - Verification: Focused application tests fail because readiness commands,
    typed errors, evaluator orchestration, and result exports are absent.

- [x] **TASK-008-8 (GREEN): Implement application readiness orchestration.**
  - Outcome: Add `EvaluateSpecReadyCommand`, `SpecReadyEvaluator`, typed
    `ReadinessError`, in-memory source test doubles, and curated public exports.
    Discover candidates once, distinguish missing roots from valid not-ready
    packets, invoke the pure evaluator, and return deterministic results.
  - Dependencies: `TASK-008-7`.
  - Traceability: `REQ-008` FR-001, FR-009, FR-010, FR-011, FR-012, and FR-013;
    `DES-008` application orchestration; constitution rules R-SEP-03,
    R-TRT-01, R-TRT-09, R-TRT-16, R-TRT-20, R-ERR-03, and R-DIR-12.
  - Constraints: Preserve `ArtifactIdentitySource`, existing validators,
    promotion APIs, and typed error boundaries. The handler orchestrates only;
    no filesystem or parser types escape the application crate. Do not add a
    CLI, serialization, network call, database, or new dependency.
  - Verification: Application tests pass for one discovery call, ready and
    not-ready decisions, missing-root errors, source failures, complete
    diagnostics, repeated deterministic results, and unchanged source state.

- [x] **TASK-008-9 (RED): Specify end-to-end readiness and schema acceptance behavior.**
  - Outcome: Add public application/adapter acceptance tests for all nine
    approved Gherkin scenarios, including complete readiness, supporting schema
    context, unrelated exclusion, aggregate predicate failures, unimplemented
    dependencies, shared dependency deduplication, dependency cycles, exact
    approved-state boundaries, invalid references, and read-only repeatability.
  - Dependencies: `TASK-008-8`.
  - Traceability: All `REQ-008` functional requirements and every scenario in
    `scenarios.feature`; `DES-008` verification approach; `ADR-008`.
  - Constraints: Use temporary repositories for real parsing/discovery and
    application-owned ports for orchestration. Assert repository bytes and
    discovery scope, not private implementation details. Do not weaken a
    failing assertion to accommodate implementation behavior.
  - Verification: Focused acceptance tests fail only where the integrated
    readiness behavior is absent or incomplete, with each failure mapped to a
    named requirement or scenario.

- [x] **TASK-008-10 (GREEN/REFACTOR/TRACE): Complete compatibility and traceability.**
  - Outcome: Make all approved US-008 scenario-equivalent tests pass, preserve
    existing validation/promotion behavior, document public APIs, curate exports,
    remove duplication, and add `REQ-008` trace comments to tests.
  - Dependencies: `TASK-008-9`.
  - Traceability: All `REQ-008` functional and quality requirements; all nine
    approved scenarios; constitution rules R-SDD-02, R-SDD-05, R-DIR-06,
    R-DIR-11, R-DIR-12, R-TST-01 through R-TST-05, R-TST-10, R-TST-16,
    R-TST-17, R-DOC-01 through R-DOC-04, and R-TOOL-02 through R-TOOL-04.
  - Constraints: Do not alter approved behavior, add unsupported schema
    semantics, change promotion behavior, or introduce CLI/release/persistence
    scope. If behavior differs from the packet, stop and revise specifications
    before changing code.
  - Verification: Unit, property, application, adapter, and acceptance tests
    pass; all public items have documentation and error docs; existing active,
    identity, relationship, reciprocal, cycle, parser, report, and promotion
    tests retain their approved expectations.

- [x] **TASK-008-11 (RED/GREEN): Complete parser and schema fuzz-regression coverage.**
  - Outcome: Verify the existing `parse_artifact` fuzz target covers DB/TABLE
    frontmatter and malformed scenario names without panics, and add a minimal
    regression seed or parser unit case only when needed by observed failures.
  - Dependencies: `TASK-008-10`.
  - Traceability: `REQ-008` FR-002, FR-005, FR-012, and FR-013; `DES-008`
    verification approach; constitution rules R-TST-19 and R-TOOL-05.
  - Constraints: Reuse the existing fuzz target and dependencies. Do not add
    unsafe code, weaken parser assertions, or claim a fuzz run that the pinned
    stable toolchain cannot execute.
  - Verification: Fuzz manifest compilation and parser fuzz target checks pass;
    available fuzz smoke output and stable/nightly limitations are recorded.

- [x] **TASK-008-12 (VERIFY): Execute and record complete Rust quality-gate evidence.**
  - Outcome: Execute all repository quality gates and record observed output,
    test counts, coverage, dependency results, release results, schema/parser
    checks, and environment limitations in this task document through
    `verify-feature`.
  - Dependencies: `TASK-008-11`.
  - Traceability: All `REQ-008` quality requirements; `DES-008` verification
    approach; `ADR-002`, `ADR-007`, and `ADR-008`; constitution rules R-AGT-05,
    R-AGT-07, R-SDD-02, R-TST-16 through R-TST-19, R-TOOL-04, and R-TOOL-05.
  - Verification: Run `cargo xtask ci`, including formatting, Clippy, workspace
    tests, rustdoc with warnings denied, dependency audit, workspace coverage
    of at least 85%, domain coverage of at least 95%, and release tests. Also
    run `cargo machete`, `cargo check --manifest-path fuzz/Cargo.toml`,
    `cargo fuzz --version`, and the available parser fuzz smoke command. Record
    limitations without claiming an unexecuted gate. .NET and frontend gates
    are not applicable.

## Test And Verification Plan

- [x] Domain unit tests cover schema kinds, canonical paths, schema metadata,
      schema-link invariants, scenario category extraction values, exact
      approved states, packet scope, parent/epic context, predicate checks,
      aggregate failures, evidence checks, ready/not-ready results, and typed
      diagnostic ordering.
- [x] Domain property tests cover readiness equality under reordered snapshots,
      deterministic diagnostics, shared dependency memoization, and cycle
      termination using the existing `proptest` dependency.
- [x] Adapter integration tests cover canonical DB/TABLE discovery, malformed
      and excluded schema files, frontmatter/headings, scenario prefixes, and
      preservation of existing parser diagnostics.
- [x] Application integration tests cover public command/evaluator behavior with
      in-memory sources, one discovery call, missing-root errors, source errors,
      ready/not-ready results, and no mutation.
- [x] Acceptance tests cover all nine approved `US-008` scenario headings with
      temporary repository fixtures and byte-preservation assertions.
- [x] Compatibility tests prove existing validators, reports, artifact kinds,
      single-artifact promotion, and packet promotion remain behaviorally
      compatible.
- [x] Rust quality gates are planned through `cargo xtask ci`; no .NET or
      frontend gates apply to this Rust-only slice.
- [x] Fuzz manifest, parser fuzz target, and available smoke checks are run and
      limitations are recorded explicitly.

## Observed Verification Evidence (2026-09-13)

- Baseline commit: `025937a2ac78f71355fdcbc9a0b9d21857f5a73c`; the US-008
  implementation remains uncommitted in the working tree.
- `cargo xtask ci`: passed all eight gates: format, Clippy, workspace tests,
  rustdoc with warnings denied, dependency audit, workspace coverage, domain
  coverage, and release tests.
- Observed final line coverage: 91.51% workspace-wide and 95.20% in
  `sdd-domain`, exceeding the 85% and 95% floors.
- `cargo machete`: passed; no unused dependencies reported.
- `cargo check --manifest-path fuzz/Cargo.toml`: passed.
- `cargo fuzz --version`: passed (`cargo-fuzz 0.13.2`).
- `cargo fuzz run parse_artifact -- -runs=1`: attempted and blocked by the
  installed stable `1.95.0` toolchain because cargo-fuzz requires nightly
  sanitizer `-Z` options. No fuzz execution result is claimed and no gate was
  weakened.
- No .NET or frontend gates apply to this Rust-only feature.

## Rollout And Recovery

### Rollout

- Add readiness evaluation, schema-document recognition, and Gherkin coverage
  extraction inside the existing domain/application/adapter boundaries.
- Keep evaluation opt-in until a future CLI composes the application use case.
- Do not rewrite existing specifications, create runtime database state, run SQL,
  move files, promote statuses, create release records, or persist readiness.
- Existing repositories without schema documents continue to use the existing
  validation behavior; only canonical schema paths become recognized.

### Recovery

- Discovery failures and missing packet roots return typed operational errors
  without mutation or rollback.
- Valid packets with unmet predicates return complete not-ready diagnostics;
  there is no write phase to recover.
- Malformed schema or Gherkin content remains a source/structural diagnostic and
  does not satisfy readiness.
- If dependency traversal detects a cycle, it terminates the branch, reports the
  canonical cycle, and continues evaluating other reachable dependencies.
- If implementation reveals a mismatch with the approved scenario, requirement,
  design, or ADR, stop and revise the specification before changing code.
- If a quality gate fails, preserve the failure evidence, correct the cause, and
  rerun the affected gate; do not weaken assertions or skip the gate.

## Definition Of Done

- [x] All ordered tasks are complete with observed RED and GREEN evidence.
- [x] Every `REQ-008` functional and quality requirement is covered by a
      traceable passing test.
- [x] Every approved `US-008` scenario heading is covered by an acceptance test.
- [x] DB/TABLE schema artifacts are recognized only at canonical paths and their
      frontmatter, target kinds, identities, and bidirectional links are
      validated deterministically.
- [x] Gherkin scenario names with all four exact prefixes satisfy category
      coverage; missing categories produce actionable diagnostics.
- [x] A complete approved packet with valid recursive dependencies returns
      Spec-Ready and no unmet-condition diagnostics.
- [x] Incomplete, blocked, incorrectly staged, cyclic, or dependency-blocked
      packets return complete deterministic not-ready results.
- [x] Shared dependency packets are evaluated once and their findings are not
      duplicated.
- [x] Invalid or nonexistent root references return typed operational errors.
- [x] Readiness never mutates packet, dependency, schema, unrelated, or metadata
      files and requires no network or AI service.
- [x] Existing validation, identity, relationship, cycle, and promotion behavior
      remains compatible.
- [x] No new crate, workspace member, external dependency, architectural layer,
      unsafe code, lint suppression, or constitutional deviation is introduced.
- [x] Public APIs and typed errors are documented and exported deliberately.
- [x] `cargo xtask ci`, coverage thresholds, dependency audit, release tests,
      unused-dependency analysis, fuzz manifest checks, and available parser fuzz
      smoke checks have observed evidence recorded in this document.

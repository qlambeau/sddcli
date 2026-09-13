---
id: TASK-007
title: "Atomic implementation packet promotion implementation tasks"
type: implementation-tasks
status: implemented
created: 2026-09-13
updated: 2026-09-13
promoted_from: in-review
promoted_to: approved
promoted_by: Project owner
promoted_at: 1789313669
owner: TBD
parent: US-007
depends_on: [TASK-006]
requires: [REQ-007, DES-007, ADR-002, ADR-007]
blockers: []
related:
  - EPIC-003
  - US-007
  - REQ-007
  - DES-007
  - DES-006
  - ADR-002
  - ADR-007
approval:
  approved_by: Project owner
  approved_on: 2026-09-13
---

# Tasks

## Implementation Approach

Extend the approved single-artifact promotion capability with a packet-level
planning and commit layer. Keep packet inclusion, next-state derivation,
skipped participants, and failure aggregation pure in the domain. Keep repository
discovery, validation composition, per-artifact facts, clock usage, and batch
commit orchestration in the application crate. Keep format-preserving multi-file
patching and source-conflict protection in the existing artifact-filesystem
adapter.

Follow the constitutional RED -> GREEN -> REFACTOR -> TRACE loop. Write failing
tests before production code for each layer. Reuse existing normalized snapshots,
relationship validators, single-artifact promotion policy, source ports,
promotion metadata format, patchers, and filesystem adapter. Do not add a crate,
workspace member, dependency, architectural layer, CLI behavior, release-record
behavior, schema, database, network access, unsafe code, or AI judgment.

## Ordered Tasks

- [x] **TASK-007-1 (RED): Specify pure packet promotion planning in domain tests.**
  - Outcome: Add traceable unit and property tests for packet references,
    colocated artifact inclusion, direct and recursive implementation-packet
    supporting artifact inclusion, outside-context exclusion, parent PRD/epic
    default exclusion, next-state derivation, skipped participants,
    all-skipped successful no-op planning, per-artifact fact precedence,
    complete failure aggregation, deterministic ordering, and accepted plan
    ordering.
  - Dependencies: `TASK-006`.
  - Traceability: `REQ-007` FR-001 through FR-009 and FR-013; scenarios
    `A complete draft packet enters review atomically`, `Mixed packet artifacts
    advance to their own next valid states`, `Implementation-packet supporting
    artifacts are included through direct references`, `Implementation-packet
    supporting artifacts are included recursively`, `Artifacts outside the
    implementation packet context are excluded`, `Parent PRD and epic are not
    included by default`, and `Included not-advanceable artifacts are reported
    as skipped without error`.
  - Constraints: Tests use normalized domain values only. Do not add I/O,
    filesystem, parser, clock, serialization, or test-double dependencies to
    domain. Do not implement production behavior before observing RED.
  - Verification: Focused domain packet-promotion tests fail because packet
    planner types and behavior are absent, not because test setup is invalid.

- [x] **TASK-007-2 (GREEN): Implement the pure packet promotion planner.**
  - Outcome: Add packet reference, participant, participant role, packet fact,
    packet plan, skipped participant, and packet decision types. Implement
    inclusion/exclusion, recursive support traversal, per-artifact fact matching,
    next-state derivation, skipped handling, all-skipped successful no-op plan,
    single-artifact policy delegation, and deterministic diagnostic ordering.
  - Dependencies: `TASK-007-1`.
  - Traceability: `REQ-007` FR-001 through FR-009 and FR-013; `DES-007` domain
    packet planning; `ADR-002`; `ADR-007`.
  - Constraints: Domain remains pure and deterministic. Do not expose YAML,
    Markdown, Gherkin, filesystem, clock, or adapter types. Do not change
    existing single-artifact promotion behavior.
  - Verification: Domain packet tests and existing domain tests pass; property
    tests prove deterministic accepted and rejected packet planning.

- [x] **TASK-007-3 (RED): Specify application packet promotion orchestration with port tests.**
  - Outcome: Add public application tests and in-memory fakes for packet
    command construction, one discovery pass, validation and relationship
    diagnostic composition, per-artifact confirmations by ID and type, accepted
    mixed-state packet plans, skipped participant results, all-skipped
    successful no-op without writes, missing confirmation failures, complete
    diagnostic aggregation, source conflicts, typed discovery and clock
    failures, and no-write rejection.
  - Dependencies: `TASK-007-2`.
  - Traceability: `REQ-007` FR-007 through FR-015; scenarios `Missing
    per-artifact confirmation fails the whole packet promotion`, `Multiple
    prerequisite failures are reported without mutation`, `Source conflict
    prevents all packet mutations`, `Successful packet promotion changes only
    advancing artifacts`, and `Repeated failed packet promotion requests are
    deterministic and read-only`.
  - Constraints: Tests use application public API and hand-written in-memory
    fakes. Rejected decisions must not invoke batch commit. All-skipped no-op
    must not invoke batch commit. Accepted advancing plans commit once. Do not
    add production ports or handlers before observing RED.
  - Verification: Focused application tests fail because packet command,
    packet result, batch store port, and handler are absent.

- [x] **TASK-007-4 (GREEN): Implement application packet promotion orchestration.**
  - Outcome: Add `PromotePacketCommand`, packet result types, batch promotion
    store port operations, in-memory packet store fake, and `PacketPromoter`.
    Discover candidates once, compose existing validators, derive per-artifact
    facts, request one timestamp when needed, invoke the pure packet planner,
    and call batch commit only for accepted advancing plans.
  - Dependencies: `TASK-007-3`.
  - Traceability: `REQ-007` FR-001 and FR-007 through FR-015; `DES-007`
    application orchestration and interfaces.
  - Constraints: Preserve existing `Promoter`, `ArtifactPromotionStore`,
    `ArtifactIdentitySource`, validators, reports, and errors. Port signatures
    remain in domain/application terms. Do not expose filesystem or parser
    errors. Use constructor injection and typed errors.
  - Verification: Application packet tests pass for accepted, rejected, skipped,
    all-skipped no-op, conflict, discovery, clock, and deterministic paths.
    Existing application tests remain green.

- [x] **TASK-007-5 (RED): Specify filesystem batch promotion behavior in adapter tests.**
  - Outcome: Add temporary-repository integration tests for batch Markdown and
    Gherkin patching, mixed artifact kinds, latest promotion metadata
    replacement, expected-source conflict before write, unsupported format
    before write, skipped artifact preservation, unrelated file preservation,
    no file movement, all-skipped no-op without writes, and best-effort recovery
    on write failure after partial replacement.
  - Dependencies: `TASK-007-4`.
  - Traceability: `REQ-007` FR-010 through FR-015; scenarios `Source conflict
    prevents all packet mutations` and `Successful packet promotion changes only
    advancing artifacts`; `DES-007` batch filesystem commit.
  - Constraints: Exercise the adapter through application-owned contracts.
    Preserve source bytes outside permitted lifecycle metadata. Do not weaken
    assertions to accommodate serializer formatting. Do not implement batch
    adapter behavior before observing RED.
  - Verification: Focused adapter tests fail because concrete batch commit and
    recovery behavior are absent.

- [x] **TASK-007-6 (GREEN): Implement format-preserving filesystem batch commit.**
  - Outcome: Extend the filesystem promotion store with batch source loading and
    batch commit. Compare all expected sources before writing, patch all
    advancing artifacts in memory, write same-directory temporary files, replace
    targets atomically, leave skipped and excluded artifacts untouched, and
    perform best-effort restoration after process-local write failures.
  - Dependencies: `TASK-007-5`.
  - Traceability: `REQ-007` FR-010 through FR-015; `DES-007` batch filesystem
    commit; `ADR-007`.
  - Constraints: Reuse existing patchers and metadata format. Do not move files,
    create release records, add dependencies, add unsafe code, or change
    single-artifact commit semantics.
  - Verification: Adapter batch tests pass for patches, conflicts, malformed
    sources, skipped artifacts, unrelated files, recovery, all-skipped no-op,
    and no movement. Existing filesystem tests remain green.

- [x] **TASK-007-7 (REFACTOR AND TRACE): Complete acceptance and compatibility coverage.**
  - Outcome: Add scenario-equivalent tests for all approved `US-007` scenario
    headings, curate public exports, remove duplication, verify stable ordering,
    document public packet promotion items and error contracts, and confirm
    every `REQ-007` functional requirement has traceable tests.
  - Dependencies: `TASK-007-6`.
  - Traceability: All `REQ-007` functional and quality requirements; every
    scenario in `scenarios.feature`; constitution rules for traceability,
    dependency direction, tests, documentation, typed errors, and no unsafe
    code.
  - Constraints: Keep CLI parsing, serialization, release records, archive
    relocation, Spec-Ready evaluation, persistence, network access, and AI
    judgment out of scope. If behavior mismatch appears, stop and revise the
    approved specifications before changing code.
  - Verification: Unit, property, application, adapter, and scenario-equivalent
    tests pass; existing single-artifact promotion and validators pass without
    changed expectations; docs build cleanly.

- [x] **TASK-007-8 (VERIFY): Execute and record complete Rust quality-gate evidence.**
  - Outcome: Execute the constitutional quality suite and additional repository
    checks, then record observed output, test counts, coverage, dependency
    results, release results, fuzz-check results, and environment limitations in
    this task document through the `verify-feature` workflow.
  - Dependencies: `TASK-007-7`.
  - Traceability: All `REQ-007` quality requirements; `DES-007` verification
    approach; `ADR-007`; constitution quality-gate rules.
  - Verification: Run `cargo xtask ci`, including formatting, clippy, workspace
    tests, rustdoc with warnings denied, dependency audit, workspace coverage of
    at least 85%, domain coverage of at least 95%, and release tests. Also run
    `cargo machete`, `cargo check --manifest-path fuzz/Cargo.toml`,
    `cargo fuzz --version`, and the available parser fuzz smoke command. Record
    limitations without claiming an unexecuted gate.

## Test And Verification Plan

### Observed Verification Evidence (2026-09-13)

- Verification baseline: `0c03a98`; US-007 implementation changes are present in the working tree.
- `cargo xtask ci`: passed all format, clippy, workspace test, documentation, dependency-audit, workspace coverage, domain coverage, and release-test gates.
- Observed final coverage: 90.95% workspace line coverage and 95.76% domain line coverage.
- `cargo machete`: passed with no unused dependencies.
- `cargo check --manifest-path fuzz/Cargo.toml`: passed.
- `cargo fuzz --version`: passed (`cargo-fuzz 0.13.2`).
- `cargo fuzz run parse_artifact -- -runs=1`: attempted but unavailable on stable `1.95.0`; cargo-fuzz requires nightly `-Z` sanitizer options. No gate was weakened.
- .NET and frontend gates: not applicable; this is a Rust-only feature.

- [x] Domain unit tests: packet references, participant roles, inclusion and
      exclusion, next-state derivation, skipped participants, all-skipped no-op,
      fact matching, diagnostics, and accepted plan ordering.
- [x] Domain property tests: deterministic packet planning under reordered
      snapshots and repeated identical inputs using the existing `proptest`
      dependency.
- [x] Application integration tests: public packet promotion orchestration with
      in-memory source, packet store, and clock fakes; one discovery pass;
      preflight-before-write; rejected/no-write paths; skipped/all-skipped
      behavior; conflicts and operational failures.
- [x] Adapter integration tests: temporary Markdown and Gherkin sources, batch
      patches, latest-only metadata, expected-source conflicts, unsupported
      formats, skipped preservation, unrelated file preservation, recovery, and
      no movement.
- [x] Scenario-equivalent acceptance tests: all approved scenario headings in
      `scenarios.feature`.
- [x] Compatibility checks: existing single-artifact promotion, active-only,
      identity, relationship, reciprocal, cycle, parser, and report tests pass
      unchanged.
- [x] Rust quality gates: `cargo xtask ci`, including format, clippy, workspace
      tests, documentation, dependency audit, workspace coverage at least 85%,
      domain coverage at least 95%, and release tests.
- [x] Additional checks: `cargo machete`, fuzz manifest check, fuzz version, and
      available parser fuzz smoke command; record toolchain limitations.
- [x] .NET and frontend gates: Not applicable; this slice changes the Rust
      workspace and specifications only.

## Rollout And Recovery

### Rollout

- Add packet promotion planning, application orchestration, and filesystem batch
  commit inside the existing Rust workspace.
- Keep existing single-artifact promotion and validation APIs compatible. Packet
  promotion is opt-in until a future CLI composes it.
- Do not add a database, schema, migration, release record, archive relocation,
  network integration, or deployment change.
- On success, update only advancing artifacts' lifecycle status and latest
  promotion metadata.

### Recovery

- A failed preflight, missing confirmation, unresolved blocker, invalid
  relationship, missing evidence, unsupported format, or source conflict
  performs no write and needs no rollback.
- An all-skipped packet request performs no write and returns a successful no-op
  with skipped participants.
- A write failure during batch replacement triggers best-effort restoration of
  any replaced target from captured source bytes and returns a typed write
  failure.
- After any conflict or operational error, reload repository state, recompute the
  packet promotion decision, and retry. Never overwrite changed source.
- If implementation exposes a behavior mismatch, stop and revise the approved
  story, scenarios, requirements, or design before changing code.

## Definition Of Done

- [x] All ordered tasks are complete with observed RED and GREEN evidence.
- [x] Every `REQ-007` functional and quality requirement is covered by a
      traceable passing test.
- [x] Every approved `US-007` scenario heading is covered by scenario-equivalent
      tests.
- [x] Packet promotion accepts one packet reference and derives per-artifact next
      states without a packet-level target state.
- [x] Colocated and implementation-packet supporting artifacts are included;
      outside-context artifacts and parent PRD/epic artifacts are excluded by
      default.
- [x] Direct and recursive support relationships are resolved only within
      implementation-packet context.
- [x] Skipped participants, including all-skipped no-op packets, are reported
      without mutation or error.
- [x] Missing confirmations, blockers, invalid relationships, missing evidence,
      unsupported formats, and conflicts fail the whole request with no partial
      mutation.
- [x] Successful packet promotion mutates only advancing artifacts and records
      source state, target state, actor, and UTC Unix-second timestamp metadata.
- [x] Batch filesystem commit preserves non-promotion bytes, detects conflicts,
      performs same-directory replacements, and never moves files.
- [x] Existing single-artifact promotion and validator behavior remains
      compatible.
- [x] No new crate, workspace member, dependency, architectural layer, unsafe
      code, lint suppression, or constitutional deviation was introduced.
- [x] `cargo xtask ci`, coverage thresholds, dependency audit, release tests,
      unused-dependency analysis, and fuzz checks have observed evidence
      recorded during verification.

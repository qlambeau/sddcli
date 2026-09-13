---
id: TASK-006
title: "Single-artifact lifecycle promotion implementation tasks"
type: implementation-tasks
status: approved
created: 2026-09-07
updated: 2026-09-07
owner: TBD
parent: US-006
depends_on: [TASK-005]
requires: [REQ-006, DES-006, ADR-002, ADR-005, ADR-006, ADR-007]
blockers: []
related:
  - EPIC-003
  - US-006
  - REQ-006
  - DES-006
  - DES-005
  - ADR-002
  - ADR-005
  - ADR-006
  - ADR-007
approval:
  approved_by: Project owner
  approved_on: 2026-09-07
---

# Tasks

<!-- Tasks turn the approved design into an ordered implementation plan. Each
 task has a concrete completion check and must not silently change behavior. -->

## Implementation Approach

Extend the existing layered Rust validator with the first mutating workflow
slice. Keep lifecycle policy pure in the domain, orchestration and process
seams in the application crate, and source-format mutation in the existing
artifact-filesystem adapter.

Follow the constitutional `RED -> GREEN -> REFACTOR -> TRACE` loop. Write and
observe failing tests before each corresponding implementation. Reuse the
existing normalized snapshots, validation rules, relationship evaluators,
source ports, diagnostic model, and report ordering. Add only the approved
application promotion store and clock ports; do not change existing read-only
validators or source contracts.

The filesystem adapter will update only lifecycle status and the latest
promotion metadata, preserve all other source content, compare the captured
source before commit, and use same-directory atomic replacement. The slice
supports the eight recognized artifact types and excludes release records,
packet-level promotion, CLI parsing, serialization, archive relocation, schema,
persistence, network access, and AI judgment.

## Ordered Tasks

- [ ] **TASK-006-1 (RED): Specify the pure lifecycle policy in domain tests.**
  - Outcome: Add traceable unit and property tests for recognized lifecycle
    states, the strict forward transition matrix, guarded supersession,
    terminal-state rejection, active same-state idempotency, unsupported
    `released` targets, actor and timestamp metadata planning, all applicable
    prerequisite failures, complete diagnostics, and deterministic ordering.
  - Dependencies: `TASK-005`.
  - Traceability: `REQ-006` FR-001 through FR-007 and FR-009 through FR-013;
    scenarios `A recognized active artifact enters review`, `An artifact under
    review is approved after external human confirmation`, `An approved artifact
    becomes implemented after verification`, `An approved artifact becomes
    superseded by an approved successor`, `A same-state promotion request is
    idempotent`, `An invalid lifecycle transition is rejected`, `Supersession
    is rejected without an approved successor and valid links`, `Missing
    implementation evidence blocks promotion`, and `Multiple applicable
    failures are reported without mutation`.
  - Constraints: Tests use normalized domain values only. Do not add I/O,
    filesystem, clock, parser, serialization, or test-double dependencies to
    the domain. Do not implement production behavior before observing RED.
  - Verification: The focused domain promotion tests fail because the specified
    promotion types and policy are absent, not because the test harness is
    invalid.

- [ ] **TASK-006-2 (GREEN): Implement the pure domain promotion policy.**
  - Outcome: Add lifecycle state, actor/timestamp values, promotion request,
    prerequisite facts, promotion plan, decision result, and stable promotion
    diagnostics. Implement strict transitions, terminal-state rules, active
    same-state no-op behavior, and target-specific guards without I/O or
    mutation.
  - Dependencies: `TASK-006-1`.
  - Traceability: `REQ-006` FR-002 through FR-011; `DES-006` domain decision
    and state rules; `ADR-007` decisions 1 and 6; constitution rules
    `R-SEP-01`, `R-SEP-02`, `R-TYP-01`, `R-TYP-04`, `R-TST-01`, `R-TST-02`,
    `R-TST-18`, and `R-DOC-01`.
  - Constraints: Keep the domain independent of filesystem, YAML, Markdown,
    Gherkin, serde, clocks, and external wire formats. Do not add a dependency
    or change existing artifact and report contracts.
  - Verification: The domain RED suite passes, property tests demonstrate
    deterministic decisions and idempotency, all public items are documented,
    and no existing domain tests change their expectations.

- [ ] **TASK-006-3 (RED): Specify application orchestration with port tests.**
  - Outcome: Add public application integration tests and in-memory test-double
    contracts for repository candidate discovery, target loading, validation
    aggregation, no-write rejection, active idempotent no-op behavior, clock
    usage, accepted-plan commit, source conflicts, typed discovery failures,
    typed clock failures, and unrelated-result preservation.
  - Dependencies: `TASK-006-2`.
  - Traceability: `REQ-006` FR-001, FR-003, FR-004, and FR-011 through FR-015;
    scenarios `A same-state promotion request is idempotent`, `Invalid
    lifecycle transition is rejected`, `Multiple applicable failures are
    reported without mutation`, `A successful promotion changes only the
    requested artifact state and metadata`, and `Repeated failed promotion
    requests are deterministic and read-only`.
  - Constraints: Tests use only the application public API and hand-written
    in-memory fakes. The tests must prove that rejected decisions do not invoke
    the store, idempotent decisions do not invoke the clock or store, and
    accepted decisions commit once. Do not add production ports or handlers
    before observing RED.
  - Verification: The focused application promotion tests fail because the
    promotion ports, errors, and handler are absent, not because the fakes or
    assertions are invalid.

- [ ] **TASK-006-4 (GREEN): Implement application promotion orchestration.**
  - Outcome: Add the application-owned `ArtifactPromotionStore` and `Clock`
    ports, typed promotion errors, in-memory doubles, and the promotion use
    case. Discover the active and historical candidate set once, reuse existing
    domain validation and relationship rules, derive target promotion facts,
    invoke the pure policy, and commit accepted plans only after preflight.
  - Dependencies: `TASK-006-3`.
  - Traceability: `REQ-006` FR-001, FR-003, FR-004, and FR-011 through FR-015;
    `DES-006` application orchestration and interface contracts;
    `ADR-005`, `ADR-006`, and `ADR-007`.
  - Constraints: Preserve `ArtifactSource`, `ArtifactIdentitySource`,
    `Validator`, `IdentityValidator`, `RelationshipValidator`,
    `ReciprocalRelationshipValidator`, `CycleValidator`, and
    `ValidationReport` behavior. Keep port signatures in domain/application
    terms; do not expose filesystem, parser, or serialization errors. Use
    constructor injection and typed errors. Do not add a crate, dependency, or
    architectural layer.
  - Verification: Application tests pass for accepted transitions, complete
    rejection diagnostics, no-write paths, idempotent no-ops, conflict and
    operational failures, and deterministic results. Existing validator tests
    remain green.

- [ ] **TASK-006-5 (RED): Specify filesystem promotion behavior in adapter tests.**
  - Outcome: Add temporary-repository integration tests for Markdown
    frontmatter and Gherkin header patching, all eight artifact kinds, latest
    promotion metadata replacement, body and unrelated-byte preservation,
    archive-location checks, missing-format handling, expected-source conflicts,
    write failures, same-directory atomic replacement, system timestamps, and
    no file movement.
  - Dependencies: `TASK-006-4`.
  - Traceability: `REQ-006` FR-001, FR-008, FR-010, FR-012, and FR-014;
    scenarios `A recognized active artifact enters review`, `An implemented
    artifact is archived from its canonical archive location`, `Archival is
    rejected when the artifact has not been relocated`, `A successful promotion
    changes only the requested artifact state and metadata`, and `Repeated
    failed promotion requests are deterministic and read-only`.
  - Constraints: Exercise the adapter through its public application contract
    and temporary filesystem fixtures. Assert source bytes before and after;
    do not weaken assertions to accommodate serializer formatting. Do not
    implement the adapter before observing RED.
  - Verification: Focused adapter tests fail because the concrete promotion
    store, patchers, and system clock are absent, not because temporary
    repository setup is invalid.

- [ ] **TASK-006-6 (GREEN): Implement format-preserving atomic filesystem promotion.**
  - Outcome: Add the filesystem promotion store, Markdown and Gherkin metadata
    patchers, expected-source conflict check, same-directory temporary-file
    replacement, and system clock adapter. Update only status and the latest
    `promoted_from`, `promoted_to`, `promoted_by`, and `promoted_at` values.
  - Dependencies: `TASK-006-5`.
  - Traceability: `REQ-006` FR-001, FR-008, FR-010, FR-012, FR-014, and FR-015;
    `DES-006` filesystem adapter and atomic commit flow; `ADR-007` decisions
    2 through 6; constitution rules `R-DIR-01`, `R-DIR-03`, `R-SEP-04`,
    `R-SEP-05`, `R-TRT-01`, `R-TRT-09`, `R-TRT-10`, `R-TRT-16`, `R-ERR-03`,
    `R-ERR-08`, and `R-UNS-01`.
  - Constraints: Reuse existing parser dependencies only at the adapter
    boundary. Preserve non-promotion source bytes and file location. Never
    write an idempotent request, move an artifact, overwrite a changed source,
    or introduce unsafe code, a new dependency, or a new crate.
  - Verification: Adapter tests pass for Markdown, Gherkin, conflicts,
    malformed/unreadable sources, atomic replacement, latest-only metadata,
    unchanged body bytes, and no movement. Application tests pass against the
    concrete store and system clock.

- [ ] **TASK-006-7 (REFACTOR AND TRACE): Complete acceptance, compatibility, and documentation coverage.**
  - Outcome: Run all 13 approved scenario headings and every Scenario Outline
    example through the application use case and real filesystem store. Curate
    exports, remove duplication, verify stable diagnostic ordering, preserve
    existing validator behavior, and document every public item and error
    contract.
  - Dependencies: `TASK-006-6`.
  - Traceability: All `REQ-006` functional and quality requirements and every
    scenario in `scenarios.feature`; constitution rules `R-SDD-02`, `R-SDD-05`,
    `R-AGT-03`, `R-TST-03`, `R-TST-04`, `R-TST-05`, `R-TST-07`, `R-TST-10`,
    `R-TST-16`, `R-TST-17`, `R-TST-18`, `R-DIR-01`, `R-DIR-02`, `R-DIR-11`,
    `R-SEP-03`, `R-SEP-07`, `R-SEP-08`, `R-TRT-14`, `R-TRT-20`, `R-DOC-01`,
    `R-DOC-03`, and `R-DOC-04`.
  - Constraints: Keep packet promotion, release records, CLI parsing, exit
    codes, serialization, archive relocation, persistence, and network access
    out of scope. If implementation reveals a specification mismatch, stop and
    revise the approved specifications before changing code.
  - Verification: All focused unit, property, application, adapter, and
    scenario-equivalent tests pass; tests cite `REQ-006` and `FR-*` identifiers;
    existing active, identity, target, reciprocal, and cycle validator tests
    pass without changed expectations; documentation builds cleanly.

- [ ] **TASK-006-8 (VERIFY): Execute and record complete Rust quality-gate evidence.**
  - Outcome: Execute the constitutional quality suite and additional repository
    checks, record observed output, test counts, coverage, dependency results,
    release results, and environment limitations in this task document.
  - Dependencies: `TASK-006-7`.
  - Traceability: All `REQ-006` quality requirements; `DES-006` verification
    approach; `ADR-007`; constitution rules `R-AGT-07`, `R-SDD-02`, `R-SDD-05`,
    `R-TST-01`, `R-TST-03`, `R-TST-05`, `R-TST-16`, `R-TST-17`, `R-TST-18`,
    `R-DIR-01`, `R-DIR-02`, `R-SEP-02`, `R-TRT-01`, `R-TRT-20`, `R-DOC-01`,
    `R-DOC-03`, and `R-TOOL-04`.
  - Verification: Run `cargo xtask ci`, reproducing formatting, clippy,
    workspace tests, rustdoc with warnings denied, dependency audit, workspace
    coverage of at least 85%, domain coverage of at least 95%, and release
    tests. Also run `cargo machete`, `cargo check --manifest-path
    fuzz/Cargo.toml`, `cargo fuzz --version`, and attempt the available parser
    fuzz smoke command. Record stable-toolchain limitations without claiming an
    unexecuted gate. Confirm no new dependencies, warnings, lint suppressions,
    unsafe code, or constitutional deviations were introduced.

## Test And Verification Plan

- [ ] Domain unit tests: Pure lifecycle state, transition, guard, idempotency,
      metadata-plan, diagnostic, and error-path tests with visible Arrange / Act
      / Assert sections.
- [ ] Domain property tests: Deterministic decisions, active same-state
      idempotency, and stable diagnostic ordering using the existing `proptest`
      dependency.
- [ ] Application integration tests: Public promotion orchestration with
      in-memory identity, promotion-store, and clock doubles; one discovery,
      preflight-before-write, complete findings, no-op behavior, conflicts, and
      typed operational failures.
- [ ] Adapter integration tests: Temporary Markdown and Gherkin sources,
      format-preserving patches, latest-only metadata, source conflicts,
      atomic replacement, write failures, archive boundaries, and no movement.
- [ ] Scenario-equivalent acceptance tests: All 13 approved scenario headings
      and every Scenario Outline example in `scenarios.feature`.
- [ ] Compatibility checks: Existing active-only, identity, relationship,
      reciprocal, cycle, parser, and report tests continue to pass unchanged.
- [ ] Rust quality gates: `cargo xtask ci`, including format, clippy, workspace
      tests, documentation, dependency audit, workspace coverage at least 85%,
      domain coverage at least 95%, and release tests. Output is recorded during
      verification, not planning.
- [ ] Additional checks: `cargo machete`, fuzz manifest check, fuzz version,
      and available parser fuzz smoke command; record any toolchain limitation.
- [ ] .NET and frontend gates: Not applicable; this slice changes the Rust
      workspace and specifications only.

## Rollout And Recovery

### Rollout

- Add the pure promotion policy, application ports and use case, and filesystem
  adapter within the existing workspace.
- Keep all existing read-only validators and source ports behaviorally
  unchanged. The new promoter is opt-in until a future CLI composes it.
- Do not add a database, schema, migration, release record, archive relocation,
  network integration, or deployment change.
- On a successful request, update only the requested artifact's status and latest
  promotion metadata through one atomic filesystem replacement.

### Recovery

- A failed preflight, invalid transition, missing evidence, unresolved blocker,
  invalid relationship, terminal-state request, or rejected archive/supersession
  guard performs no write and needs no rollback.
- An active same-state request performs no write and does not obtain a timestamp.
- A repository discovery, source-read, clock, unsupported-format, or write
  failure returns a typed error. The original artifact remains unchanged; fix
  the cause and retry.
- An expected-source mismatch indicates a concurrent change. Reload the current
  repository state, recompute the decision, and retry; never overwrite the
  changed source.
- A failed atomic replacement leaves the original target intact. Temporary
  files are implementation cleanup and must not be treated as committed
  artifacts.
- If implementation exposes a behavior mismatch, stop and revise the approved
  story, scenarios, requirements, or design before changing code. Do not weaken
  tests or make code the source of truth.

## Definition Of Done

- [ ] All ordered tasks are complete with observed RED and GREEN evidence.
- [ ] Every `REQ-006` functional and quality requirement is covered by a
      traceable passing test.
- [ ] All 13 approved scenario headings and every outline example are covered.
- [ ] All eight recognized artifact types support valid review-entry promotion.
- [ ] The strict forward transition matrix and guarded supersession behavior are
      enforced.
- [ ] Archived, released, and superseded artifacts cannot be reopened, while
      active same-state requests are idempotent no-ops.
- [ ] Human approval remains an external workflow responsibility and actor
      classification is not performed by promotion.
- [ ] Verification evidence, archive placement, completion conditions, and
      supersession prerequisites are enforced before mutation.
- [ ] Failed requests report all applicable deterministic diagnostics and leave
      the target and unrelated artifacts unchanged.
- [ ] Successful transitions record only the latest source, target, actor, and
      UTC Unix-second timestamp metadata.
- [ ] Markdown and Gherkin source content outside permitted lifecycle metadata
      remains unchanged.
- [ ] Source conflicts are detected and cannot overwrite concurrent changes.
- [ ] Filesystem replacement is atomic and promotion never moves files.
- [ ] Promotion is offline, requires no AI service, and adds no persistence,
      release-record, CLI, or serialization behavior.
- [ ] Existing active, identity, target, reciprocal, cycle, parser, and report
      behavior remains compatible.
- [ ] No new crate, workspace member, dependency, architectural layer, unsafe
      code, lint suppression, or constitutional deviation was introduced.
- [ ] Domain purity, dependency direction, constructor injection, typed errors,
      public documentation, and coverage thresholds pass review.
- [ ] `cargo xtask ci`, coverage thresholds, dependency audit, release tests,
      unused-dependency analysis, and fuzz checks have observed evidence recorded
      in this document during verification.
- [ ] Relevant specifications and ADR links remain current in the implementation
      change set.

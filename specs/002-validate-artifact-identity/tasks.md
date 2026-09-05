---
id: TASK-002
title: "Repository-wide artifact identity implementation tasks"
type: implementation-tasks
status: approved
created: 2026-09-05
updated: 2026-09-05
owner: Project owner
parent: US-002
depends_on: [TASK-001]
requires: [REQ-002, DES-002, ADR-005, ADR-006]
blockers: []
related:
  - EPIC-002
  - US-002
  - REQ-002
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

## Implementation Approach

Extend the existing layered Rust validator with one repository-wide identity
vertical slice. Preserve the active-only `ArtifactSource` contract from
EPIC-001, add the approved `ArtifactIdentitySource` seam, and keep identity
decisions in pure domain code. The filesystem adapter will opt into historical
packet discovery for this use case while continuing to exclude templates and
supporting documents.

Follow the constitutional `RED -> GREEN -> REFACTOR -> TRACE` loop. Add tests
and observe their expected failures before each corresponding implementation.
Use the existing workspace crates, parser dependencies, test doubles, and
quality tooling. Do not add a crate, dependency, persistence, schema, CLI,
serialization, mutation, lifecycle promotion, or relationship graph behavior.

The archive layout used by the identity source is the release workflow's
canonical packet layout: `specs/archive/NNN-feature-slug/` containing the same
packet files and IDs as the active packet. Active PRD, epic, and ADR paths retain
their existing path-encoded identity checks.

## Ordered Tasks

- [ ] **TASK-002-1 (RED): Specify pure identity rules in domain tests.**
  - Outcome: Add traceable table-driven and property tests for the identity
    index and path-identity policy. Cover an empty set, unique IDs across active
    and historical snapshots, duplicate IDs across lifecycle states, a duplicate
    finding on every conflicting path, continued evaluation of unrelated
    snapshots, matching and mismatching PRD/epic/ADR path identities, stable
    diagnostic ordering, and deterministic repeated evaluation.
  - Dependencies: TASK-001.
  - Traceability: `REQ-002` FR-001 through FR-008; scenarios `All recognized
    current and historical identities are valid`, `Empty identity set
    succeeds`, `Duplicate IDs are reported for every conflicting artifact`,
    `Path-encoded identity mismatches remain diagnosable`, and `Identity
    conflicts do not suppress unrelated artifacts`.
  - Constraints: Keep tests pure and filesystem-free. Use existing `rstest` and
    `proptest` dependencies; do not introduce a new test or runtime dependency.
  - Verification: Focused domain tests fail for the expected missing identity
    behavior or API, not because of invalid fixtures or a broken test harness.

- [ ] **TASK-002-2 (GREEN): Implement pure identity indexing and domain report merging.**
  - Outcome: Add the minimum domain behavior needed to collect recognized
    `ArtifactId` values, detect global exact-ID collisions, derive and compare
    path-encoded PRD/epic/ADR identities, create stable actionable
    `ARTIFACT.IDENTITY.*` diagnostics, and merge identity findings into the
    existing `ArtifactResult`/`ValidationReport` contract.
  - Dependencies: TASK-002-1.
  - Traceability: `REQ-002` FR-002, FR-003, FR-004, FR-005, FR-007, and FR-008;
    postconditions for complete collision reporting, valid unrelated results,
    stable ordering, and overall failure on identity errors.
  - Constraints: Domain code remains synchronous, deterministic, read-only, and
    free of filesystem, parser, serialization, async, and framework
    dependencies. Do not reparse malformed snapshots or resolve relationships.
    Preserve existing structural diagnostics and result ordering.
  - Verification: TASK-002-1 tests pass without weakened assertions; focused
    domain clippy, documentation, and property-test checks pass; no new public
    item lacks a specification link and doc contract.

- [ ] **TASK-002-3 (RED): Specify application identity orchestration through a fake source.**
  - Outcome: Add application tests for the new identity use case and source
    contract using an in-memory source. Cover successful unique active and
    historical candidates, empty success, duplicate and path-mismatch findings,
    complete retention of unrelated candidates, malformed snapshots retaining
    EPIC-001 parser/structural findings without identity reparsing, per-file
    source diagnostics, invalid empty candidates, deterministic output, and
    terminal repository-level discovery failure.
  - Dependencies: TASK-002-2.
  - Traceability: `REQ-002` FR-001 through FR-008 and all corresponding failure,
    boundary, and deterministic scenarios in `scenarios.feature`.
  - Constraints: Test through the application public API and a hand-written
    in-memory fake. Keep the existing `ArtifactSource` tests passing and prove
    that active validation remains active-only.
  - Verification: Focused application tests fail with expected missing identity
    port/orchestration behavior before production implementation is added.

- [ ] **TASK-002-4 (GREEN): Implement the identity source port and use case.**
  - Outcome: Add the documented application-owned `ArtifactIdentitySource` port,
    its in-memory test double, and the identity validation use case. Orchestrate
    repository-wide candidates, existing structural validation, pure identity
    evaluation, complete diagnostic merging, and the existing ordered report.
  - Dependencies: TASK-002-3.
  - Traceability: `REQ-002` FR-001, FR-003, FR-005, FR-006, FR-007, and FR-008;
    `DES-002` interfaces and data-flow contract; `ADR-006` source-port
    decision.
  - Constraints: Keep port signatures in domain/application terms, return typed
    `ValidationError` values, preserve candidate-level failures as diagnostics,
    and use constructor injection. Do not change the established active
    `ArtifactSource` behavior or expose adapter types from application.
  - Verification: TASK-002-3 tests pass; the fake and identity use case satisfy
    public-item documentation, error, visibility, and layering checks.

- [ ] **TASK-002-5 (RED): Specify filesystem scope, archive discovery, and integrated acceptance behavior.**
  - Outcome: Add failing adapter and integrated tests for repository-wide
    identity discovery. Cover active artifacts, archived packet files under
    `specs/archive/`, superseded statuses, unique identities, duplicate IDs
    across active/history, all three path-encoded mismatch examples, template
    and supporting-file exclusion, empty identity sets, malformed frontmatter,
    unreadable files, unrelated-result retention, deterministic ordering,
    read-only bytes/statuses, and repeated discovery.
  - Dependencies: TASK-002-4.
  - Traceability: Every scenario in `specs/002-validate-artifact-identity/scenarios.feature`;
    `REQ-002` FR-001 through FR-008; `DES-002` filesystem and verification
    contracts.
  - Constraints: Use temporary local repositories and the real public
    filesystem/application APIs. Do not test through a CLI because CLI behavior
    and serialization are explicitly deferred to EPIC-004.
  - Verification: Focused adapter and acceptance tests fail for the expected
    absence of historical identity discovery or identity reporting, while the
    existing EPIC-001 active-only tests continue to identify their baseline.

- [ ] **TASK-002-6 (GREEN): Implement repository-wide filesystem identity discovery.**
  - Outcome: Extend the filesystem adapter to implement
    `ArtifactIdentitySource` while retaining active-only `ArtifactSource`
    discovery. Recognize the canonical active and archived packet paths, sort
    paths before reading, exclude templates/supporting files, read each eligible
    file once, map parser/read failures to candidates, and preserve normalized
    snapshots for domain identity evaluation.
  - Dependencies: TASK-002-5.
  - Traceability: `REQ-002` FR-001, FR-005, FR-006, and FR-008; scenarios for
    historical scope, excluded files, malformed frontmatter, empty input, and
    deterministic read-only validation; `ADR-005` and `ADR-006` boundaries.
  - Constraints: Keep adapter logic limited to path discovery, file access, and
    parser translation. Do not put duplicate or path-identity business rules in
    the adapter, change parser dependencies, execute artifact content, follow
    network links, write files, or persist reports.
  - Verification: TASK-002-5 adapter and acceptance tests pass; active source
    tests prove archives remain excluded from EPIC-001 validation; repository
    root/specs discovery failures remain typed terminal errors and per-file
    failures remain reportable diagnostics.

- [ ] **TASK-002-7 (REFACTOR AND TRACE): Complete the identity vertical slice and contract coverage.**
  - Outcome: Run the complete approved scenario set through the real filesystem
    source and application use case; add or refine the generic source contract
    suite for both the in-memory and filesystem identity-source implementations;
    preserve parser fuzz coverage and add an archive-layout seed only if needed;
    curate public exports, docs, ordering helpers, and test fixtures; remove
    duplication without changing approved behavior.
  - Dependencies: TASK-002-6.
  - Traceability: Every `REQ-002` `FR-*`, every scenario in
    `scenarios.feature`, `R-SDD-02`, `R-TST-14`, `R-TST-16`, and `R-TST-19`.
  - Constraints: Contract tests assert observable port behavior rather than
    implementation calls. Do not add CLI assertions, relationship checks,
    lifecycle promotion, persistence, or speculative abstractions.
  - Verification: All focused domain, application, adapter, contract,
    acceptance, and fuzz-target smoke checks pass; two identical validations
    return equal ordered reports; before/after repository snapshots prove no
    artifact content, lifecycle status, or persisted result changed; dependency
    direction remains `application -> domain` and adapter -> application/domain.

- [ ] **TASK-002-8 (VERIFY): Execute and record the complete quality-gate evidence.**
  - Outcome: Run the constitutional Rust quality suite and record observed output,
    test counts, coverage, dependency audit, and any environment limitation in
    this task document for `verify-feature`.
  - Dependencies: TASK-002-7.
  - Traceability: `REQ-002` quality requirements; `CONSTITUTION.md` R-AGT-07,
    R-SDD-02, R-TST-01, R-TST-14, R-TST-16, R-TST-17, R-TST-18, R-TST-19,
    R-DIR-01, R-SEP-02, R-TRT-01, R-TRT-20, R-DOC-01, and R-TOOL-04.
  - Verification: `cargo xtask ci` passes, reproducing `cargo fmt --all -- --check`,
    `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
    `cargo test --workspace --all-features`, rustdoc with
    `RUSTDOCFLAGS="-D warnings"`, `cargo deny check`, workspace and domain
    `cargo llvm-cov` thresholds, and release tests. Also run `cargo machete` and
    the available fuzz-target compile/smoke command, recording if the
    `cargo-fuzz` executable is unavailable.

## Test And Verification Plan

- [ ] Domain unit tests: Identity indexing, exact-ID collision grouping,
  path-encoded PRD/epic/ADR comparison, empty input, complete diagnostic
  retention, severity/rule fields, and stable ordering.
- [ ] Domain property tests: Repeated evaluation of equal snapshots produces
  equal ordered diagnostics; adding unrelated unique snapshots does not remove
  existing identity findings; identity evaluation remains pure and deterministic.
- [ ] Application integration tests: In-memory identity source, empty/valid/
  duplicate/mismatch/mixed candidates, malformed parser findings, candidate
  diagnostics, invalid empty candidates, and typed discovery failures.
- [ ] Port contract tests: One generic observable contract executed against the
  in-memory and filesystem identity-source implementations, including scope,
  ordering, candidate failure, and empty-set behavior.
- [ ] Filesystem integration tests: Active and archived packet discovery,
  superseded metadata, all exclusions, path/ID mismatches, malformed and
  unreadable files, full-result continuation, deterministic reruns, and
  read-only repository snapshots.
- [ ] Gherkin acceptance tests: All eight approved scenario headings, including
  all three examples of the path-identity scenario outline, pass through the
  real adapter and application path.
- [ ] Fuzz checks: Existing parser fuzz target still compiles and exercises
  untrusted artifact bytes; no parser dependency or unsafe code is added.
- [ ] Rust quality gates: `cargo xtask ci` and its constitutional component
  commands pass with workspace line coverage at least 85% and domain line
  coverage at least 95%; `cargo machete` reports no unused dependencies.
- [ ] CLI and serialization checks: Not applicable to this slice; explicitly
  deferred to EPIC-004 per US-002 and REQ-002.
- [ ] .NET and frontend gates: Not applicable; this feature changes only the
  Rust workspace and its specifications.

## Rollout And Recovery

### Rollout

- Add the identity domain behavior, application port/use case, and filesystem
  implementation as a library-only extension to the existing workspace.
- Keep existing active validation callers and outputs compatible; repository-wide
  identity discovery is opt-in through the new application use case.
- No database, file-format, lifecycle, or persisted-result migration is needed.
- No deployment coordination, network service, retry loop, or background job is
  needed. The validator reads a repository snapshot and returns a complete
  result or a typed repository-discovery error.

### Recovery

- Per-file read, parse, or recognition failures require no rollback: preserve
  them as path-based diagnostics and continue processing all other candidates.
- A repository-level discovery failure is terminal for that invocation; correct
  the repository access condition and rerun. Do not claim a partial success.
- Validation performs no mutation, so a partial or interrupted run needs no data
  cleanup. Rerunning against the same repository state is safe and should
  produce the same ordered result.
- If implementation verification fails, revert the identity implementation and
  its same-commit specification/task changes together. Do not alter archived
  artifact contents or statuses as recovery work.

## Definition Of Done

- [ ] All ordered tasks are complete with observed RED and GREEN evidence.
- [ ] Every `REQ-002` functional and quality requirement is covered by a
  traceable passing test.
- [ ] All approved Gherkin scenarios pass through the real adapter and
  application identity path.
- [ ] Active, archived, superseded, empty, duplicate, mismatch, excluded,
  malformed, unreadable, unrelated, deterministic, and read-only paths are
  verified.
- [ ] Every conflicting artifact receives an actionable duplicate diagnostic;
  every path-encoded mismatch remains represented and diagnosable.
- [ ] Malformed or unrecognized frontmatter retains the EPIC-001 structural
  diagnostic and is not reparsed for identity.
- [ ] The active-only `ArtifactSource` behavior remains unchanged, and the new
  identity port has a documented in-memory test double and contract coverage.
- [ ] Domain purity, dependency direction, public documentation, typed errors,
  and no-unsafe/no-new-dependency constraints pass review.
- [ ] `cargo xtask ci`, coverage thresholds, dependency audit, unused-
  dependency analysis, release tests, and available fuzz smoke checks pass with
  observed output recorded for `verify-feature`.
- [ ] No CLI, serialization, persistence, lifecycle, schema, relationship, or
  release behavior is implemented by this feature.
- [ ] Relevant specifications and ADRs remain current in the implementation
  change set.

---
id: REQ-009
title: "Guarded completion transition requirements"
type: feature-requirements
status: implemented
created: 2026-09-13
updated: 2026-09-13
promoted_from: in-review
promoted_to: approved
promoted_by: Project owner
promoted_at: 1789325867
owner: TBD
parent: US-009
depends_on: [REQ-008]
requires: [US-009, EPIC-003, REQ-008, REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-007, ADR-002]
blockers: []
related: [PRD-001, EPIC-003, ADR-002]
approval:
  approved_by: Project owner
  approved_on: 2026-09-13
---

# Requirements

## Purpose And Actors

### Purpose

Define the externally observable contract for safely promoting one eligible
artifact, release record, or relocated feature packet through one guarded
completion transition. The capability prevents invalid closure while preserving
`record-release` as the owner of release-record creation and packet relocation.

### Actors And External Systems

- An LLM coding agent requesting a lifecycle completion transition.
- A human reviewer requesting or inspecting a completion transition.
- The local repository's recognized artifacts, release records, and feature
  packet paths.
- Existing lifecycle, identity, relationship, verification-evidence, and
  release-record validation rules.

The capability does not require network access, hosted services, or AI judgment.
It does not create release records or move files.

## Preconditions

- The target exists and is a recognized artifact, release record, or feature
  packet eligible for lifecycle management.
- The request identifies exactly one target and exactly one completion state.
- The target's current lifecycle state is available to the completion guard.
- Supersession, release, and archival prerequisite content is available in the
  local repository when the requested transition requires it.
- For archival, `record-release` has already relocated the feature packet under
  `specs/archive/` and produced the corresponding release evidence.

## Inputs And Outputs

| Interaction | Inputs | Outputs | Validation |
| --- | --- | --- | --- |
| Request one completion transition | One target reference and one target state | A successful promotion result identifying the target and resulting state, or a rejected result containing all applicable diagnostics | The target and transition must satisfy the supported completion-transition rules |
| Request supersession | One approved artifact and `superseded` as the target state | The predecessor becomes `superseded` when its approved successor and reciprocal links are valid | The successor must be approved and both supersession directions must be valid |
| Request release | One approved release record and `released` as the target state | The release record becomes `released` when its included features and release evidence are valid | Every included feature must be implemented or archived; verification evidence and a release commit are required |
| Request archival | One implemented feature packet and `archived` as the target state | The relocated packet becomes `archived` without a file move | The packet must already be under `specs/archive/` and its corresponding release record must be `released` |

## Functional Requirements

| ID | Requirement | Priority | Traceability (Story & Scenario) |
| --- | --- | --- | --- |
| FR-001 | The capability shall accept exactly one target and one requested completion state per request. | Must | US-009 / Acceptance Criteria; Scenario: Boundary: One request cannot batch completion transitions |
| FR-002 | The capability shall support exactly `approved` to `superseded`, `approved` to `released` for release records, and `implemented` to `archived` for feature packets. | Must | US-009 / Business Rules; Scenarios: Happy: An approved artifact is superseded by a valid successor, Happy: A complete release record becomes released, Happy: A relocated feature packet becomes archived |
| FR-003 | An approved artifact shall become `superseded` only when an approved successor exists and the predecessor and successor contain valid reciprocal `supersedes` and `superseded_by` links. | Must | US-009 / Acceptance Criteria; Scenarios: Happy: An approved artifact is superseded by a valid successor, Failure: Invalid supersession prerequisites are aggregated |
| FR-004 | A successful supersession shall preserve the predecessor at its existing path and leave the approved successor unchanged. | Must | US-009 / Business Rules; Scenario: Happy: An approved artifact is superseded by a valid successor |
| FR-005 | An approved release record shall become `released` only when every included feature is implemented or archived, verification evidence is recorded, and a release commit is recorded. | Must | US-009 / Acceptance Criteria; Scenarios: Happy: A complete release record becomes released, Failure: An incomplete release is rejected without mutation |
| FR-006 | An implemented feature packet shall become `archived` only when it already resides under `specs/archive/` and its corresponding validated release record is `released`. | Must | US-009 / Acceptance Criteria; Scenarios: Happy: A relocated feature packet becomes archived, Failure: A packet lacking an archival prerequisite cannot be archived |
| FR-007 | The capability shall not create release records, relocate files, or change included features as part of a completion transition. | Must | US-009 / Scope Boundaries; Scenarios: Happy: A relocated feature packet becomes archived, Failure: A packet lacking an archival prerequisite cannot be archived |
| FR-008 | The capability shall reject unsupported, skipped, backward, and batched completion transitions, and shall reject any completion request for a target already in `superseded`, `released`, or `archived`. | Must | US-009 / Acceptance Criteria; Scenarios: Boundary: Terminal completion states are immutable, Boundary: Skipped and backward completion transitions are rejected, Boundary: One request cannot batch completion transitions |
| FR-009 | A rejected completion request shall leave the target lifecycle state, promotion metadata, and repository content unchanged. | Must | US-009 / Acceptance Criteria; Scenarios: Failure: Invalid supersession prerequisites are aggregated, Failure: An incomplete release is rejected without mutation, Failure: A packet lacking an archival prerequisite cannot be archived |
| FR-010 | A rejected completion request shall return every applicable prerequisite diagnostic rather than stopping at the first failure. | Must | US-009 / Business Rules; Scenarios: Failure: Invalid supersession prerequisites are aggregated, Failure: An incomplete release is rejected without mutation |
| FR-011 | Completion diagnostics shall be actionable and ordered deterministically for identical target and repository inputs. | Must | US-009 / Acceptance Criteria; Scenarios: Failure: Invalid supersession prerequisites are aggregated, Failure: An incomplete release is rejected without mutation, Boundary: Repeated failed evaluation is deterministic and offline |
| FR-012 | Repeating an unchanged failed completion request shall produce the same rejection decision and identically ordered diagnostics. | Must | US-009 / Acceptance Criteria; Scenario: Boundary: Repeated failed evaluation is deterministic and offline |
| FR-013 | Completion handling shall operate offline and shall not require AI judgment. | Must | US-009 / Business Rules; Scenario: Boundary: Repeated failed evaluation is deterministic and offline |

## Postconditions And Invariants

- A successful supersession leaves the predecessor at its original path with
  lifecycle state `superseded` and leaves the approved successor unchanged.
- A successful release changes only the validated release record to `released`;
  included features retain their existing states.
- A successful archival transition changes the relocated feature packet's
  lifecycle state to `archived` without moving files.
- A rejected request changes no lifecycle state, promotion metadata, or
  repository content.
- A completion request never applies more than one target transition.
- Terminal completion states remain immutable.
- The same unchanged failed request has the same classification and diagnostic
  ordering on every evaluation.
- Release-record creation, evidence compilation, and packet relocation remain
  outside this capability.

## Edge And Failure Behavior

| Condition | Expected behavior | User-visible result |
| --- | --- | --- |
| The request contains multiple targets or transition types | Reject before changing any target | Actionable diagnostic identifying the single-target, single-transition requirement |
| The target is missing or unrecognized | Reject without mutation | Target-resolution diagnostic |
| The requested state is unsupported, skipped, or backward | Reject without mutation | Invalid-transition diagnostic |
| An approved artifact lacks an approved successor | Continue checking applicable supersession prerequisites | Not-promoted result with a missing-successor diagnostic |
| Supersession links are missing, invalid, or non-reciprocal | Continue checking applicable supersession prerequisites | Not-promoted result with link diagnostics for each applicable failure |
| A release includes an unfinished feature | Continue checking release prerequisites | Not-promoted result identifying the unfinished feature |
| A release lacks verification evidence or a release commit | Continue checking release prerequisites | Not-promoted result identifying every missing evidence condition |
| A feature packet is not already under `specs/archive/` | Do not move it and reject archival | Not-promoted result identifying the relocation prerequisite |
| The corresponding release record is not `released` or is invalid | Reject archival without mutation | Not-promoted result identifying the release prerequisite |
| The target is already `superseded`, `released`, or `archived` | Reject as immutable | Terminal-state diagnostic and unchanged target |
| Several prerequisites fail together | Retain all applicable findings | Deterministically ordered aggregate diagnostics |
| The same failed request is repeated without repository changes | Re-evaluate without mutation | Identical rejection and diagnostic order |
| Network or AI judgment would be required | Do not access the network or infer a decision | Deterministic local result based on repository evidence only |

## Quality Requirements

- **Determinism:** Identical target, transition, and repository content shall
  produce identical classifications and diagnostic ordering.
- **Completeness:** A failed request shall retain every applicable prerequisite
  failure from that evaluation.
- **Atomic failure safety:** No failed request may change status, promotion
  metadata, file paths, or unrelated repository content.
- **Single-target scope:** One request shall never partially process a batch of
  targets or mixed transitions.
- **Offline operation:** Completion handling shall require no network, hosted
  service, or AI service.
- **Actionability:** Diagnostics shall identify the target and unmet guard well
  enough for an agent or reviewer to correct the prerequisite.
- **Compatibility:** Existing valid lifecycle, identity, relationship,
  verification-evidence, readiness, and release-record rules remain enforced.

## Dependencies And Deferred Decisions

- Depends on `REQ-008` readiness behavior, `REQ-007` packet promotion behavior,
  existing structural and relationship requirements, and lifecycle policy in
  `ADR-002`.
- `record-release` remains responsible for creating release records, compiling
  release evidence, and relocating feature packets before archival promotion.
- CLI syntax, exit-code taxonomy, and machine-readable serialization remain
  deferred to EPIC-004.
- The exact presentation format of human-readable and machine-readable
  diagnostics remains governed by the existing promotion contract and is not
  redefined here.

## Traceability

- Source story: `US-009`
- Executable scenarios: `scenarios.feature`
- Parent epic: `EPIC-003`
- Parent PRD: `PRD-001`
- Related decision: `ADR-002`

Every functional requirement maps to one or more approved US-009 scenarios, and
every approved scenario is covered by at least one functional requirement.

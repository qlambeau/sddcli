---
id: REQ-006
title: "Single-artifact lifecycle promotion requirements"
type: feature-requirements
status: approved
created: 2026-09-07
updated: 2026-09-07
owner: TBD
parent: US-006
depends_on: [REQ-005]
requires: [US-006, EPIC-003, REQ-001, REQ-005, ADR-002]
blockers: []
related: [PRD-001, EPIC-001, EPIC-002, ADR-002]
approval:
  approved_by: Project owner
  approved_on: 2026-09-07
---

# Requirements

## Purpose And Actors

### Purpose

Define the observable contract for safely promoting one recognized SDD artifact
through its permitted lifecycle while preserving validation gates, artifact
content, and the latest promotion metadata.

### Actors And External Systems

- LLM coding agent requesting a valid lifecycle transition.
- Human reviewer confirming approval through the surrounding workflow.
- The current repository and its local `specs/` tree as the promotion source
  and persistence boundary.
- The earlier artifact-validation and relationship-integrity capabilities.

The promotion capability records the supplied actor identity. It does not
classify the actor or determine whether a human approval decision is genuine.

## Preconditions

- The target is one recognized artifact at a canonical repository location.
- The repository and the target artifact can be read and, for a successful
  state change, safely updated.
- The requested target state is a recognized lifecycle state.
- A state-changing request identifies a non-empty actor identity.
- For `in-review -> approved`, the surrounding workflow has already confirmed
  human approval.
- For `approved -> implemented`, required implementation and verification
  evidence is available.
- For `implemented -> archived`, the artifact is already in its canonical
  archive location and the closed release conditions are satisfied.
- For `approved -> superseded`, an approved successor and valid supersession
  links are available.
- Existing artifact validation and relationship-integrity findings are
  available for applicable prerequisite checks.

## Inputs And Outputs

| Interaction | Inputs | Outputs | Validation |
| --- | --- | --- | --- |
| Request one-artifact promotion | Single-artifact reference, requested target state, actor identity, repository state, and applicable external workflow confirmation | A success result, an active-artifact idempotent no-op result, or a failure result with ordered diagnostics | The artifact is recognized, the target state is valid, the transition is permitted, and all applicable prerequisites pass before mutation |
| Inspect the promotion result | Result of the requested transition and the resulting artifact state | Current artifact state, latest promotion metadata when a real transition succeeds, and applicable diagnostics | A failed request exposes all applicable failures; a same-state active request exposes no mutation |

A successful real transition exposes one latest promotion record containing the
source state, target state, actor identity, and promotion timestamp. A later
real transition replaces that record. A same-state request does not create or
replace promotion metadata.

## Functional Requirements

| ID | Requirement | Priority | Traceability (Story & Scenario) |
| --- | --- | --- | --- |
| FR-001 | The capability shall accept one-artifact promotion requests for PRDs, epic briefs, user stories, Gherkin scenarios, requirements, designs, tasks, and ADRs at recognized canonical locations. Gherkin lifecycle state shall use its canonical status header. | Must | US-006 / Scenario: A recognized active artifact enters review |
| FR-002 | The capability shall permit only the strict forward transitions `draft -> in-review`, `in-review -> approved`, `approved -> implemented`, and `implemented -> archived`, plus the guarded `approved -> superseded` transition where applicable. | Must | US-006 / Scenario Outline: An invalid lifecycle transition is rejected; Scenarios: A recognized active artifact enters review; An artifact under review is approved after external human confirmation; An approved artifact becomes implemented after verification; An implemented artifact is archived from its canonical archive location; An approved artifact becomes superseded by an approved successor |
| FR-003 | The capability shall reject promotion requests for archived, released, and already-superseded artifacts, except that a request targeting the current state of an active artifact is handled as an idempotent no-op. | Must | US-006 / Scenario Outline: An invalid lifecycle transition is rejected; Scenario: A same-state promotion request is idempotent |
| FR-004 | The capability shall allow an active artifact already in the requested state to return a successful idempotent no-op without changing content, lifecycle state, or promotion metadata. | Must | US-006 / Scenario: A same-state promotion request is idempotent |
| FR-005 | Before permitting `draft -> in-review`, the capability shall verify the artifact's applicable review-entry checks. | Must | US-006 / Scenario Outline: A recognized active artifact enters review |
| FR-006 | Before permitting `in-review -> approved`, the capability shall verify all applicable approval checks and require that human approval has been confirmed by the surrounding workflow; it shall record the supplied actor without classifying that actor or judging the approval's authenticity. | Must | US-006 / Scenario: An artifact under review is approved after external human confirmation |
| FR-007 | Before permitting `approved -> implemented`, the capability shall verify that the required implementation and verification evidence is recorded. | Must | US-006 / Scenarios: An approved artifact becomes implemented after verification; Missing implementation evidence blocks promotion |
| FR-008 | Before permitting `implemented -> archived`, the capability shall verify the canonical archive location and closed release conditions, and shall not move the artifact or any other file. | Must | US-006 / Scenarios: An implemented artifact is archived from its canonical archive location; Archival is rejected when the artifact has not been relocated |
| FR-009 | Before permitting `approved -> superseded`, the capability shall verify that an approved successor exists and that the required supersession links are valid. | Must | US-006 / Scenarios: An approved artifact becomes superseded by an approved successor; Supersession is rejected without an approved successor and valid links |
| FR-010 | For each successful real transition, the capability shall record the source state, target state, supplied actor identity, and promotion timestamp as the artifact's latest promotion metadata. | Must | US-006 / Scenarios: A recognized active artifact enters review; An artifact under review is approved after external human confirmation; An approved artifact becomes implemented after verification; An implemented artifact is archived from its canonical archive location; An approved artifact becomes superseded by an approved successor |
| FR-011 | The capability shall validate every applicable transition, prerequisite, blocker, evidence, location, and relationship condition before applying a state-changing request. If any applicable condition fails, it shall apply no mutation to the artifact. | Must | US-006 / Scenarios: Invalid lifecycle transition is rejected; Archival is rejected when the artifact has not been relocated; Supersession is rejected without an approved successor and valid links; Missing implementation evidence blocks promotion; Multiple applicable failures are reported without mutation |
| FR-012 | A failed request shall leave the target artifact's lifecycle state, content, and promotion metadata unchanged, and shall not modify unrelated artifacts or files. | Must | US-006 / Scenarios: Invalid lifecycle transition is rejected; Archival is rejected when the artifact has not been relocated; Supersession is rejected without an approved successor and valid links; Missing implementation evidence blocks promotion; Multiple applicable failures are reported without mutation; A successful promotion changes only the requested artifact state and metadata |
| FR-013 | The capability shall report every applicable failure as an actionable diagnostic, retain deterministic diagnostic ordering, and continue evaluation after an individual failure. | Must | US-006 / Scenarios: Multiple applicable failures are reported without mutation; Repeated failed promotion requests are deterministic and read-only |
| FR-014 | A successful real promotion shall change only the requested artifact's lifecycle state and latest promotion metadata; artifact body content, unrelated artifacts, and unrelated files shall remain unchanged. | Must | US-006 / Scenario: A successful promotion changes only the requested artifact state and metadata |
| FR-015 | The capability shall perform no network access, hosted-service interaction, or AI judgment, and release records and their `released` transition shall remain outside this contract. | Must | US-006 / Scenarios: An artifact under review is approved after external human confirmation; A successful promotion changes only the requested artifact state and metadata; Repeated failed promotion requests are deterministic and read-only |

## Postconditions And Invariants

- A successful real transition leaves the target artifact in the requested
  state.
- A successful real transition has exactly one latest promotion record with the
  source state, target state, actor identity, and promotion timestamp.
- A successful same-state request for an active artifact leaves the artifact
  byte-equivalent with respect to content, state, and promotion metadata and
  reports an idempotent no-op.
- A failed request leaves the target artifact's state, content, and promotion
  metadata unchanged.
- A failed request does not move or modify unrelated artifacts or files.
- A transition never skips a permitted normal state or moves backward.
- Archived, released, and superseded artifacts cannot be reopened through this
  capability.
- Promotion never relocates an artifact into or out of the archive.
- The result for identical repository state and request inputs is identical and
  ordered identically.
- No network, AI service, or persisted validation result is required for a
  promotion decision.

## Edge And Failure Behavior

| Condition | Expected behavior | User-visible result |
| --- | --- | --- |
| The artifact type or location is not recognized | Reject the request without mutation | Actionable recognition diagnostic |
| The requested target is skipped, backward, or otherwise not allowed from the current state | Reject the request without mutation | Invalid-transition diagnostic |
| The artifact is archived, released, or already superseded and a state-changing request is made | Reject the request without mutation | Terminal-state diagnostic |
| An active artifact is already in the requested state | Return success without mutation | Idempotent no-op result with no new promotion metadata |
| Review-entry checks fail | Reject `draft -> in-review` without mutation | Review-entry diagnostics |
| Approval checks fail or external human confirmation is absent | Reject `in-review -> approved` without mutation | Approval diagnostics; actor classification is not attempted |
| Required implementation or verification evidence is missing | Reject `approved -> implemented` without mutation | Evidence diagnostic |
| The implemented artifact is not in its canonical archive location | Reject `implemented -> archived` without mutation | Archive-location diagnostic and no file move |
| Closed release conditions are absent | Reject `implemented -> archived` without mutation | Release-condition diagnostic |
| No approved successor or valid supersession links exist | Reject `approved -> superseded` without mutation | Supersession-prerequisite diagnostic |
| Multiple applicable conditions fail | Continue evaluation and report each applicable failure | Complete, deterministically ordered diagnostics and no mutation |
| The same failed request is repeated against unchanged repository state | Produce the same result without mutation | Identical diagnostics in identical order |
| A successful request is made | Change only the requested state and latest promotion metadata | Success result; unrelated content remains unchanged |

## Quality Requirements

- Promotion decisions are deterministic for identical repository state and
  request inputs.
- Failed requests are atomic and do not partially update status, content, or
  metadata.
- Promotion is read/write only within the requested artifact's permitted
  lifecycle metadata and does not move files.
- The capability reports all applicable diagnostics rather than stopping at the
  first failure.
- Diagnostic ordering is stable across repeated evaluations.
- Diagnostics identify the failed condition and provide enough context for an
  agent or reviewer to correct the artifact or prerequisite.
- Promotion requires no network, hosted service, or AI dependency.
- The capability does not determine whether an approval decision is genuinely
  human.
- CLI syntax, exit-code taxonomy, and machine-readable serialization remain
  deferred to EPIC-004.

## Dependencies And Deferred Decisions

- Source story: approved `US-006`.
- Parent epic: approved `EPIC-003`.
- Parent PRD: approved `PRD-001`.
- Existing validation and relationship-integrity contracts: `REQ-001` and
  `REQ-005`, with the earlier EPIC-002 requirements transitively included.
- Strict promote-only lifecycle policy: approved `ADR-002`.
- CLI syntax, stable exit codes, and machine-readable output are deferred to
  EPIC-004.
- Release-record recognition and the `released` transition are deferred.
- Human semantic approval remains an external workflow responsibility.

## Traceability

- Source story: `US-006`
- Parent epic: `EPIC-003`
- Parent PRD: `PRD-001`
- Executable scenarios: `scenarios.feature`
- Covered scenarios: A recognized active artifact enters review; An artifact
  under review is approved after external human confirmation; An approved
  artifact becomes implemented after verification; An implemented artifact is
  archived from its canonical archive location; An approved artifact becomes
  superseded by an approved successor; A same-state promotion request is
  idempotent; An invalid lifecycle transition is rejected; Archival is rejected
  when the artifact has not been relocated; Supersession is rejected without an
  approved successor and valid links; Missing implementation evidence blocks
  promotion; Multiple applicable failures are reported without mutation; A
  successful promotion changes only the requested artifact state and metadata;
  Repeated failed promotion requests are deterministic and read-only.

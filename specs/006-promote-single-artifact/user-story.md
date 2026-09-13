---
id: US-006
title: "Promote a single SDD artifact through its lifecycle"
type: user-story
status: approved
created: 2026-09-07
updated: 2026-09-07
owner: TBD
parent: PRD-001
epic: EPIC-003
feature: 006-promote-single-artifact
depends_on: []
requires: [EPIC-003, EPIC-001, EPIC-002]
blockers: []
related: [ADR-002]
approval:
  approved_by: Project owner
  approved_on: 2026-09-07
---

# User Story

## Story Card

As an LLM coding agent or human reviewer,
I want to request a valid lifecycle transition for one recognized SDD artifact,
so that work advances safely without bypassing review, evidence, or lifecycle
rules.

## Context And Value

The SDD workflow requires `promote-artifact` to be the sole authority for
lifecycle changes. A single-artifact promotion capability gives agents and
reviewers a safe way to advance one artifact while preserving prerequisites,
review evidence, audit metadata, and the artifact's existing content.

## Business Rules

- The story applies to all currently recognized specification types: PRDs,
  epic briefs, user stories, Gherkin scenarios, requirements, designs, tasks,
  and ADRs.
- The normal lifecycle is strictly forward:
  `draft -> in-review -> approved -> implemented -> archived`.
- An `approved -> superseded` transition is allowed only when an approved
  successor exists and the required supersession links are valid.
- Release-record promotion is outside this story.
- Promotion is available only for active artifacts. Archived, released, and
  already-superseded artifacts cannot be promoted.
- An `implemented -> archived` transition requires the artifact to already be
  in its canonical archive location; promotion does not move files.
- An `approved -> implemented` transition requires recorded verification
  evidence.
- An `in-review -> approved` transition requires that the surrounding workflow
  has confirmed human approval and that all applicable approval checks pass.
  The promotion capability records the supplied actor identity but does not
  classify the actor or determine whether approval is genuinely human.
- A request whose target equals the current state succeeds idempotently without
  changing the artifact or creating duplicate promotion metadata.
- A failed request changes neither the artifact status nor its promotion
  metadata and reports all applicable diagnostics.
- Every successful state change records the source state, target state, actor,
  and promotion timestamp.
- Human confirmation for approval is a workflow responsibility external to the
  promotion capability.

## Examples

| Example | Given | When | Expected outcome |
| --- | --- | --- | --- |
| EX-001 | A recognized artifact is in `draft` and satisfies review-entry checks | The agent requests `in-review` with actor information | The transition succeeds and promotion metadata is recorded. |
| EX-002 | A recognized artifact is in `in-review` and approval checks pass | A human reviewer explicitly requests `approved` | The transition succeeds and records the reviewer and transition metadata. |
| EX-003 | An artifact is in `approved` but verification evidence is absent | The agent requests `implemented` | The request fails, the artifact is unchanged, and the missing evidence is reported. |
| EX-004 | An artifact is in `implemented` but is not in the canonical archive location | The agent requests `archived` | The request fails without moving or changing the artifact. |
| EX-005 | An approved artifact has an approved successor and valid supersession links | The agent requests `superseded` | The transition succeeds and records the transition metadata. |
| EX-006 | An artifact is already in the requested state | The agent repeats a request for that same state | The request succeeds idempotently without changing content or metadata. |
| EX-007 | An artifact is in a state that cannot move backward or skip states | The agent requests an invalid target state | The request fails, leaves the artifact unchanged, and reports the invalid transition. |
| EX-008 | Multiple transition, prerequisite, blocker, or relationship issues apply | The agent requests a transition | The request reports all applicable diagnostics and applies no mutation. |

## Acceptance Criteria

- The capability accepts promotion requests for every currently recognized
  specification type.
- Valid normal transitions follow the strict forward lifecycle and invalid,
  skipped, backward, or terminal-state transitions are rejected.
- Supersession is accepted only when its approved successor and required links
  satisfy the applicable checks.
- Implementation and archival transitions enforce their required verification
  evidence and canonical-location conditions.
- Same-state requests are successful idempotent no-ops.
- Every successful state change records source state, target state, actor, and
  promotion timestamp.
- Failed requests leave status, content, and promotion metadata unchanged.
- Failed requests report all applicable diagnostics in deterministic order.
- Promotion does not move files, modify unrelated artifacts, access the
  network, or require AI judgment.
- Release records and their `released` transition are not handled by this
  story.

## Scope Boundaries

### In Scope

- Safe promotion of one recognized specification artifact.
- Normal lifecycle transitions through archival.
- Guarded supersession for one artifact.
- Transition prerequisites, blockers, verification evidence, archive-location
  checks, idempotency, diagnostics, and promotion metadata.

### Out Of Scope

- Atomic promotion of a complete feature packet.
- Release-record creation, validation, or promotion to `released`.
- Moving artifacts into `specs/archive/`.
- CLI user experience, exit-code taxonomy, and machine-readable serialization.
- Human semantic judgment or proof that an approval decision is genuine.
- Artifact authoring, ID allocation, and relationship-integrity rules delivered
  by earlier epics.

## Dependencies

- `EPIC-003` must be approved.
- `EPIC-001` and `EPIC-002` validation and relationship-integrity capabilities
  are available.
- `ADR-002` defines strict promote-only status transitions.

## Open Questions

| ID | Question | Blocking? | Owner | Status |
| --- | --- | --- | --- | --- |
| OQ-001 | What stable exit-code taxonomy and machine-readable output schema should downstream agents consume? | No | EPIC-004 | Deferred |

## INVEST Check

- [x] Independent
- [x] Negotiable
- [x] Valuable
- [x] Estimable
- [x] Small enough for roughly 1 to 3 days
- [x] Testable

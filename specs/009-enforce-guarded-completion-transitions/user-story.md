---
id: US-009
title: "Enforce guarded completion transitions"
type: user-story
status: implemented
created: 2026-09-13
updated: 2026-09-13
promoted_from: in-review
promoted_to: approved
promoted_by: Project owner
promoted_at: 1789325293
owner: TBD
parent: PRD-001
epic: EPIC-003
feature: 009-enforce-guarded-completion-transitions
depends_on: [US-008]
requires: [EPIC-003, EPIC-001, EPIC-002, US-008]
blockers: []
related: [ADR-002]
approval:
  approved_by: Project owner
  approved_on: 2026-09-13
---

# User Story

## Story Card

As an LLM coding agent or human reviewer,
I want completion transitions to enforce their prerequisites one transition at a time,
so that artifacts, releases, and feature packets cannot be incorrectly closed.

## Context And Value

The lifecycle includes completion states with different evidence and relationship
requirements. Without explicit guards, an artifact could be superseded without a
valid successor, a release could be closed while included features remain
unfinished, or a feature packet could be archived without a completed release.
A deterministic completion check protects the lifecycle while preserving the
separate responsibility of `record-release` to create release records and move
completed packets.

## Business Rules

- Each request targets exactly one artifact or release record and one completion
  transition.
- The supported completion transitions are exactly `approved` to `superseded`,
  `approved` to `released` for release records, and `implemented` to `archived`
  for feature packets.
- Superseding an artifact requires an approved successor and valid reciprocal
  `supersedes` and `superseded_by` links. The predecessor remains in place as
  an immutable superseded artifact.
- Releasing a release record requires every included feature to be implemented
  or archived, recorded verification evidence, and a release commit.
- Archiving a feature packet requires the packet to already be under
  `specs/archive/` and a corresponding validated release record in `released`.
- Completion promotion does not create release records or move files; those
  responsibilities remain with the release workflow.
- No guarded completion request may skip states, move backward, or combine
  multiple targets or transition types.
- An artifact already in a terminal completion state is immutable; another
  completion request for it is rejected rather than treated as a no-op.
- A failed request leaves the target unchanged and returns every applicable
  prerequisite diagnostic in deterministic order.
- Completion decisions are deterministic, offline, and do not require AI
  judgment.

## Examples

| Example | Given | When | Expected outcome |
| --- | --- | --- | --- |
| EX-001 | An approved artifact has an approved successor with reciprocal supersession links | A reviewer requests supersession | The predecessor becomes `superseded` and remains at its existing path |
| EX-002 | An approved artifact has no valid successor or has a one-way supersession link | An agent requests supersession | The request is rejected, the artifact is unchanged, and all applicable diagnostics are returned |
| EX-003 | A release record includes only implemented or archived features, has verification evidence, and records a release commit | A reviewer requests release | The release record becomes `released` |
| EX-004 | A release record includes an unfinished feature or lacks required evidence | An agent requests release | The request is rejected with all applicable diagnostics and no status changes |
| EX-005 | An implemented feature packet is already under `specs/archive/` and has a validated `released` record | A reviewer requests archival | The packet becomes `archived` without moving files |
| EX-006 | An implemented feature packet is still active or has no released record | An agent requests archival | The request is rejected and the packet remains unchanged |
| EX-007 | An artifact is already `superseded`, `released`, or `archived` | Any further completion request is made | The request is rejected as an immutable terminal-state operation |

## Acceptance Criteria

- A request accepts exactly one eligible target and one completion transition.
- `approved` artifacts can become `superseded` only when an approved successor
  exists and supersession links are reciprocal and valid.
- An approved release record can become `released` only when every included
  feature is implemented or archived and the record contains verification
  evidence and a release commit.
- An implemented feature packet can become `archived` only after
  `record-release` has relocated it under `specs/archive/` and a corresponding
  validated release record is `released`.
- Completion promotion never creates release records or moves files.
- Unsupported, skipped, backward, batched, and terminal-state transitions are
  rejected with actionable diagnostics.
- Failed completion requests do not change lifecycle status or promotion
  metadata.
- Failed requests return all applicable diagnostics in deterministic order.
- Repeated evaluation of the same unchanged target produces the same decision
  and diagnostics.
- Completion handling operates offline and does not require AI judgment.

## Scope Boundaries

### In Scope

- Guarded supersession of approved artifacts.
- Guarded release of validated release records.
- Guarded archival of already-relocated implemented feature packets.
- Deterministic aggregate diagnostics and no-mutation-on-failure behavior.
- Enforcement that each request handles one target and one completion transition.

### Out Of Scope

- Creating or compiling release records.
- Moving feature packets or other files into `specs/archive/`.
- Authoring successors, release evidence, or verification records.
- Batch completion operations.
- CLI syntax, exit-code taxonomy, and machine-readable serialization.
- Human semantic judgment or proof that approval is genuine.
- Network access or hosted-service integration.

## Dependencies

- `US-008` Spec-Ready evaluation and `US-007` implementation-packet behavior are
  available.
- `EPIC-001` structural validation and `EPIC-002` identity and relationship
  integrity are available.
- `EPIC-003` lifecycle rules and `ADR-002` strict promote-only transitions
  define the completion lifecycle.
- `record-release` remains responsible for release-record creation, evidence
  compilation, and packet relocation before archival promotion.

## Open Questions

| ID | Question | Blocking? | Owner | Status |
| --- | --- | --- | --- | --- |
| — | No unresolved blocking questions remain. | No | Project owner | Resolved |

## INVEST Check

- [x] Independent
- [x] Negotiable
- [x] Valuable
- [x] Estimable
- [x] Small enough for roughly 1 to 3 days
- [x] Testable

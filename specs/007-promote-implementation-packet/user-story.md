---
id: US-007
title: "Promote a complete implementation packet one lifecycle step atomically"
type: user-story
status: implemented
created: 2026-09-13
updated: 2026-09-13
promoted_from: in-review
promoted_to: approved
promoted_by: Project owner
promoted_at: 1789312551
owner: TBD
parent: PRD-001
epic: EPIC-003
feature: 007-promote-implementation-packet
depends_on: [US-006]
requires: [EPIC-003, EPIC-001, EPIC-002, US-006]
blockers: []
related: [ADR-002, ADR-007]
approval:
  approved_by: Project owner
  approved_on: 2026-09-13
---

# User Story

## Story Card

As an LLM coding agent or human reviewer,
I want to request one atomic promotion step for a complete implementation packet
and its implementation-packet supporting artifacts,
so that packet lifecycle progress is consistent and never leaves partial status
changes.

## Context And Value

A single-artifact promotion capability protects one lifecycle change at a time,
but implementation packets are reviewed and advanced as coherent units. Agents
need a packet-level promotion request that discovers the packet's relevant
artifacts, determines each artifact's next valid lifecycle step, and applies all
eligible mutations atomically. This prevents a packet from being left in a mixed
or misleading state when one artifact fails a prerequisite.

## Business Rules

- The request is "promote this implementation packet one step"; callers do not
  supply a single packet-level target state.
- The capability discovers included implementation-packet artifacts from the
  selected packet:
  - colocated packet files: `user-story.md`, `scenarios.feature`,
    `requirements.md`, `design.md`, and `tasks.md`;
  - directly referenced implementation-packet supporting artifacts such as ADRs
    and schemas;
  - recursively resolved `requires`, `depends_on`, and `related` artifacts only
    when they are part of the implementation packet context.
- Parent PRDs and epic briefs are not generally included unless they are
  implementation-packet supporting artifacts.
- Each included artifact that can advance moves to its own next valid lifecycle
  state.
- Included artifacts that cannot advance one next valid state are skipped,
  reported as skipped, and left unchanged without producing an error.
- Per-artifact confirmations and prerequisite facts are supplied by artifact ID
  or artifact type, not by filesystem path.
- If any artifact selected for advancement has failed prerequisites, unresolved
  blockers, invalid relationships, missing required confirmation, missing
  evidence, or an operational write conflict, the entire packet promotion fails
  without mutating any artifact.
- Atomicity applies to all advancing artifacts: either every eligible advancing
  artifact is updated, or no artifact is updated.
- Successful mutations record the same promotion metadata required for
  single-artifact promotion: source state, target state, actor, and promotion
  timestamp.
- Skipped artifacts are included in the result as skipped participants and do
  not receive new promotion metadata.
- The capability does not move artifacts, create release records, archive
  packets, access the network, or perform AI judgment.

## Examples

| Example | Given | When | Expected outcome |
| --- | --- | --- | --- |
| EX-001 | A packet contains draft story, scenarios, requirements, design, and tasks that all satisfy review-entry checks | The agent requests one packet promotion step | All five artifacts advance to `in-review` and promotion metadata is recorded for each. |
| EX-002 | A packet contains a mix of `draft` and `in-review` artifacts with required per-artifact confirmations present | The reviewer requests one packet promotion step | Draft artifacts advance to `in-review`, in-review artifacts advance to `approved`, and all mutations are committed atomically. |
| EX-003 | A packet references an implementation-packet ADR through `related` and that ADR can advance | The agent requests one packet promotion step | The ADR is included and advances one valid lifecycle step with the packet. |
| EX-004 | A recursively related artifact is outside the implementation packet context | The agent requests one packet promotion step | The artifact is not included in the promotion set. |
| EX-005 | An included artifact is already in a state that cannot advance one next valid step | The agent requests one packet promotion step | The artifact is reported as skipped, remains unchanged, and does not block other eligible artifacts. |
| EX-006 | One eligible artifact is missing its required per-artifact human approval confirmation | The reviewer requests one packet promotion step | The entire packet promotion fails, all artifacts remain unchanged, and the missing confirmation is reported. |
| EX-007 | Multiple eligible artifacts have prerequisite, blocker, and relationship failures | The agent requests one packet promotion step | The request reports all applicable diagnostics in deterministic order and applies no mutation. |
| EX-008 | All advancement decisions pass, but one artifact source changes before commit | The agent requests one packet promotion step | The request fails with a conflict, no artifact is updated, and the caller can retry after reloading state. |

## Acceptance Criteria

- The capability accepts a single implementation packet reference and promotes
  the packet one lifecycle step without requiring a packet-level target state.
- The promotion set includes colocated packet artifacts and implementation-
  packet supporting artifacts resolved directly or recursively from `requires`,
  `depends_on`, and `related` relationships.
- Artifacts outside the implementation packet context are excluded even when
  they are reachable through repository relationships.
- Each advanceable included artifact is evaluated against its own next valid
  lifecycle transition.
- Included artifacts that cannot advance one next valid state are reported as
  skipped, are not errors, and remain unchanged.
- Per-artifact confirmations and prerequisite facts are matched by artifact ID
  or artifact type.
- If any advanceable artifact fails validation, prerequisite, confirmation,
  blocker, relationship, evidence, or conflict checks, the whole packet request
  fails with no mutations.
- A successful packet promotion updates every advancing artifact and no skipped,
  excluded, unrelated, or external file.
- Every successful artifact mutation records source state, target state, actor,
  and promotion timestamp.
- Failure diagnostics are complete, actionable, and deterministically ordered.
- Promotion is deterministic, offline, atomic, and does not move files or create
  release records.

## Scope Boundaries

### In Scope

- One-step atomic promotion of one complete implementation packet.
- Discovery of colocated and implementation-packet supporting artifacts.
- Recursive inclusion through `requires`, `depends_on`, and `related` only for
  implementation-packet context.
- Per-artifact next-state derivation, prerequisite checks, confirmations,
  skipped participants, diagnostics, and promotion metadata.
- All-or-nothing mutation across every advancing artifact.

### Out Of Scope

- Promotion to a caller-supplied packet-level target state.
- Packet readiness evaluation without mutation.
- Release-record creation or promotion to `released`.
- Moving packets into `specs/archive/`.
- CLI syntax, exit-code taxonomy, and machine-readable serialization.
- Human semantic judgment or proof that approval is genuine.
- Artifact authoring, ID allocation, and relationship rules already delivered by
  earlier epics.

## Dependencies

- `US-006` single-artifact promotion behavior is available.
- `EPIC-001` and `EPIC-002` validation and relationship-integrity capabilities
  are available.
- `ADR-002` defines strict promote-only status transitions.
- `ADR-007` defines atomic promotion metadata and persistence behavior.

## Open Questions

| ID | Question | Blocking? | Owner | Status |
| --- | --- | --- | --- | --- |
| OQ-001 | If every included artifact is skipped and zero artifacts can advance, should the request be a successful no-op or a non-error skipped result? | No | Product owner | Resolved: successful no-op with all included artifacts reported as skipped. |

## INVEST Check

- [x] Independent
- [x] Negotiable
- [x] Valuable
- [x] Estimable
- [x] Small enough for roughly 1 to 3 days
- [x] Testable

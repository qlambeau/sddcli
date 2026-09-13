---
id: EPIC-003
title: "Lifecycle promotion and readiness"
type: epic-brief
status: approved
created: 2026-09-07
updated: 2026-09-07
owner: TBD
parent: PRD-001
depends_on: [EPIC-001, EPIC-002]
requires: [PRD-001, EPIC-001, EPIC-002, ADR-002]
blockers: []
related: [ADR-002]
approval:
  approved_by: Project owner
  approved_on: 2026-09-07
---

# Epic Brief

<!-- Refines EPIC-003 from the approved PRD into an actionable lifecycle and
readiness capability. Candidate slices are seeds for refine-user-stories. -->

## Outcome Statement

Agents and reviewers can safely advance valid SDD artifacts and feature packets
through their lifecycle, while determining whether a packet satisfies the
Spec-Ready predicate before implementation.

## Capability Boundaries

### In Scope

- Promote a single artifact through a valid lifecycle transition.
- Promote a complete feature packet atomically, with all-or-nothing behavior.
- Enforce the strict forward lifecycle:
  `draft -> in-review -> approved -> implemented -> archived`.
- Enforce guarded special transitions: `approved -> superseded` only with an
  approved successor and valid links, and release records becoming `released`
  only after their included features are implemented and verified.
- Validate prerequisites, blockers, dependencies, references, and promotion
  metadata before applying a transition.
- Restrict promotion to active artifacts and packets; archived, released, and
  already-superseded artifacts are immutable.
- Evaluate the Spec-Ready predicate as a read-only operation using the
  normative workflow checklist.
- Produce complete, deterministic, offline diagnostics.
- Preserve mutation safety and do not move files during promotion.

### Out Of Scope

- CLI user experience and machine-readable serialization, deferred to EPIC-004.
- CI-provider integrations.
- Judging whether a human genuinely approved an artifact.
- Artifact authoring and ID allocation.
- Relationship validation already delivered by EPIC-001 and EPIC-002.
- Release-record compilation and archive file relocation.

## Candidate Vertical Slices

| Candidate slice | User value | Notes / risks |
| --- | --- | --- |
| Promote a single artifact | Agents and reviewers can advance one valid artifact without bypassing lifecycle rules. | Must reject skipped, backward, invalid, and terminal-state transitions while recording promotion metadata. |
| Promote a complete feature packet atomically | Agents can advance a packet consistently without partial lifecycle updates. | All applicable transitions must be preflighted; any failure leaves the packet unchanged. |
| Evaluate Spec-Ready status | Agents can determine whether a packet is ready for implementation without mutating specifications. | The result must apply the normative predicate and report every applicable unmet condition. |
| Enforce guarded completion transitions | The repository can safely represent supersession, release, and archival outcomes. | Supersession, release, and archival conditions must remain explicit and must not reopen terminal artifacts. |

## Success Criteria

- [ ] Valid lifecycle transitions succeed; skipped, backward, invalid, and
      terminal-state transitions fail with actionable diagnostics.
- [ ] Every successful promotion records the source state, target state, actor,
      and promotion timestamp.
- [ ] A failed packet promotion mutates no artifact in the packet.
- [ ] Spec-Ready evaluation reports every applicable unmet predicate and never
      mutates repository content.
- [ ] Valid complete packets pass the readiness check; incomplete or blocked
      packets fail it.
- [ ] Promotion and readiness results are deterministic and require no network
      or AI service.
- [ ] Promotion does not move files and does not alter excluded artifacts.

## Dependencies

- Depends on: EPIC-001 and EPIC-002
- Required artifacts: PRD-001, EPIC-001, EPIC-002, and ADR-002

## Open Questions

| ID | Question | Blocking? | Owner | Status |
| --- | --- | --- | --- | --- |
| OQ-001 | What stable exit-code taxonomy and JSON output schema should downstream agents consume? | No | TBD | Deferred to EPIC-004 |

## Readiness Checklist

- [x] Outcome is stated without prescribing implementation.
- [x] Boundaries exclude future epics and speculative capabilities.
- [x] At least one independently valuable candidate slice is identified.
- [x] Success criteria are observable or explicitly marked `TBD`.
- [x] No blocking questions remain open.

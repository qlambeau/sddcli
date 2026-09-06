---
id: EPIC-002
title: "Identity and relationship integrity"
type: epic-brief
status: implemented
created: 2026-09-05
updated: 2026-09-07
owner: TBD
parent: PRD-001
depends_on: [EPIC-001]
requires: [PRD-001, EPIC-001]
blockers: []
related: [ADR-002, ADR-003, ADR-004, ADR-005, ADR-006]
approval:
  approved_by: Project owner
  approved_on: 2026-09-06
---

# Epic Brief

<!-- Refines EPIC-002 from PRD-001 into a bounded identity and relationship
integrity capability. Candidate slices are seeds for story refinement; delivered
slices remain listed so the brief reflects the current implementation baseline. -->

## Outcome Statement

Agents and CI can verify the identity and relationship integrity of the entire
SDD repository, including active, archived, and superseded artifacts, so that
duplicate identities, broken links, and relationship cycles are detected before
downstream workflow actions.

## Capability Boundaries

### In Scope

- Enforce unique artifact IDs across active, archived, and superseded artifacts.
- Enforce that path-encoded PRD, epic, and ADR filenames agree with their
  frontmatter IDs.
- Resolve current local relationships for parent, epic, dependency, requirement,
  blocker, related, and supersession references.
- Verify that resolved references target the expected artifact kind.
- Report missing targets and wrong-kind targets as diagnostics while continuing
  the complete repository validation.
- Require reciprocal `related` and supersession relationships to agree.
- Detect cycles in parent, dependency, and supersession relationships.
- Preserve valid unrelated artifacts and relationships when integrity failures
  are reported.
- Produce complete, deterministic, offline, read-only integrity results.
- Continue excluding templates and non-artifact supporting documents from the
  integrity graph.

### Out Of Scope

- Schema and release relationships until those artifact types and contracts are
  introduced.
- Lifecycle promotion, promotion metadata, and Spec-Ready evaluation.
- CLI command UX, machine-readable serialization, and CI-provider integration.
- ID reservation, file mutation, persistence, network access, and AI judgment.
- Inferring semantic relationships that are not represented by local artifact
  references.

## Candidate Vertical Slices

| Candidate slice | User value | Notes / risks |
| --- | --- | --- |
| Verify repository-wide artifact identity | Agents and CI can identify duplicate IDs and path/ID mismatches across current and historical artifacts before downstream use | Implemented as `US-002`; identity findings remain part of the integrity baseline |
| Validate relationship targets | Agents and CI can identify missing and wrong-kind relationship targets without suppressing other artifact results | Implemented as `US-003`; current and historical recognized artifacts participate while templates remain excluded |
| Validate reciprocal relationships | Agents and CI can identify asymmetric `related` and supersession links before downstream use | Implemented as `US-004`; valid unrelated relationships and deterministic read-only results remain part of the baseline |
| Detect relationship cycles | Agents and CI can identify invalid cycles across parent, dependency, and supersession relationships before downstream use | Implemented as `US-005`; cycle identities, overlapping-cycle reporting, and source-owned diagnostics are covered by the approved packet |

## Success Criteria

- [x] Valid active and historical artifact graphs pass integrity validation.
- [x] Duplicate IDs are reported with every conflicting artifact location.
- [x] Path-encoded filename and frontmatter ID mismatches are reported.
- [x] Missing and wrong-kind relationship targets are reported without aborting
  validation of other artifacts.
- [x] Non-reciprocal `related` and supersession links are reported.
- [x] Invalid parent, dependency, and supersession cycles are reported.
- [x] Cycle failures do not suppress valid unrelated artifact and relationship
  results.
- [x] Archived and superseded artifacts participate in integrity checks while
  templates and supporting documents remain excluded.
- [x] Results are complete, deterministic, offline, and read-only.
- [x] Schema and release relationship checks remain deferred until their
  contracts exist.

## Dependencies

- Depends on: EPIC-001 (implemented)
- Required artifacts: PRD-001 (approved), EPIC-001 (implemented), ADR-002,
  ADR-003, ADR-004, ADR-005, and ADR-006 (approved)

## Open Questions

- Exact human-readable cycle diagnostic wording remains flexible under `REQ-005`;
  the stable rule ID, cycle identity, source ownership, and overlapping-cycle
  behavior are implemented.

## Readiness Checklist

- [x] Outcome is stated without prescribing implementation.
- [x] Boundaries exclude future epics and speculative capabilities.
- [x] At least one independently valuable slice is identified.
- [x] Success criteria are observable or explicitly marked `TBD`.
- [x] No blocking questions remain open.

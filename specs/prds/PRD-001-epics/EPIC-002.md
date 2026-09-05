---
id: EPIC-002
title: "Identity and relationship integrity"
type: epic-brief
status: approved
created: 2026-09-05
updated: 2026-09-05
owner: TBD
parent: PRD-001
depends_on: [EPIC-001]
requires: [PRD-001, EPIC-001]
blockers: []
related: [ADR-002, ADR-003, ADR-004, ADR-005]
approval:
  approved_by: Project owner
  approved_on: 2026-09-05
---

# Epic Brief

<!-- Refines EPIC-002 from PRD-001 into a bounded identity and relationship
integrity capability. Candidate slices are seeds for story refinement. -->

## Outcome Statement

Agents and CI can verify the identity and relationship integrity of the entire
SDD repository, including active, archived, and superseded artifacts, so that
duplicate identities and broken links are detected before downstream workflow
actions.

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
| Verify repository-wide identity and relationship integrity in one pass | Agents and CI can identify duplicate IDs and broken artifact links, including links involving historical artifacts, before promotion or downstream use | The graph must distinguish valid reciprocal links from invalid cycles and must retain every actionable diagnostic without treating templates as repository artifacts |

## Success Criteria

- [ ] Valid active and historical artifact graphs pass integrity validation.
- [ ] Duplicate IDs are reported with every conflicting artifact location.
- [ ] Path-encoded filename and frontmatter ID mismatches are reported.
- [ ] Missing and wrong-kind relationship targets are reported without aborting
  validation of other artifacts.
- [ ] Non-reciprocal `related` and supersession links are reported.
- [ ] Invalid parent, dependency, and supersession cycles are reported.
- [ ] Archived and superseded artifacts participate in integrity checks while
  templates and supporting documents remain excluded.
- [ ] Results are complete, deterministic, offline, and read-only.
- [ ] Schema and release relationship checks remain deferred until their
  contracts exist.

## Dependencies

- Depends on: EPIC-001 (implemented)
- Required artifacts: PRD-001 (approved), EPIC-001 (implemented), ADR-002,
  ADR-003, ADR-004, and ADR-005 (approved)

## Open Questions

No unresolved or blocking questions.

## Readiness Checklist

- [x] Outcome is stated without prescribing implementation.
- [x] Boundaries exclude future epics and speculative capabilities.
- [x] At least one independently valuable slice is identified.
- [x] Success criteria are observable or explicitly marked `TBD`.
- [x] No blocking questions remain open.

---
id: EPIC-001
title: "Deterministic artifact validation"
type: epic-brief
status: approved
created: 2026-09-03
updated: 2026-09-03
owner: TBD
parent: PRD-001
depends_on: []
requires: [PRD-001]
blockers: []
related: []
approval:
  approved_by: Project owner
  approved_on: 2026-09-03
---

# Epic Brief

## Outcome Statement

Agents and automated checks can deterministically validate the structure and
completeness of active SDD artifacts and receive actionable diagnostics.

## Capability Boundaries

### In Scope

- Validate active PRDs, epic briefs, user stories, scenarios, requirements,
  designs, tasks, and ADRs.
- Enforce required frontmatter fields and allowed values for each artifact type.
- Enforce required document sections and review or readiness checklists for each
  artifact type.
- Validate common syntax, placeholders, local IDs, and local-reference syntax.
- Report all applicable diagnostics in one validation run.
- Include the artifact path, location when available, rule identifier, severity,
  message, and remediation guidance in each diagnostic.
- Produce deterministic results without network or AI dependencies.

### Out Of Scope

- Scanning `specs/templates/` or `specs/archive/` as active artifacts.
- Global ID uniqueness, collision detection, and cross-artifact relationship
  resolution.
- Lifecycle promotion and Spec-Ready evaluation.
- CLI command UX, machine-readable serialization, and CI-provider integration.

## Candidate Vertical Slices

| Candidate slice | User value | Notes / risks |
| --- | --- | --- |
| Validate all eight active artifact types in one structural validation pass | Agents and automated checks can identify malformed SDD documents before they are reviewed or consumed downstream | Type-specific rules must remain distinct; one malformed artifact may produce multiple diagnostics |

## Success Criteria

- [ ] Valid fixtures for all eight artifact types pass structural validation.
- [ ] Invalid fixtures for all eight artifact types fail with all applicable
  diagnostics in one run.
- [ ] Each diagnostic includes the agreed actionable context.
- [ ] Template and archive files are ignored as active artifacts.
- [ ] Results are deterministic and require no network or AI service.
- [ ] Validation does not perform global uniqueness or cross-artifact integrity
  checks reserved for EPIC-002.

## Dependencies

- Depends on: none
- Required artifacts: PRD-001 (approved)

## Open Questions

No unresolved or blocking questions.

## Readiness Checklist

- [x] Outcome is stated without prescribing implementation.
- [x] Boundaries exclude future epics and speculative capabilities.
- [x] At least one independently valuable slice is identified.
- [x] Success criteria are observable or explicitly marked `TBD`.
- [x] No blocking questions remain open.

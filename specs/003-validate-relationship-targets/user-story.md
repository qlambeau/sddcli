---
id: US-003
title: "Validate relationship targets"
type: user-story
status: approved
created: 2026-09-05
updated: 2026-09-05
owner: TBD
parent: PRD-001
epic: EPIC-002
feature: 003-validate-relationship-targets
depends_on: [US-002]
requires: [EPIC-002]
blockers: []
related: []
approval:
  approved_by: Project owner
  approved_on: 2026-09-05
---

# User Story

## Story Card

As an LLM coding agent or CI check, I want local artifact relationships resolved
against the repository, so that missing and wrong-kind targets are detected
before lifecycle or downstream actions.

## Context And Value

Artifact relationships connect a specification to its parent, epic, dependency,
requirement, blocker, related artifact, or superseded successor. A reference can
look syntactically valid while pointing to an artifact that does not exist or to
an artifact of the wrong kind. These errors make downstream workflow actions
ambiguous and weaken the traceability of the repository.

Resolving concrete relationships against the complete recognized artifact set
lets agents and CI identify broken links while retaining valid and unrelated
results for correction.

## Business Rules

- The relationship target set includes recognized canonical artifacts in active
  locations and under `specs/archive/`, including artifacts marked
  `superseded`.
- Files under `specs/templates/` and non-artifact supporting files are excluded
  from relationship target resolution.
- Applicable `parent`, `epic`, `depends_on`, `requires`, `blockers`, `related`,
  `supersedes`, and `superseded_by` references are resolved.
- An applicable `parent` reference from an epic or user story targets a PRD; an
  applicable `parent` reference from requirements, design, or task artifacts
  targets a user story.
- An applicable `epic` reference from a user story targets an epic.
- `depends_on`, `requires`, `blockers`, and `related` references may target any
  recognized canonical artifact.
- `supersedes` and `superseded_by` references target an artifact of the same
  kind as the referencing artifact.
- A concrete syntactically valid reference to no recognized target is a missing
  target failure.
- A concrete syntactically valid reference to a target of the wrong kind is a
  wrong-kind failure.
- Each missing or wrong-kind relationship entry receives its own actionable
  diagnostic on the referencing artifact.
- Relationship validation continues after failures and retains valid unrelated
  artifact results.
- Empty relationship collections do not create target failures.
- Malformed, empty, or unresolved relationship values remain governed by
  EPIC-001 structural diagnostics and do not receive an additional target
  failure from this story.
- Active, archived, and superseded target resolution is deterministic, offline,
  complete, and read-only.

## Examples

| Example | Given | When | Expected outcome |
| --- | --- | --- | --- |
| EX-001 | All concrete relationships point to existing artifacts of the expected kind | Relationship validation runs | The relationship result succeeds and no target diagnostic is reported |
| EX-002 | A concrete relationship points to an ID absent from active and historical artifacts | Relationship validation runs | The referencing artifact receives an actionable missing-target diagnostic and validation continues |
| EX-003 | A user story's `parent` points to an existing ADR instead of a PRD | Relationship validation runs | The referencing artifact receives an actionable wrong-kind diagnostic identifying the expected and actual kinds |
| EX-004 | A relationship points to an archived or superseded recognized artifact | Relationship validation runs | The historical target resolves successfully and creates no missing-target diagnostic |
| EX-005 | A template contains an ID that appears in a relationship reference but no canonical artifact has that ID | Relationship validation runs | The template is excluded and the reference is treated as missing from the recognized target set |
| EX-006 | One artifact has several invalid relationship entries while other artifacts have valid links | Relationship validation runs | One diagnostic is reported per invalid entry and valid unrelated results remain represented |
| EX-007 | A relationship value is malformed, empty, or unresolved | Relationship validation runs | EPIC-001 retains responsibility for the structural finding and no duplicate target diagnostic is added |

## Acceptance Criteria

- Given recognized active, archived, and superseded artifacts with valid
  relationship targets, when relationship validation runs, then all concrete
  references resolve and the overall relationship result succeeds.
- Given a concrete relationship ID with no recognized canonical target, when
  relationship validation runs, then the referencing artifact receives an
  actionable missing-target diagnostic and the overall result fails.
- Given a concrete relationship ID whose recognized target has the wrong
  expected kind, when relationship validation runs, then the referencing artifact
  receives an actionable wrong-kind diagnostic that identifies the relationship,
  expected kind, and actual kind.
- Given multiple missing or wrong-kind relationship entries, when relationship
  validation runs, then each invalid entry receives its own diagnostic.
- Given relationship failures in one artifact, when relationship validation runs,
  then valid unrelated artifacts remain represented and are still checked.
- Given a relationship target in an archived or superseded artifact, when
  relationship validation runs, then the target resolves regardless of lifecycle
  state.
- Given an ID that appears only in a template or non-artifact supporting file,
  when relationship validation runs, then the excluded file cannot satisfy the
  relationship.
- Given a malformed, empty, or unresolved relationship value, when relationship
  validation runs, then EPIC-001 structural diagnostics remain intact and no
  additional target diagnostic is reported.
- Given the same repository state and inputs, repeated relationship validation
  produces the same complete ordered result without network access or mutation.

## Scope Boundaries

### In Scope

- Resolving concrete local relationships for parent, epic, dependency,
  requirement, blocker, related, and supersession fields.
- Missing-target and wrong-kind diagnostics with one finding per invalid entry.
- Active, archived, and superseded recognized target artifacts.
- Complete continuation after relationship failures.
- Deterministic, offline, read-only relationship results.

### Out Of Scope

- Reciprocal `related` or supersession consistency.
- Parent, dependency, or supersession cycle detection.
- Schema and release relationship validation until those contracts exist.
- Re-parsing malformed or unrecognized frontmatter or relationship syntax.
- Lifecycle promotion, Spec-Ready evaluation, CLI behavior, serialization,
  persistence, file mutation, ID reservation, and network access.

## Dependencies

- Implemented `US-002` provides repository-wide recognized artifact identities and
  uniqueness diagnostics.
- Approved `PRD-001` and `EPIC-002` provide the product and capability boundary.
- EPIC-001 structural validation remains the source of truth for malformed,
  empty, and unresolved relationship values.
- No runtime, external service, schema, or persistence dependency is introduced.

## Open Questions

| ID | Question | Blocking? | Owner | Status |
| --- | --- | --- | --- | --- |
| None | No unresolved or blocking questions remain. | No | Project owner | Closed |

## INVEST Check

- [x] Independent
- [x] Negotiable
- [x] Valuable
- [x] Estimable
- [x] Small enough for roughly 1 to 3 days
- [x] Testable

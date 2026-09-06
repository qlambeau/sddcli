---
id: US-004
title: "Validate reciprocal relationships"
type: user-story
status: approved
created: 2026-09-05
updated: 2026-09-05
owner: TBD
parent: PRD-001
epic: EPIC-002
feature: 004-validate-reciprocal-relationships
depends_on: [US-003]
requires: [EPIC-002]
blockers: []
related: []
approval:
  approved_by: Project owner
  approved_on: 2026-09-05
---

# User Story

## Story Card

As an LLM coding agent or CI check, I want reciprocal `related` and supersession
relationships validated in both directions, so that asymmetric links are
detected before downstream actions.

## Context And Value

Artifact relationships provide traceability between specifications and their
related or superseding artifacts. A relationship can be syntactically valid and
point to an existing artifact while its required reverse link is absent or
points elsewhere. These asymmetric links make the repository graph incomplete
and weaken the audit trail used by agents and CI.

Checking reciprocity against the complete recognized artifact set lets agents
and CI identify incomplete traceability while retaining valid and unrelated
results for correction.

## Business Rules

- The reciprocity target set includes recognized canonical artifacts in active
  locations and under `specs/archive/`, including artifacts marked
  `superseded`.
- Files under `specs/templates/` and non-artifact supporting files are excluded
  from reciprocity checks.
- For a concrete `A.related` reference to `B`, the recognized target `B` must
  contain `A` in its `related` collection.
- For a concrete `A.supersedes` reference to `B`, the recognized target `B` must
  contain `A` in its `superseded_by` field.
- For a concrete `A.superseded_by` reference to `B`, the recognized target `B`
  must contain `A` in its `supersedes` field.
- Reciprocity uses unordered membership. Relationship list order and duplicate
  occurrences do not affect whether a counterpart exists.
- Only concrete, syntactically valid relationships whose targets resolve are
  evaluated by this story.
- Malformed, empty, unresolved, missing-target, and wrong-kind values remain
  governed by EPIC-001 and US-003 and do not receive duplicate reciprocity
  findings.
- Each unmatched directed relationship entry receives one actionable diagnostic
  on its source artifact identifying the source field, target ID, and expected
  reverse field or link.
- A reverse link that points to a different artifact does not satisfy the
  original link; each unmatched directed entry is diagnosed independently.
- Reciprocity validation continues after failures and retains valid unrelated
  artifact results.
- Empty relationship collections do not create reciprocity failures.
- Active, archived, and superseded artifact reciprocity validation is
  deterministic, offline, complete, and read-only.

## Examples

| Example | Given | When | Expected outcome |
| --- | --- | --- | --- |
| EX-001 | Artifact A relates to artifact B and B relates to A | Reciprocity validation runs | The relationship is reciprocal and no reciprocity diagnostic is reported |
| EX-002 | Artifact A relates to artifact B but B does not relate to A | Reciprocity validation runs | A receives one actionable diagnostic identifying the missing reverse `related` link and the overall result fails |
| EX-003 | Artifact A relates to B but B relates to C instead | Reciprocity validation runs | A's unmatched link receives a diagnostic, and B's independent link is checked separately |
| EX-004 | Artifact A supersedes historical artifact B and B identifies A with `superseded_by` | Reciprocity validation runs | The supersession is reciprocal regardless of lifecycle state and no reciprocity diagnostic is reported |
| EX-005 | Artifact A supersedes B but B does not identify A with `superseded_by` | Reciprocity validation runs | A receives one actionable missing-counterpart diagnostic and the overall result fails |
| EX-006 | Several artifacts contain asymmetric links alongside valid links | Reciprocity validation runs | Each unmatched directed entry receives one diagnostic and valid unrelated artifacts remain represented |
| EX-007 | A relationship value is malformed, empty, unresolved, missing, or wrong-kind | Reciprocity validation runs | The earlier structural or target diagnostic remains and no duplicate reciprocity diagnostic is added |
| EX-008 | Reciprocal list entries appear in different order or with duplicate occurrences | Reciprocity validation runs | Membership matches and no reciprocity diagnostic is reported |

## Acceptance Criteria

- Given recognized active and historical artifacts with reciprocal `related`
  links, when reciprocity validation runs, then the links pass without a
  reciprocity diagnostic.
- Given a concrete resolved `related` link without its exact reverse membership,
  when reciprocity validation runs, then the source artifact receives one
  actionable missing-counterpart diagnostic and the overall result fails.
- Given a concrete resolved supersession link without its exact reverse field
  membership, when reciprocity validation runs, then the source artifact
  receives one actionable missing-counterpart diagnostic and the overall result
  fails.
- Given a reverse field that points to a different artifact, when reciprocity
  validation runs, then each unmatched directed entry is diagnosed independently.
- Given multiple asymmetric links, when reciprocity validation runs, then one
  diagnostic is reported per unmatched directed entry and validation continues.
- Given a reciprocal link involving an archived or superseded artifact, when
  reciprocity validation runs, then lifecycle state does not prevent resolution.
- Given a template or non-artifact supporting file containing a referenced ID,
  when reciprocity validation runs, then the excluded file cannot satisfy the
  counterpart relationship.
- Given empty relationship collections, when reciprocity validation runs, then
  no reciprocity failure is created solely because they are empty.
- Given malformed, empty, unresolved, missing-target, or wrong-kind values, when
  reciprocity validation runs, then the existing EPIC-001 or US-003 diagnostic
  remains and no duplicate reciprocity diagnostic is reported.
- Given the same repository state and inputs, repeated reciprocity validation
  produces the same complete ordered result without network access or mutation.

## Scope Boundaries

### In Scope

- Reciprocal membership for `related` relationships.
- Reciprocal membership between `supersedes` and `superseded_by` relationships.
- Active, archived, and superseded recognized artifacts.
- One source-owned diagnostic per unmatched directed relationship entry.
- Complete continuation, deterministic ordering, offline operation, and
  read-only results.

### Out Of Scope

- Relationship target existence and expected-kind validation, which belong to
  US-003.
- Malformed frontmatter or relationship syntax, which belongs to EPIC-001.
- Parent, dependency, or supersession cycle detection.
- Schema and release relationships until those contracts exist.
- Lifecycle promotion, Spec-Ready evaluation, CLI behavior, serialization,
  persistence, file mutation, ID reservation, and network access.
- Semantic inference of relationships not represented by local references.

## Dependencies

- Implemented `US-003` provides recognized active and historical relationship
  targets and target diagnostics.
- Implemented `US-002` provides repository-wide recognized artifact identities
  and uniqueness diagnostics.
- Approved `PRD-001` and `EPIC-002` provide the product and capability boundary.
- EPIC-001 structural validation remains the source of truth for malformed,
  empty, and unresolved relationship values.
- No runtime, external service, schema, persistence, or new dependency is
  introduced.

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

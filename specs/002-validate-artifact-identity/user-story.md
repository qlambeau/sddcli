---
id: US-002
title: "Validate repository-wide artifact identity"
type: user-story
status: approved
created: 2026-09-05
updated: 2026-09-05
owner: TBD
parent: PRD-001
epic: EPIC-002
feature: 002-validate-artifact-identity
depends_on: []
requires: [EPIC-001]
blockers: []
related: []
approval:
  approved_by: Project owner
  approved_on: 2026-09-05
---

# User Story

## Story Card

As an LLM coding agent or CI check, I want artifact identities checked across
the repository's current and historical artifacts, so that duplicate IDs and
path/ID mismatches are found before relationship or lifecycle actions.

## Context And Value

Artifact IDs are used to identify specifications and connect later workflow
actions to the correct source. Reusing an ID in active or historical content,
or changing a path-encoded identity without updating frontmatter, can make
references ambiguous and compromise the audit trail. A complete identity
result lets an agent or CI check correct those conflicts before resolving
relationships or promoting artifacts.

## Business Rules

- The identity set includes canonical artifact files in active locations and in
  `specs/archive/`, including artifacts marked as superseded.
- Files under `specs/templates/` and non-artifact supporting files do not enter
  the identity set.
- Every recognized artifact ID is globally unique across the identity set,
  regardless of the artifact's lifecycle state.
- PRD, epic, and ADR filenames encode an identity that must match the
  artifact's recognized frontmatter ID.
- A duplicate ID identifies every conflicting artifact path and does not stop
  identity validation for other artifacts.
- A path/frontmatter mismatch identifies the artifact and retains it in the
  complete result.
- Identity checks build on EPIC-001 artifact recognition; malformed or
  unrecognized frontmatter remains a structural path-based diagnostic rather
  than being reparsed by this story.
- Identity validation is deterministic, offline, and read-only.

## Examples

| Example | Given | When | Expected outcome |
| --- | --- | --- | --- |
| EX-001 | Active and archived artifacts have distinct recognized IDs and matching path-encoded IDs | Identity validation runs | The identity result succeeds and every recognized artifact identity is accepted once |
| EX-002 | An active artifact and an archived artifact both use `US-001` | Identity validation runs | Both artifact paths receive duplicate-ID diagnostics, all other artifacts are still checked, and the overall result fails |
| EX-003 | `PRD-003.md` contains recognized frontmatter ID `PRD-004` | Identity validation runs | The file receives a path-identity diagnostic showing the filename and frontmatter identities, and validation continues |
| EX-004 | A template or supporting file contains an ID that duplicates an active artifact | Identity validation runs | The excluded file does not participate in identity comparison or produce an identity result |
| EX-005 | One artifact has an identity conflict while other artifacts are valid | Identity validation runs | The conflict is reported without suppressing the valid artifacts or their results |

## Acceptance Criteria

- Given a repository containing active, archived, and superseded artifacts with
  unique recognized IDs, when identity validation runs, then the result
  succeeds and includes each recognized artifact identity once.
- Given a recognized ID used by more than one canonical artifact anywhere in
  the repository, when identity validation runs, then every conflicting path
  receives an actionable duplicate-ID diagnostic and the overall result fails.
- Given a PRD, epic, or ADR whose filename ID differs from its recognized
  frontmatter ID, when identity validation runs, then the artifact receives an
  actionable path-identity diagnostic and remains in the complete result.
- Given a duplicate ID in a template or supporting file, when identity
  validation runs, then that excluded file does not participate in the
  comparison.
- Given an identity conflict, when identity validation runs, then unrelated
  artifacts are still checked and reported.
- Given the same repository state and inputs, repeated identity validation
  produces the same ordered result without network access or file mutation.

## Scope Boundaries

### In Scope

- Repository-wide identity collection for active, archived, and superseded
  canonical artifacts.
- Global exact-ID uniqueness and complete collision diagnostics.
- Filename/frontmatter identity agreement for PRDs, epics, and ADRs.
- Deterministic, offline, read-only identity results.

### Out Of Scope

- Resolving relationship targets or checking expected target kinds.
- Reciprocal relationship consistency and relationship-cycle detection.
- Schema and release relationship validation.
- Lifecycle promotion, Spec-Ready evaluation, CLI behavior, serialization,
  persistence, file mutation, and ID reservation.
- Re-parsing malformed or unrecognized frontmatter already handled by EPIC-001.

## Dependencies

- Implemented EPIC-001 provides recognized artifact identities and structural
  diagnostics.
- Approved PRD-001 and EPIC-002 provide the product and epic boundaries.
- No runtime or external service dependencies.

## Open Questions

No unresolved or blocking questions.

## INVEST Check

- [x] Independent
- [x] Negotiable
- [x] Valuable
- [x] Estimable
- [x] Small enough for roughly 1 to 3 days
- [x] Testable

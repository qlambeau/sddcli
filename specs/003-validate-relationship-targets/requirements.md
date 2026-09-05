---
id: REQ-003
title: "Relationship target validation requirements"
type: feature-requirements
status: approved
created: 2026-09-05
updated: 2026-09-05
owner: TBD
parent: US-003
depends_on: []
requires: [US-003, REQ-002]
blockers: []
related: [PRD-001, EPIC-002, ADR-002, ADR-003, ADR-004, ADR-005, ADR-006]
approval:
  approved_by: Project owner
  approved_on: 2026-09-05
---

# Requirements

<!-- Requirements describe externally observable behavior. Architecture and
implementation sequencing belong in design.md and tasks.md. -->

## Purpose And Actors

### Purpose

Define the observable contract for resolving concrete local artifact
relationships against the complete recognized repository artifact set, so that
missing and wrong-kind targets are detected without suppressing valid results.

### Actors And External Systems

- LLM coding agent.
- CI check.
- The current repository and its local `specs/` tree as the validation source.
- No external system, network service, or AI service.

## Preconditions

- The current repository can be identified and its eligible canonical artifact
  locations can be read.
- EPIC-001 artifact recognition and structural diagnostics are available.
- REQ-002 identity discovery provides the recognized active and historical
  artifact set, including recognized artifact IDs and kinds.
- The repository may contain active, archived, and superseded artifacts, or no
  eligible recognized artifacts.
- Validation can run without network access.

## Inputs And Outputs

| Interaction | Inputs | Outputs | Validation |
| --- | --- | --- | --- |
| Validate repository relationship targets | Current repository state, recognized canonical artifacts, their recognized kinds, and applicable relationship values | An overall relationship result and an ordered list of per-artifact results containing valid relationships and relationship diagnostics | Concrete syntactically valid references resolve against the eligible target set; missing or wrong-kind targets fail; malformed, empty, and unresolved values remain structural diagnostics |

## Relationship Result Contract

- The eligible target set includes recognized canonical artifacts in active
  locations and under `specs/archive/`, including artifacts marked
  `superseded`.
- Files under `specs/templates/` and non-artifact supporting files are excluded
  from the target set and cannot satisfy a relationship.
- The relationship result represents every eligible recognized artifact exactly
  once, with its repository-relative path, recognized kind, applicable valid
  relationships, and diagnostics.
- The overall relationship result is `success` when no relationship target
  error exists, including when the eligible artifact set or relationship
  collections are empty. It is `failure` when at least one missing-target or
  wrong-kind error exists.
- Relationship diagnostics use stable rule IDs in the
  `ARTIFACT.RELATIONSHIP.*` namespace, error severity, repository-relative
  source path, actionable context, and remediation guidance.
- A missing-target diagnostic identifies the relationship field and concrete
  target ID. A wrong-kind diagnostic identifies the relationship field, target
  ID, expected kind, and actual kind.
- Relationship diagnostics are attached to the referencing artifact. Each
  invalid concrete relationship entry produces exactly one diagnostic.
- Relationship validation preserves the deterministic ordering established by
  `REQ-001`; stable rule ID, relationship field, and target ID provide stable
  tie-breakers for relationship findings.

## Functional Requirements

| ID | Requirement | Priority | Traceability (Story & Scenario) |
| --- | --- | --- | --- |
| FR-001 | The relationship validator shall resolve targets only from recognized canonical artifacts in active locations and `specs/archive/`, including artifacts marked `superseded`, and shall exclude templates and non-artifact supporting files. | Must | US-003 / Scenarios: Valid relationships resolve across current and historical artifacts; Archived and superseded targets resolve; Excluded files cannot satisfy a relationship |
| FR-002 | The relationship validator shall evaluate applicable `parent`, `epic`, `depends_on`, `requires`, `blockers`, `related`, `supersedes`, and `superseded_by` references that are concrete and syntactically valid. | Must | US-003 / Scenarios: Valid relationships resolve across current and historical artifacts; Empty relationship collections do not create target failures; Wrong-kind targets remain diagnosable |
| FR-003 | The relationship validator shall apply these expected-target rules: an epic or user story `parent` targets a PRD; a requirements, design, or task `parent` targets a user story; a user story `epic` targets an epic; `depends_on`, `requires`, `blockers`, and `related` may target any recognized canonical artifact; and `supersedes` and `superseded_by` target the same kind as the referencing artifact. | Must | US-003 / Scenarios: Valid relationships resolve across current and historical artifacts; Wrong-kind targets remain diagnosable |
| FR-004 | When a concrete syntactically valid relationship target ID is absent from the eligible recognized target set, the validator shall attach one actionable `ARTIFACT.RELATIONSHIP.MISSING_TARGET` diagnostic to the referencing artifact, identify the relationship field and target ID, continue validation, and return an overall failure. | Must | US-003 / Scenario: Missing targets are reported for every invalid relationship entry |
| FR-005 | When a concrete syntactically valid relationship target resolves to an artifact that violates the expected-target rule, the validator shall attach one actionable `ARTIFACT.RELATIONSHIP.WRONG_KIND` diagnostic to the referencing artifact, identify the relationship field, target ID, expected kind, and actual kind, continue validation, and return an overall failure. | Must | US-003 / Scenario Outline: Wrong-kind targets remain diagnosable |
| FR-006 | The relationship validator shall report one missing-target or wrong-kind diagnostic for every invalid concrete relationship entry, including multiple invalid entries in one artifact, without collapsing entries that happen to use the same target ID. | Must | US-003 / Scenarios: Missing targets are reported for every invalid relationship entry; Relationship failures do not suppress unrelated artifacts |
| FR-007 | The relationship validator shall continue checking all eligible artifacts after relationship failures and shall retain valid relationships and valid unrelated artifact results without relationship target diagnostics. | Must | US-003 / Scenario: Relationship failures do not suppress unrelated artifacts |
| FR-008 | Empty relationship collections and other empty relationship values shall produce no relationship target diagnostic and shall not cause a relationship result failure solely because they are empty. | Must | US-003 / Scenario: Empty relationship collections do not create target failures |
| FR-009 | Malformed, empty, or unresolved relationship values shall remain represented by EPIC-001 structural diagnostics and shall not receive an additional missing-target or wrong-kind diagnostic from relationship target validation. | Must | US-003 / Scenario: Structural relationship failures are not duplicated as target failures |
| FR-010 | The relationship validator shall produce the same complete ordered result for the same repository state and inputs on repeated runs, without network access or modifying artifact content, lifecycle status, or persisted validation results. | Must | US-003 / Scenario: Relationship validation is deterministic and read-only |

## Postconditions And Invariants

- Every eligible recognized artifact is represented exactly once in the
  relationship result.
- Every concrete syntactically valid relationship entry is evaluated against
  the eligible recognized target set and its applicable expected-kind rule.
- Every missing or wrong-kind relationship entry has exactly one diagnostic on
  its referencing artifact.
- A valid reference to an active, archived, or superseded recognized artifact
  does not produce a target diagnostic solely because of lifecycle state.
- No template or supporting file is represented as an eligible target or
  satisfies a relationship.
- Empty relationship collections and values do not produce target failures.
- Structural relationship diagnostics remain attributable to EPIC-001 and are
  not duplicated by target validation.
- Any relationship target error produces overall `failure`; a clean or empty
  relationship set produces overall `success`.
- Valid unrelated artifact results remain represented after relationship
  failures.
- No artifact content, lifecycle status, or persisted validation result changes.

## Edge And Failure Behavior

| Condition | Expected behavior | User-visible result |
| --- | --- | --- |
| No eligible recognized artifacts exist | Complete relationship validation without a target error | Overall `success` and an empty relationship result list |
| A relationship collection is empty | Treat the collection as having no concrete entries | No missing-target or wrong-kind diagnostic |
| A concrete target ID is absent from active and historical recognized artifacts | Record the failure for the referencing artifact and continue | One actionable missing-target diagnostic for the entry and overall `failure` |
| A concrete target ID resolves to the wrong artifact kind | Record the failure for the referencing artifact and continue | One actionable wrong-kind diagnostic containing expected and actual kinds and overall `failure` |
| A target exists only in `specs/templates/` or a non-artifact supporting file | Exclude that file from target resolution | The relationship receives a missing-target diagnostic |
| A target is archived or marked `superseded` | Include the recognized artifact in target resolution regardless of lifecycle state | The relationship resolves without a missing-target diagnostic |
| One artifact has several invalid relationship entries | Diagnose each entry independently | One diagnostic per invalid entry; validation continues |
| Other artifacts have valid relationships while one artifact has failures | Continue the complete validation | Valid unrelated artifacts remain represented and free of target diagnostics |
| A relationship value is malformed, empty, or unresolved | Leave the value to EPIC-001 structural validation and skip target resolution | Structural diagnostic remains; no additional target diagnostic |
| The same repository is validated repeatedly without network access | Use the same eligible inputs and stable ordering | Identical complete ordered results and no mutation |

## Quality Requirements

- Relationship validation is deterministic for identical repository state and
  inputs.
- Relationship validation requires no network, hosted service, or AI
  dependency.
- Relationship validation is read-only and persists no results.
- Relationship validation reports all applicable target findings rather than
  stopping at the first error.
- Result and diagnostic ordering is stable and consistent with `REQ-001`.
- Diagnostics contain sufficient source path, relationship field, target ID,
  rule, severity, message, and remediation context for an agent or CI check to
  correct the source artifact.
- The contract is suitable for pull-request validation; no exact runtime
  threshold is introduced because PRD-001 leaves that target open.

## Dependencies And Deferred Decisions

- Source story: approved `US-003`.
- Parent epic: approved `EPIC-002`.
- Recognized artifact identities and uniqueness boundary: approved `REQ-002`
  and implemented US-002.
- Structural recognition and malformed-value diagnostics: implemented EPIC-001
  and approved `REQ-001`.
- Layered validation and separate active/historical discovery boundaries are
  governed by approved `ADR-005` and `ADR-006`; this contract does not select
  an implementation architecture.
- Reciprocal `related` or supersession consistency, relationship cycle
  detection, schema and release relationships, lifecycle promotion, Spec-Ready
  evaluation, CLI behavior, serialization, persistence, file mutation, and
  network access remain deferred or out of scope as stated by US-003.
- No runtime, external service, schema, persistence, or new dependency is
  required by this observable contract.

## Traceability

- Source story: `US-003`
- Parent epic: `EPIC-002`
- Parent PRD: `PRD-001`
- Preceding contract: `REQ-002`
- Structural validation contract: `REQ-001`
- Executable scenarios: `scenarios.feature`
- Covered scenarios: Valid relationships resolve across current and historical
  artifacts; Empty relationship collections do not create target failures;
  Missing targets are reported for every invalid relationship entry; Wrong-kind
  targets remain diagnosable; Archived and superseded targets resolve; Excluded
  files cannot satisfy a relationship; Relationship failures do not suppress
  unrelated artifacts; Structural relationship failures are not duplicated as
  target failures; Relationship validation is deterministic and read-only.

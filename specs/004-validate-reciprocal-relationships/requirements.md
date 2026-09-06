---
id: REQ-004
title: "Reciprocal relationship validation requirements"
type: feature-requirements
status: approved
created: 2026-09-06
updated: 2026-09-06
owner: TBD
parent: US-004
depends_on: [REQ-003]
requires: [US-004, REQ-003]
blockers: []
related: [PRD-001, EPIC-002, ADR-002, ADR-003, ADR-004, ADR-005, ADR-006, REQ-003]
approval:
  approved_by: Project owner
  approved_on: 2026-09-06
---

# Requirements

<!-- Requirements describe externally observable behavior. Architecture and
implementation sequencing belong in design.md and tasks.md. -->

## Purpose And Actors

### Purpose

Define the observable contract for checking that concrete, resolved `related`
and supersession relationships have the required reverse membership across the
complete recognized repository artifact set.

### Actors And External Systems

- LLM coding agent.
- CI check.
- The current repository and its local `specs/` tree as the validation source.
- No external system, network service, or AI service.

## Preconditions

- The current repository can be identified and its eligible canonical artifact
  locations can be read.
- EPIC-001 structural validation and its diagnostics are available.
- REQ-003 relationship target validation has identified the recognized
  canonical artifact set, normalized relationship entries, and any structural
  or target diagnostics.
- The recognized artifact set includes active artifacts and eligible artifacts
  under `specs/archive/`, including artifacts marked `superseded`.
- Files under `specs/templates/` and non-artifact supporting files are excluded
  from the recognized artifact set.
- Validation can run without network access and without mutating the
  repository.

## Inputs And Outputs

| Interaction | Inputs | Outputs | Validation |
| --- | --- | --- | --- |
| Validate reciprocal relationships | Current repository state, the complete recognized artifact set, each artifact's normalized relationship entries, and prior structural or target diagnostics | An ordered per-artifact result set containing the valid relationship results and reciprocity diagnostics, plus an overall reciprocal-consistency result | Evaluate only concrete, syntactically valid entries whose targets resolve to recognized artifacts; leave malformed, empty, unresolved, missing-target, and wrong-kind values to their existing diagnostics |

## Reciprocal Relationship Contract

- The recognized artifact set is the same active and historical set established
  by REQ-003. Every recognized artifact is eligible to provide or satisfy a
  counterpart, regardless of active, archived, or `superseded` lifecycle state.
- The required counterpart mapping is:

  | Source field | Required reverse field on the target |
  | --- | --- |
  | `related` | `related` |
  | `supersedes` | `superseded_by` |
  | `superseded_by` | `supersedes` |

- A directed relationship entry is eligible for reciprocity validation only
  when it is concrete, syntactically valid, resolves to a recognized target,
  and has no applicable structural or target diagnostic from EPIC-001 or
  REQ-003.
- Reciprocity uses unordered membership. A counterpart exists when the target's
  required reverse field contains the source artifact ID at least once.
  Relationship list order and duplicate occurrences do not affect the result.
- A concrete eligible entry without its required counterpart is an unmatched
  directed relationship entry. The reverse field must contain the exact source
  artifact ID; a link to another artifact does not satisfy it.
- Each unmatched directed relationship entry produces exactly one
  `ARTIFACT.RELATIONSHIP.NON_RECIPROCAL` error diagnostic on the source
  artifact. The diagnostic identifies the source field, target artifact ID,
  and required reverse field or link.
- A reciprocity diagnostic is not added for an entry already represented by a
  structural or target diagnostic. Such an entry is not eligible for this
  contract.
- The overall reciprocal-consistency result is `success` when no unmatched
  eligible entry exists, including when the recognized artifact set or all
  relationship collections are empty. It is `failure` when at least one
  unmatched eligible entry exists.

## Functional Requirements

| ID | Requirement | Priority | Traceability (Story & Scenario) |
| --- | --- | --- | --- |
| FR-001 | The reciprocity validator shall use the complete recognized canonical artifact set from active locations and `specs/archive/`, including artifacts marked `superseded`, and shall exclude templates and non-artifact supporting files from counterpart resolution. | Must | US-004 / Scenarios: Reciprocal related links resolve across current and historical artifacts; Reciprocal supersession links resolve across historical artifacts; Excluded files cannot satisfy a reciprocal counterpart |
| FR-002 | The reciprocity validator shall evaluate only concrete, syntactically valid relationship entries whose targets resolve to recognized artifacts, using `related` to `related`, `supersedes` to `superseded_by`, and `superseded_by` to `supersedes` as the required counterpart mappings. | Must | US-004 / Scenarios: A missing related counterpart is reported on its source; Reciprocal supersession links resolve across historical artifacts; Missing supersession counterparts are reported on their source |
| FR-003 | When an eligible directed relationship has its source artifact ID in the target's required reverse field, the validator shall treat that relationship as reciprocal and shall report no `ARTIFACT.RELATIONSHIP.NON_RECIPROCAL` diagnostic for it, regardless of relationship order or lifecycle state. | Must | US-004 / Scenarios: Reciprocal related links resolve across current and historical artifacts; Reciprocal supersession links resolve across historical artifacts; Relationship membership ignores order and duplicate occurrences |
| FR-004 | When an eligible `related` entry lacks exact reverse membership in the target's `related` collection, the validator shall attach exactly one actionable `ARTIFACT.RELATIONSHIP.NON_RECIPROCAL` diagnostic to the source artifact, identify the source field, target ID, and expected reverse `related` link, and make the overall reciprocal-consistency result fail. | Must | US-004 / Scenario: A missing related counterpart is reported on its source |
| FR-005 | When an eligible `supersedes` or `superseded_by` entry lacks exact membership in the mapped reverse field, the validator shall attach exactly one actionable `ARTIFACT.RELATIONSHIP.NON_RECIPROCAL` diagnostic to the source artifact, identify both fields and the target ID, and make the overall reciprocal-consistency result fail. | Must | US-004 / Scenario Outline: Missing supersession counterparts are reported on their source |
| FR-006 | The validator shall compare reciprocal membership as a set: list order shall not matter, duplicate occurrences shall not create additional findings, and a reverse reference to a different artifact shall not satisfy the source entry. Each unmatched directed entry shall be evaluated independently. | Must | US-004 / Scenarios: A different reverse related link does not satisfy the original link; Multiple reciprocity failures do not suppress valid unrelated artifacts; Relationship membership ignores order and duplicate occurrences |
| FR-007 | The validator shall continue checking every recognized artifact and every eligible directed relationship after reciprocity failures, shall report one diagnostic for each unmatched directed entry, and shall retain valid unrelated artifact and relationship results. | Must | US-004 / Scenario: Multiple reciprocity failures do not suppress valid unrelated artifacts |
| FR-008 | The validator shall not add a reciprocity diagnostic for malformed, empty, unresolved, missing-target, or wrong-kind relationship values when the applicable EPIC-001 or REQ-003 diagnostic is already represented. | Must | US-004 / Scenarios: Excluded files cannot satisfy a reciprocal counterpart; Earlier structural and target failures are not duplicated |
| FR-009 | Empty `related`, `supersedes`, and `superseded_by` collections shall contain no eligible directed entries, shall produce no reciprocity diagnostic solely because they are empty, and shall not cause reciprocal-consistency failure solely because they are empty. | Must | US-004 / Scenario: Empty relationship collections do not create reciprocity failures |
| FR-010 | For the same repository state and inputs, repeated reciprocity validation shall produce the same complete ordered result, shall require no network access, and shall not modify artifact content, lifecycle status, or persisted validation results. | Must | US-004 / Scenario: Reciprocity validation is deterministic and read-only |

## Postconditions And Invariants

- Every recognized artifact remains represented exactly once in the ordered
  reciprocal-consistency result.
- Every concrete, syntactically valid relationship entry with a resolved,
  recognized target is evaluated against the required reverse field.
- Every unmatched eligible directed entry has exactly one
  `ARTIFACT.RELATIONSHIP.NON_RECIPROCAL` diagnostic attached to its source
  artifact.
- A reciprocal relationship involving an active, archived, or superseded
  artifact does not fail solely because of lifecycle state.
- A reverse membership is satisfied only by the exact source artifact ID in the
  mapped reverse field; another target ID cannot satisfy it.
- Relationship list ordering and duplicate occurrences do not change
  reciprocal membership or create duplicate findings.
- Templates and non-artifact supporting files cannot provide counterpart
  membership.
- Structural and target diagnostics remain represented and are not duplicated by
  reciprocity validation.
- Any unmatched eligible relationship produces overall `failure`; a clean or
  empty eligible relationship set produces overall `success`.
- Valid unrelated artifacts and relationships remain represented after
  reciprocity failures.
- No artifact content, lifecycle status, or persisted validation result changes.

## Edge And Failure Behavior

| Condition | Expected behavior | User-visible result |
| --- | --- | --- |
| No recognized canonical artifacts exist | Complete reciprocity validation with no eligible entries | Overall `success` and an empty result set |
| A recognized artifact has an empty relationship collection | Treat the collection as having no eligible directed entries | No reciprocity diagnostic and no failure solely for emptiness |
| A concrete `related` entry lacks exact reverse membership | Record the finding on the source and continue | One actionable `ARTIFACT.RELATIONSHIP.NON_RECIPROCAL` diagnostic and overall `failure` |
| A concrete `supersedes` entry lacks `superseded_by` membership | Apply the supersession reverse mapping and continue | One source-owned diagnostic identifying `supersedes`, target ID, and `superseded_by`; overall `failure` |
| A concrete `superseded_by` entry lacks `supersedes` membership | Apply the reverse supersession mapping and continue | One source-owned diagnostic identifying `superseded_by`, target ID, and `supersedes`; overall `failure` |
| The target's reverse field points to another artifact | Do not treat the different ID as a counterpart; evaluate each directed entry independently | The unmatched source entry receives one reciprocity diagnostic |
| The target exists only in `specs/templates/` or a supporting file | Exclude the file; do not run reciprocity validation for the unresolved entry | Existing missing-target diagnostic remains; no additional reciprocity diagnostic |
| The target is archived or marked `superseded` | Include the recognized target in the counterpart set | Reciprocity can succeed regardless of lifecycle state |
| The reverse collection contains the source ID in another position or more than once | Compare membership without order or multiplicity | The counterpart is satisfied and no duplicate reciprocity diagnostic is reported |
| A relationship value is malformed, empty, unresolved, missing-target, or wrong-kind | Leave the value to EPIC-001 or REQ-003 and skip reciprocity evaluation | Existing structural or target diagnostic remains; no duplicate reciprocity diagnostic |
| Several eligible entries are unmatched | Diagnose every unmatched directed entry and continue the complete validation | One diagnostic per unmatched entry; valid unrelated results remain; overall `failure` |
| The same repository is validated repeatedly without network access | Reuse the same recognized inputs and deterministic ordering | Identical complete ordered results and no mutation |

## Quality Requirements

- Reciprocity validation is deterministic for identical repository state and
  inputs.
- Reciprocity validation requires no network, hosted service, or AI
  dependency.
- Reciprocity validation is read-only and persists no results.
- Reciprocity validation reports all applicable unmatched entries rather than
  stopping at the first failure.
- Result and diagnostic ordering is stable and consistent with REQ-003 and the
  repository ordering contract.
- Diagnostics contain sufficient source path, source relationship field, target
  ID, required reverse field, stable rule ID, severity, message, and remediation
  context for an agent or CI check to correct the source artifact.
- The contract is suitable for pull-request validation; no exact runtime
  threshold is introduced because PRD-001 leaves that target open.

## Dependencies And Deferred Decisions

- Source story: approved `US-004`.
- Parent epic: approved `EPIC-002`.
- Recognized artifact identities and uniqueness boundary: implemented `US-002`
  and approved `REQ-002`.
- Structural recognition and malformed-value diagnostics: implemented EPIC-001
  and approved `REQ-001`.
- Relationship target resolution and expected-kind diagnostics: implemented
  `US-003` and approved `REQ-003`.
- Active and historical discovery boundaries, layered validation, and parser
  boundaries are governed by approved `ADR-005` and `ADR-006`; this contract
  does not select an implementation architecture.
- Parent, dependency, and supersession cycle detection remains a later EPIC-002
  slice.
- Schema and release relationships remain deferred until those artifact types
  and contracts exist.
- Lifecycle promotion, Spec-Ready evaluation, CLI behavior, serialization,
  persistence, file mutation, ID reservation, semantic inference, and network
  access remain out of scope.
- No runtime, external service, schema, persistence, or new dependency is
  required by this observable contract.

## Traceability

- Source story: `US-004`
- Parent epic: `EPIC-002`
- Parent PRD: `PRD-001`
- Preceding contract: `REQ-003`
- Structural validation contract: `REQ-001`
- Identity contract: `REQ-002`
- Executable scenarios: `scenarios.feature`
- Covered scenarios: Reciprocal related links resolve across current and
  historical artifacts; A missing related counterpart is reported on its
  source; A different reverse related link does not satisfy the original link;
  Reciprocal supersession links resolve across historical artifacts; Missing
  supersession counterparts are reported on their source; Multiple reciprocity
  failures do not suppress valid unrelated artifacts; Excluded files cannot
  satisfy a reciprocal counterpart; Empty relationship collections do not
  create reciprocity failures; Earlier structural and target failures are not
  duplicated; Relationship membership ignores order and duplicate occurrences;
  Reciprocity validation is deterministic and read-only.

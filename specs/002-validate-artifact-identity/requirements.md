---
id: REQ-002
title: "Repository-wide artifact identity requirements"
type: feature-requirements
status: approved
created: 2026-09-05
updated: 2026-09-05
owner: TBD
parent: US-002
depends_on: []
requires: [US-002, REQ-001]
blockers: []
related: [EPIC-002, ADR-002, ADR-003, ADR-004, ADR-005]
approval:
  approved_by: Project owner
  approved_on: 2026-09-05
---

# Requirements

## Purpose And Actors

### Purpose

Define the observable contract for deterministic, complete, offline, and
read-only identity validation across the repository's current and historical
SDD artifacts.

### Actors And External Systems

- LLM coding agent.
- CI check.
- The current repository and its local `specs/` tree as the validation source.
- No external system, network service, or AI service.

## Preconditions

- The current repository can be identified and its eligible canonical artifact
  locations can be read.
- EPIC-001 artifact recognition and structural diagnostics are available as the
  identity validation boundary.
- Active, archived, and superseded artifacts may be present, or the eligible
  identity set may be empty.
- Validation can run without network access.

## Inputs And Outputs

| Interaction | Inputs | Outputs | Validation |
| --- | --- | --- | --- |
| Validate repository artifact identities | Current repository state and recognized identities from canonical active and historical artifact files | An overall identity result and an ordered list of per-artifact results with identity diagnostics | Empty identity sets succeed; duplicate IDs and path/ID mismatches fail; excluded files are absent; source files and lifecycle statuses are unchanged |

## Identity Result Contract

- The eligible identity set includes recognized canonical artifacts in active
  locations and under `specs/archive/`, including artifacts marked
  `superseded`.
- Files under `specs/templates/` and non-artifact supporting files are excluded
  from the identity set and cannot create identity collisions.
- Each eligible recognized artifact appears exactly once in the identity result
  with its repository-relative path, recognized type when available, recognized
  ID, status, and applicable diagnostics.
- The identity result is `success` when no identity error exists, including when
  the eligible identity set is empty. It is `failure` when at least one identity
  error exists.
- Identity diagnostics use stable rule IDs in the `ARTIFACT.IDENTITY.*`
  namespace, error severity, repository-relative path, actionable message, and
  remediation guidance. Duplicate diagnostics identify the repeated ID and the
  conflicting paths. Path-identity diagnostics identify both the filename ID
  and the frontmatter ID.
- Identity diagnostics reuse the existing deterministic result ordering from
  `REQ-001`; stable rule ID is ordered before repository-relative path when
  identity findings share other ordering fields.
- Identity validation does not reparse malformed or unrecognized frontmatter.
  The path-based structural diagnostic produced by EPIC-001 remains in the
  complete result, and the file contributes no unrecognized identity.

## Functional Requirements

| ID | Requirement | Priority | Traceability (Story & Scenario) |
| --- | --- | --- | --- |
| FR-001 | The identity validator shall include recognized canonical artifacts from active locations, `specs/archive/`, and superseded artifact states, while excluding templates and supporting documents. | Must | US-002 / Scenarios: All recognized current and historical identities are valid; Empty identity set succeeds; Excluded files do not create identity collisions |
| FR-002 | The identity validator shall require every recognized artifact ID to be globally unique across the eligible identity set, regardless of lifecycle state. | Must | US-002 / Scenarios: All recognized current and historical identities are valid; Duplicate IDs are reported for every conflicting artifact |
| FR-003 | When an ID is duplicated, the identity validator shall attach an actionable duplicate-ID diagnostic to every conflicting artifact, identify the repeated ID and conflicting paths, continue checking unrelated artifacts, and return an overall failure. | Must | US-002 / Scenarios: Duplicate IDs are reported for every conflicting artifact; Identity conflicts do not suppress unrelated artifacts |
| FR-004 | The identity validator shall require the filename ID of every path-encoded PRD, epic, and ADR to match its recognized frontmatter ID, and shall report a mismatch as an actionable path-identity diagnostic without discarding the artifact. | Must | US-002 / Scenario: Path-encoded identity mismatches remain diagnosable |
| FR-005 | The identity validator shall return a successful empty result when no eligible recognized artifact identities exist and shall return one result per eligible recognized artifact when identities exist. | Must | US-002 / Scenarios: All recognized current and historical identities are valid; Empty identity set succeeds |
| FR-006 | The identity validator shall preserve EPIC-001 path-based structural diagnostics for canonical artifacts with malformed or unrecognized frontmatter and shall not reparse those artifacts for identity validation. | Must | US-002 / Scenario: Malformed frontmatter remains an EPIC-001 structural diagnostic |
| FR-007 | The identity validator shall retain valid unrelated artifact results and continue processing after duplicate-ID or path-identity findings. | Must | US-002 / Scenarios: Duplicate IDs are reported for every conflicting artifact; Path-encoded identity mismatches remain diagnosable; Identity conflicts do not suppress unrelated artifacts |
| FR-008 | The identity validator shall produce the same ordered result for the same repository state and inputs without modifying artifact content, lifecycle status, or persisted results. | Must | US-002 / Scenario: Identity validation is deterministic and read-only |

## Postconditions And Invariants

- Every eligible recognized artifact is represented exactly once in the identity
  result.
- No template or supporting document is represented as an eligible identity or
  creates a collision.
- Every duplicate ID is represented by a diagnostic on each conflicting path.
- Every path-encoded filename/frontmatter mismatch is represented by a
  diagnostic without silently choosing one identity.
- A clean or empty eligible identity set produces overall `success`.
- Any duplicate or path-identity error produces overall `failure`.
- Unrelated artifacts remain represented after identity errors.
- Upstream structural diagnostics remain attributable to their source paths.
- No artifact content, lifecycle status, or persisted validation result changes.

## Edge And Failure Behavior

| Condition | Expected behavior | User-visible result |
| --- | --- | --- |
| No eligible recognized artifact identities exist | Complete identity validation without error | Overall `success` and an empty identity result list |
| A recognized ID is repeated across active, archived, or superseded artifacts | Record the collision for every conflicting artifact and continue | Each conflicting path has an actionable duplicate-ID diagnostic and overall `failure` |
| A PRD, epic, or ADR filename ID differs from its recognized frontmatter ID | Retain the artifact and continue identity validation | The artifact has an actionable path-identity diagnostic and overall `failure` |
| A template or supporting file contains a duplicate-looking ID | Exclude the file from identity collection | No identity result or collision is created by the excluded file |
| Identity errors coexist with valid unrelated artifacts | Continue processing all eligible artifacts | Valid unrelated results remain present and free of identity diagnostics |
| Canonical frontmatter is malformed or unrecognized | Preserve the EPIC-001 structural path diagnostic without reparsing | The file remains diagnosable by path and contributes no recognized identity |

## Quality Requirements

- Identity validation is deterministic for identical repository state and inputs.
- Identity validation requires no network, hosted service, or AI dependency.
- Identity validation is read-only and persists no results.
- Identity validation reports all applicable identity findings rather than
  stopping at the first error.
- Identity result ordering and diagnostic ordering are stable and reuse the
  established `REQ-001` contract.
- Diagnostics contain enough path, identity, rule, severity, message, and
  remediation context for an agent or CI check to correct the source artifact.
- The contract is suitable for pull-request validation; no exact runtime
  threshold is introduced here because PRD-001 leaves that target open.

## Dependencies And Deferred Decisions

- Source story: approved `US-002`.
- Parent epic: approved `EPIC-002`.
- Structural recognition and diagnostic boundary: implemented `EPIC-001` and
  approved `REQ-001`.
- Relationship target resolution, expected target kinds, reciprocal links, and
  relationship cycles remain deferred to a later EPIC-002 story.
- Schema and release relationship validation remains deferred until those
  artifact contracts exist.
- Lifecycle promotion and Spec-Ready evaluation remain deferred to EPIC-003.
- CLI commands, serialization, and CI-provider integration remain deferred to
  EPIC-004.

## Traceability

- Source story: `US-002`
- Parent epic: `EPIC-002`
- Parent PRD: `PRD-001`
- Preceding contract: `REQ-001`
- Executable scenarios: `scenarios.feature`
- Covered scenarios: All recognized current and historical identities are valid;
  Empty identity set succeeds; Duplicate IDs are reported for every conflicting
  artifact; Path-encoded identity mismatches remain diagnosable; Excluded files
  do not create identity collisions; Identity conflicts do not suppress
  unrelated artifacts; Malformed frontmatter remains an EPIC-001 structural
  diagnostic; Identity validation is deterministic and read-only.

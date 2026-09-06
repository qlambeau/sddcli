---
id: REQ-005
title: "Relationship cycle detection requirements"
type: feature-requirements
status: approved
created: 2026-09-07
updated: 2026-09-07
owner: TBD
parent: US-005
depends_on: [REQ-004]
requires: [US-005, REQ-004]
blockers: []
related: [PRD-001, EPIC-002, ADR-002, ADR-003, ADR-004, ADR-005, ADR-006, REQ-001, REQ-002, REQ-003, REQ-004]
approval:
  approved_by: Project owner
  approved_on: 2026-09-07
---

# Requirements

<!-- Requirements describe externally observable behavior. Architecture and
implementation sequencing belong in design.md and tasks.md. -->

## Purpose And Actors

### Purpose

Define the observable contract for detecting distinct directed cycles in parent,
dependency, and supersession relationships across the complete recognized
repository artifact set.

### Actors And External Systems

- LLM coding agent.
- CI check.
- The current repository and its local `specs/` tree as the validation source.
- No external system, network service, or AI service.

## Preconditions

- The current repository can be identified and its eligible canonical artifact
  locations can be read.
- EPIC-001 structural validation and its diagnostics are available.
- REQ-002 identity validation provides the recognized artifact identities.
- REQ-003 relationship target validation provides the recognized artifact set,
  normalized relationship entries, expected-kind results, and structural or
  target diagnostics.
- REQ-004 reciprocal validation provides any applicable non-reciprocal
  diagnostics for the same recognized artifact set.
- The recognized artifact set includes active artifacts and eligible artifacts
  under `specs/archive/`, including artifacts marked `superseded`.
- Files under `specs/templates/` and non-artifact supporting files are excluded
  from the recognized artifact set.
- Validation can run without network access and without mutating the
  repository.

## Inputs And Outputs

| Interaction | Inputs | Outputs | Validation |
| --- | --- | --- | --- |
| Validate relationship cycles | Current repository state, the complete recognized artifact set, normalized relationship entries, and prior structural, target, and reciprocity diagnostics | An ordered per-artifact result set containing the recognized artifacts, existing results, cycle diagnostics, and an overall cycle result | Build independent parent, dependency, and supersession graphs from concrete, resolved, expected-kind-valid entries; preserve prior diagnostics and exclude ineligible values |

## Cycle Detection Contract

- The recognized artifact set is the complete active and historical set
  established by REQ-002 and REQ-003. Recognized artifacts participate regardless
  of active, archived, or `superseded` lifecycle state.
- Templates and non-artifact supporting files cannot provide graph nodes or
  relationship edges.
- Cycle validation builds three independent directed graphs:
  - The parent graph contains eligible `parent` entries.
  - The dependency graph contains eligible `depends_on`, `requires`, and
    `blockers` entries.
  - The supersession graph contains eligible `supersedes` and `superseded_by`
    entries.
- A supersession `supersedes` entry is directed from the superseding artifact to
  the referenced superseded artifact.
- A supersession `superseded_by` entry is normalized to the same logical
  superseder-to-superseded direction: the referenced artifact is the superseder
  and the source artifact is the superseded artifact.
- Edges from different relationship families are never combined into one graph
  or cycle.
- A relationship entry is eligible for cycle detection only when it is concrete,
  syntactically valid, resolves to a recognized artifact, and satisfies its
  expected target-kind rule.
- Malformed, empty, unresolved, missing-target, and wrong-kind values remain
  represented by their existing structural or target diagnostics and are not
  included as cycle edges.
- A target-valid relationship from an artifact to itself is a one-edge cycle.
  A wrong-kind self-loop is not cycle-eligible.
- A non-reciprocal diagnostic does not remove an otherwise target-valid
  relationship entry from cycle detection.
- A distinct cycle is a distinct directed cycle in one relationship-family
  graph. The same directed cycle reached from different starting artifacts or
  traversal rotations is one cycle; different directed edge sets are different
  cycles.
- Cycle identity is stable for equivalent directed cycles and distinguishes
  different relationship families and directed edge sets.
- Cycle validation identifies every distinct eligible cycle, including distinct
  overlapping cycles.
- Each distinct cycle produces one source-owned cycle diagnostic for every
  relationship entry participating in that cycle. An entry participating in
  multiple distinct cycles receives one diagnostic for each cycle.
- A cycle diagnostic identifies the relationship family, source artifact and
  relationship entry, target information, and canonical cycle identity. Its
  human-readable wording and remediation context are actionable but not fixed
  by this contract.
- Cycle validation continues after findings and preserves valid unrelated
  artifacts, relationships, and prior diagnostics.
- The cycle-validation result is `success` when no eligible cycle exists and
  `failure` when at least one eligible cycle exists.

## Functional Requirements

| ID | Requirement | Priority | Traceability (Story & Scenario) |
| --- | --- | --- | --- |
| FR-001 | The cycle validator shall use the complete recognized canonical artifact set from active locations and `specs/archive/`, including artifacts marked `superseded`, and shall exclude templates and non-artifact supporting files from graph construction. | Must | US-005 / Scenarios: A valid repository with no cycles succeeds; Historical recognized artifacts participate while excluded files do not; Cycle findings do not suppress valid unrelated artifacts |
| FR-002 | The cycle validator shall construct and evaluate parent, dependency, and supersession relationships as independent directed graph families and shall not combine edges from different families into one cycle. | Must | US-005 / Scenarios: An eligible parent cycle is reported within the parent graph; Each dependency relationship field participates in cycle detection; Supersession fields are normalized to one logical direction; Different relationship families do not form a cycle together |
| FR-003 | The cycle validator shall evaluate eligible `parent` entries as directed edges in the parent graph and shall report every distinct eligible parent cycle. | Must | US-005 / Scenario: An eligible parent cycle is reported within the parent graph |
| FR-004 | The cycle validator shall evaluate eligible `depends_on`, `requires`, and `blockers` entries as directed edges in the dependency graph and shall report cycles formed by any of those fields. | Must | US-005 / Scenario Outline: Each dependency relationship field participates in cycle detection |
| FR-005 | The cycle validator shall normalize `supersedes` and `superseded_by` entries to logical superseder-to-superseded edges before evaluating supersession cycles. | Must | US-005 / Scenario: Supersession fields are normalized to one logical direction |
| FR-006 | The cycle validator shall include only concrete, syntactically valid, resolved, expected-kind-valid relationship entries and shall leave malformed, empty, unresolved, missing-target, and wrong-kind values to their existing diagnostics. | Must | US-005 / Scenarios: A target-valid self-loop is reported; A wrong-kind self-loop is not duplicated as a cycle; Ineligible relationship values retain existing diagnostics without cycle findings |
| FR-007 | The cycle validator shall report an eligible self-loop as a cycle and shall not add a cycle diagnostic for a self-loop that is excluded by the expected-kind rule. | Must | US-005 / Scenarios: A target-valid self-loop is reported; A wrong-kind self-loop is not duplicated as a cycle |
| FR-008 | The cycle validator shall identify the same directed cycle only once when traversal reaches it through different starting artifacts or rotations, while preserving distinct cycle identities for different directed edge sets. | Must | US-005 / Scenarios: An eligible parent cycle is reported within the parent graph; A directed cycle is reported once regardless of traversal rotation; Distinct overlapping cycles receive distinct findings |
| FR-009 | The cycle validator shall report every distinct eligible overlapping cycle rather than collapsing cycles that share one or more relationship entries. | Must | US-005 / Scenario: Distinct overlapping cycles receive distinct findings |
| FR-010 | For every distinct cycle, the validator shall attach one cycle diagnostic to each participating source relationship entry, identify the source and canonical cycle identity, and attach one diagnostic per cycle when an entry participates in multiple distinct cycles. | Must | US-005 / Scenarios: An eligible parent cycle is reported within the parent graph; Each dependency relationship field participates in cycle detection; Supersession fields are normalized to one logical direction; A target-valid self-loop is reported; A directed cycle is reported once regardless of traversal rotation; Distinct overlapping cycles receive distinct findings; A target-valid non-reciprocal relationship remains eligible |
| FR-011 | The cycle validator shall preserve existing structural, target, and non-reciprocal diagnostics and shall not duplicate them with cycle diagnostics unless the relationship is target-valid and participates in an eligible cycle. | Must | US-005 / Scenarios: A wrong-kind self-loop is not duplicated as a cycle; Ineligible relationship values retain existing diagnostics without cycle findings; A target-valid non-reciprocal relationship remains eligible |
| FR-012 | The cycle validator shall continue checking all recognized artifacts and eligible relationship entries after cycle findings and shall retain valid unrelated artifact and relationship results. | Must | US-005 / Scenarios: A valid repository with no cycles succeeds; Historical recognized artifacts participate while excluded files do not; Cycle findings do not suppress valid unrelated artifacts |
| FR-013 | The cycle-validation result shall be `success` when no eligible cycle exists and `failure` when one or more eligible cycles exist, while representing all applicable cycle diagnostics in either case. | Must | US-005 / Scenarios: A valid repository with no cycles succeeds; An eligible parent cycle is reported within the parent graph; Each dependency relationship field participates in cycle detection; Supersession fields are normalized to one logical direction; A target-valid self-loop is reported; Different relationship families do not form a cycle together; Cycle findings do not suppress valid unrelated artifacts |
| FR-014 | For identical repository state and inputs, repeated cycle validation shall produce the same complete ordered result, require no network access, and modify no artifact content, lifecycle status, or persisted validation result. | Must | US-005 / Scenario: Cycle validation is deterministic and read-only |

## Postconditions And Invariants

- Every recognized artifact is represented exactly once in the ordered cycle
  result.
- Every graph edge belongs to exactly one relationship family.
- Every graph edge is backed by a concrete, syntactically valid, resolved,
  expected-kind-valid relationship entry.
- No malformed, empty, unresolved, missing-target, or wrong-kind value creates a
  cycle edge or cycle diagnostic.
- No template or supporting file provides a graph node or edge.
- Every distinct eligible cycle is represented exactly once by its canonical
  cycle identity.
- Every participating source relationship entry receives exactly one cycle
  diagnostic for each distinct cycle in which it participates.
- Cycle diagnostics remain attached to their source artifact and identify the
  relationship family and cycle identity.
- A target-valid entry with a non-reciprocal diagnostic remains eligible for
  cycle detection.
- Valid unrelated artifacts and relationships remain represented after cycle
  findings.
- The cycle result fails if and only if at least one eligible cycle is found.
- Repeated validation does not change artifact content, lifecycle status, or
  persisted validation results.

## Edge And Failure Behavior

| Condition | Expected behavior | User-visible result |
| --- | --- | --- |
| No recognized canonical artifacts exist | Evaluate an empty eligible graph set | Overall cycle result `success` and an empty recognized-artifact result set |
| Recognized artifacts contain valid relationships but no cycle in any family | Complete all graph checks | No cycle diagnostic and overall cycle result `success` |
| Eligible parent relationships form a directed cycle | Evaluate only the parent graph and continue all other checks | One source-owned cycle diagnostic per participating parent entry and overall cycle result `failure` |
| An eligible `depends_on`, `requires`, or `blockers` path closes | Evaluate the dependency graph | One diagnostic per participating dependency entry for the distinct cycle and overall cycle result `failure` |
| Eligible `supersedes` and `superseded_by` entries close after normalization | Evaluate the logical supersession graph | One diagnostic per participating supersession entry and overall cycle result `failure` |
| A target-valid relationship points to its own source artifact | Treat the entry as a one-edge cycle | One cycle diagnostic on the source entry and overall cycle result `failure` |
| A self-loop violates the expected target-kind rule | Exclude the entry from graph construction | Existing wrong-kind diagnostic remains; no cycle diagnostic is added |
| A path closes only by combining relationship families | Keep each family in its own graph | No cycle diagnostic for the mixed-family path |
| The same cycle is reached through multiple traversal rotations | Use one canonical cycle identity | One cycle finding rather than one finding per rotation |
| An entry belongs to multiple distinct overlapping cycles | Preserve each distinct directed cycle identity | One diagnostic for each applicable cycle on the shared entry |
| A relationship is malformed, empty, unresolved, missing-target, or wrong-kind | Leave the value to EPIC-001 or REQ-003 and skip cycle evaluation | Existing structural or target diagnostic remains; no cycle diagnostic is added |
| A target-valid relationship has a non-reciprocal diagnostic | Include the relationship in its graph | Existing non-reciprocal diagnostic remains and a cycle diagnostic is added when the entry participates in a cycle |
| A cycle includes active, archived, and/or superseded recognized artifacts | Include all recognized lifecycle states | The applicable cycle is reported regardless of lifecycle state |
| A referenced ID exists only in a template or supporting file | Exclude the file from graph construction | Existing missing-target result remains; the excluded file cannot provide a cycle edge |
| Cycle findings coexist with valid unrelated artifacts | Continue complete validation | Cycle diagnostics are retained and valid unrelated results remain represented |
| The same repository is validated repeatedly without network access | Reuse the same recognized inputs and stable ordering | Identical complete ordered results and no repository mutation |

## Quality Requirements

- Cycle validation is deterministic for identical repository state and inputs,
  including stable artifact, diagnostic, and cycle ordering.
- Cycle validation is complete: it reports every distinct eligible cycle and
  does not stop after the first finding.
- Cycle validation requires no network, hosted service, or AI dependency.
- Cycle validation is read-only and persists no validation results.
- Cycle diagnostics use the stable rule identifier
  `ARTIFACT.RELATIONSHIP.CYCLE`, error severity, source path, source relationship
  field, target information, and canonical cycle identity.
- Cycle diagnostics provide actionable message and remediation context, while
  their exact human-readable wording remains outside this contract.
- The result preserves the deterministic ordering contract established by
  REQ-003 and REQ-004.
- The contract is suitable for pull-request validation; no exact runtime
  threshold is introduced because PRD-001 leaves that target open.

## Dependencies And Deferred Decisions

- Source story: approved `US-005`.
- Parent epic: approved `EPIC-002`.
- Parent PRD: approved `PRD-001`.
- Structural recognition and malformed-value diagnostics: implemented EPIC-001
  and approved `REQ-001`.
- Repository-wide identity discovery: implemented `US-002` and approved
  `REQ-002`.
- Relationship target resolution and expected-kind diagnostics: implemented
  `US-003` and approved `REQ-003`.
- Reciprocal relationship diagnostics and the recognized historical artifact
  boundary: implemented `US-004` and approved `REQ-004`.
- Layered validation, parser boundaries, and source discovery remain governed by
  approved `ADR-005` and `ADR-006`; this contract does not select an
  implementation architecture.
- Exact human-readable cycle message and remediation wording remain a
  non-blocking deferred decision.
- Schema and release relationship cycles remain deferred until those artifact
  types and contracts exist.
- Lifecycle promotion, Spec-Ready evaluation, CLI behavior, serialization,
  persistence, file mutation, ID reservation, semantic inference, and network
  access remain out of scope.
- No runtime, external service, schema, persistence, or new dependency is
  required by this observable contract.

## Traceability

- Source story: `US-005`.
- Parent epic: `EPIC-002`.
- Parent PRD: `PRD-001`.
- Preceding contract: `REQ-004`.
- Relationship target contract: `REQ-003`.
- Identity contract: `REQ-002`.
- Structural validation contract: `REQ-001`.
- Executable scenarios: `scenarios.feature`.
- Covered scenarios: A valid repository with no cycles succeeds; An eligible
  parent cycle is reported within the parent graph; Each dependency relationship
  field participates in cycle detection; Supersession fields are normalized to
  one logical direction; A target-valid self-loop is reported; A wrong-kind
  self-loop is not duplicated as a cycle; Different relationship families do not
  form a cycle together; A directed cycle is reported once regardless of
  traversal rotation; Distinct overlapping cycles receive distinct findings;
  Ineligible relationship values retain existing diagnostics without cycle
  findings; A target-valid non-reciprocal relationship remains eligible;
  Historical recognized artifacts participate while excluded files do not; Cycle
  findings do not suppress valid unrelated artifacts; Cycle validation is
  deterministic and read-only.

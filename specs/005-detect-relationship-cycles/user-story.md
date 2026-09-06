---
id: US-005
title: "Detect relationship cycles"
type: user-story
status: approved
created: 2026-09-06
updated: 2026-09-06
owner: TBD
parent: PRD-001
epic: EPIC-002
feature: 005-detect-relationship-cycles
depends_on: [US-004]
requires: [EPIC-002]
blockers: []
related: []
approval:
  approved_by: Project owner
  approved_on: 2026-09-06
---

# User Story

## Story Card

As an LLM coding agent or CI check, I want cycles detected in the repository's
parent, dependency, and supersession relationships, so that invalid traceability
graphs are identified before downstream workflow actions.

## Context And Value

Identity and relationship validation already detects duplicate identities,
broken relationship targets, and asymmetric links. A graph can still contain a
closed path that makes parentage, dependency ordering, or supersession history
ambiguous or impossible to resolve. These cycles weaken traceability and can
cause agents or CI to revisit the same artifacts indefinitely.

Detecting cycles across the recognized repository artifact set gives agents and
CI an actionable integrity result while preserving valid unrelated artifacts
and the existing structural, target, and reciprocity diagnostics.

## Business Rules

- Parent, dependency, and supersession relationships are evaluated as three
  independent directed graphs. Edges from different relationship families do
  not combine to form a cycle.
- The dependency graph includes `depends_on`, `requires`, and `blockers`
  relationships.
- The supersession graph normalizes both `supersedes` and `superseded_by` to the
  logical direction from the superseding artifact to the superseded artifact.
- A relationship is eligible for cycle detection when it is concrete,
  syntactically valid, resolves to a recognized artifact, and satisfies the
  expected target-kind rule.
- Malformed, empty, unresolved, missing-target, and wrong-kind values remain
  governed by the existing structural and target validators and do not receive
  cycle diagnostics.
- A relationship that has a non-reciprocal diagnostic may still participate in
  cycle detection when its target is otherwise valid and resolved.
- A relationship from an artifact to itself is a cycle.
- The same directed cycle encountered from different starting artifacts or
  rotations is one distinct cycle.
- Cycles with different directed edge sets are distinct cycles.
- Each distinct cycle produces one source-owned diagnostic for every
  relationship entry participating in that cycle.
- When an entry participates in multiple distinct overlapping cycles, it
  receives one diagnostic for each cycle.
- Validation continues after cycle findings and retains valid unrelated results.
- Cycle validation is deterministic, offline, complete, and read-only.

## Examples

| Example | Given | When | Expected outcome |
| --- | --- | --- | --- |
| EX-001 | Recognized artifacts contain parent, dependency, and supersession relationships with no closed path in any family | Cycle validation runs | The overall result succeeds and no cycle diagnostic is reported |
| EX-002 | Artifacts A, B, and C form `A depends_on B`, `B depends_on C`, and `C depends_on A` | Cycle validation runs | One cycle is identified and each participating dependency entry receives one source-owned diagnostic |
| EX-003 | Artifact A has a parent relationship to itself | Cycle validation runs | The self-loop is reported as a parent cycle on the participating entry |
| EX-004 | Active artifact A supersedes archived artifact B and the logical supersession path returns to A | Cycle validation runs | The supersession cycle is reported regardless of lifecycle state |
| EX-005 | A path combines a dependency edge with a `related` edge but does not close within one relationship family | Cycle validation runs | No cycle is reported for the mixed-family path |
| EX-006 | Two dependency cycles share one relationship entry | Cycle validation runs | Each distinct cycle is reported, and the shared entry receives one diagnostic per cycle |
| EX-007 | A relationship value is malformed, unresolved, missing-target, or wrong-kind | Cycle validation runs | The existing structural or target diagnostic remains and no cycle diagnostic is added |
| EX-008 | Some artifacts contain cycles while other recognized artifacts have valid relationships | Cycle validation runs | Cycle findings are retained and valid unrelated artifact results remain represented |
| EX-009 | The same repository state is validated twice | Cycle validation runs | Both ordered results are identical and no artifact content or lifecycle state changes |

## Acceptance Criteria

- Given recognized artifacts with parent relationships, when cycle validation
  runs, then cycles are detected within the parent graph without combining it
  with another relationship family.
- Given recognized artifacts with `depends_on`, `requires`, or `blockers`
  relationships, when cycle validation runs, then cycles are detected within
  the dependency graph.
- Given recognized artifacts with `supersedes` or `superseded_by` relationships,
  when cycle validation runs, then both fields are interpreted in logical
  superseder-to-superseded direction for the supersession graph.
- Given a relationship from an artifact to itself, when cycle validation runs,
  then the self-loop receives a cycle diagnostic.
- Given a distinct directed cycle, when cycle validation runs, then each
  participating relationship entry receives one source-owned cycle diagnostic.
- Given an entry participating in multiple distinct overlapping cycles, when
  cycle validation runs, then that entry receives one diagnostic per cycle.
- Given the same directed cycle is encountered from different starting points,
  when cycle validation runs, then it is reported once rather than once per
  traversal rotation.
- Given malformed, empty, unresolved, missing-target, or wrong-kind values,
  when cycle validation runs, then existing structural or target diagnostics
  remain and no cycle diagnostic is added for those values.
- Given a target-valid relationship with a non-reciprocal diagnostic, when cycle
  validation runs, then the relationship remains eligible for cycle detection.
- Given active, archived, and superseded recognized artifacts, when cycle
  validation runs, then all recognized lifecycle states participate and
  templates and supporting files cannot provide graph edges.
- Given cycles and valid unrelated artifacts in the same repository, when cycle
  validation runs, then every applicable cycle is reported and valid unrelated
  results remain represented.
- Given no eligible cycles, when cycle validation runs, then the cycle result
  succeeds; given one or more eligible cycles, then it fails.
- Given the same repository state and inputs, when cycle validation runs twice,
  then both complete ordered results are identical and no repository content,
  lifecycle status, or persisted result changes.

## Scope Boundaries

### In Scope

- Parent cycle detection.
- Dependency cycle detection for `depends_on`, `requires`, and `blockers`.
- Supersession cycle detection for `supersedes` and `superseded_by`.
- Independent relationship-family graphs.
- Self-loops, overlapping cycles, and rotation-stable cycle identity.
- Active, archived, and superseded recognized artifacts.
- Complete source-owned diagnostics, deterministic ordering, and read-only
  validation.

### Out Of Scope

- Cycles formed by combining different relationship families.
- Schema and release relationship cycles until those contracts exist.
- Relationship target validation and reciprocal validation as separate rules.
- CLI command behavior, machine-readable serialization, and CI-provider
  integration.
- Lifecycle promotion, Spec-Ready evaluation, persistence, file mutation,
  network access, semantic inference, and automatic cycle repair.

## Dependencies

- Implemented `US-004` provides the recognized active and historical artifact
  set, structural validation boundary, relationship target resolution, and
  reciprocal relationship results.
- Implemented `US-003` provides recognized relationship targets and expected-kind
  diagnostics used to establish cycle eligibility.
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
| OQ-001 | What exact human-readable wording and remediation context should cycle diagnostics use? | No | Project owner | Open |

## INVEST Check

- [x] Independent
- [x] Negotiable
- [x] Valuable
- [x] Estimable
- [x] Small enough for roughly 1 to 3 days
- [x] Testable

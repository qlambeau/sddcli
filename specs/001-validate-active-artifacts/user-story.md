---
id: US-001
title: "Validate active SDD artifacts"
type: user-story
status: approved
created: 2026-09-03
updated: 2026-09-03
owner: TBD
parent: PRD-001
epic: EPIC-001
feature: 001-validate-active-artifacts
depends_on: []
requires: [EPIC-001]
blockers: []
related: []
approval:
  approved_by: Project owner
  approved_on: 2026-09-03
---

# User Story

## Story Card

As an LLM coding agent or CI check, I want to validate all active SDD artifacts
and see each artifact's validation status plus every violation, so that I can
identify and correct invalid documents before downstream review or use.

## Context And Value

Active SDD artifacts must be structurally valid before they can be reviewed or
consumed by later workflow steps. A complete per-artifact result lets agents and
automated checks locate every problem in one read-only validation pass instead
of discovering failures one at a time.

## Business Rules

- The active set includes PRDs, epic briefs, user stories, scenarios,
  requirements, designs, tasks, and ADRs.
- Files under `specs/templates/` and `specs/archive/` are excluded from the
  active set.
- Every active artifact appears exactly once with validation status `ok` or
  `diagnostic`.
- Every violation for an artifact is reported as a separate list entry.
- Malformed or unrecognized frontmatter still produces a path-based
  `diagnostic`.
- Any artifact with status `diagnostic` makes the overall validation fail.
- Validation is deterministic, offline, and strictly read-only.
- Each diagnostic includes the artifact path, location when available, rule ID,
  severity, message, and remediation guidance.

## Examples

| Example | Given | When | Expected outcome |
| --- | --- | --- | --- |
| EX-001 | All active artifacts satisfy their type-specific rules | Validation runs | Every active artifact is listed once with status `ok`, and the overall result succeeds |
| EX-002 | One active artifact violates a rule | Validation runs | That artifact is listed with status `diagnostic`, its violation is reported, and the overall result fails |
| EX-003 | One active artifact has multiple violations | Validation runs | Each violation appears as a separate entry for that artifact with the required diagnostic context |
| EX-004 | An active file has malformed or unrecognized frontmatter | Validation runs | The file is listed by path with status `diagnostic` and the parsing or recognition violation |
| EX-005 | Matching files exist under `specs/templates/` or `specs/archive/` | Validation runs | Those files are not listed as active artifacts |

## Acceptance Criteria

- Given valid active artifacts of all eight supported types, when validation runs,
  then each artifact is listed exactly once with status `ok` and the overall
  result succeeds.
- Given one or more invalid active artifacts, when validation runs, then each
  affected artifact is listed with status `diagnostic` and the overall result
  fails.
- Given an artifact with multiple violations, when validation runs, then every
  violation is reported as a separate list entry with path, location when
  available, rule ID, severity, message, and remediation guidance.
- Given malformed or unrecognized frontmatter, when validation runs, then the
  file is still listed by path with status `diagnostic`.
- Given files under `specs/templates/` or `specs/archive/`, when validation runs,
  then those files are excluded from the active artifact result.
- Given the same repository state and inputs, repeated validation produces the
  same result without network access or file mutation.

## Scope Boundaries

### In Scope

- Structural and type-specific validation for all eight active artifact types.
- Per-artifact statuses and complete violation lists.
- Actionable diagnostic context.

### Out Of Scope

- Global ID uniqueness, collision detection, and cross-artifact relationship
  resolution.
- Lifecycle promotion and Spec-Ready evaluation.
- File mutation, persisted validation results, CLI serialization, and
  CI-provider integration.

## Dependencies

- Approved `EPIC-001` and `PRD-001`.
- No runtime or feature dependencies.

## Open Questions

No unresolved or blocking questions.

## INVEST Check

- [x] Independent
- [x] Negotiable
- [x] Valuable
- [x] Estimable
- [x] Small enough for roughly 1 to 3 days
- [x] Testable

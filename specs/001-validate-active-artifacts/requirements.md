---
id: REQ-001
title: "Active artifact validation requirements"
type: feature-requirements
status: approved
created: 2026-09-03
updated: 2026-09-03
owner: Project owner
parent: US-001
depends_on: []
requires: [US-001]
blockers: []
related: [ADR-002, ADR-003, ADR-004]
approval:
  approved_by: Project owner
  approved_on: 2026-09-03
---

# Requirements

## Purpose And Actors

### Purpose

Define the observable contract for deterministic, complete, offline, and
read-only validation of active SDD artifacts.

### Actors And External Systems

- LLM coding agent.
- CI check.
- The current repository and its local `specs/` tree as the validation source.

## Preconditions

- The current repository can be identified and its canonical artifact locations
  can be read.
- Validation can run without network access or an AI service.
- The active artifact set may be empty.

## Inputs And Outputs

| Interaction | Inputs | Outputs | Validation |
| --- | --- | --- | --- |
| Validate active artifact set | Current repository state and files at canonical active artifact locations | Overall result and an ordered list of per-artifact results | Empty active sets succeed; invalid artifacts produce diagnostics; no files or persisted results are changed |

## Validation Result Contract

- The overall result is `success` when the artifact result list contains no
  `diagnostic` status; otherwise it is `failure`.
- Each artifact result contains its repository-relative path, recognized type
  when available, ID when available, status (`ok` or `diagnostic`), and a list
  of violations.
- The artifact result list uses this fixed type order: PRD, EPIC, user story,
  Gherkin, requirements, design, ADR, TASK.
- Artifacts within a type are sorted by ascending ID.
- Artifacts without valid IDs appear after identified artifacts and are sorted by
  repository-relative path.
- Violations are sorted by stable rule ID and location.
- Each violation contains the repository-relative path, location when available,
  a stable rule ID, severity, message, and remediation guidance.
- Rule IDs use the stable namespace `ARTIFACT` dot type token dot rule token.
- A warning retains severity `warning`, but is treated as an error for the
  artifact status and overall result.
- An empty active artifact set produces `success` with an empty artifact list.

## Artifact Schema Rules

### Canonical Artifact Locations And Required Metadata

| Artifact type | Canonical location | Required metadata or headers |
| --- | --- | --- |
| PRD | `specs/prds/PRD-` followed by three digits and `.md` | `id`, `title`, `type`, `scope`, `status`, `created`, `updated`, `owner`, `parent`, `supersedes`, `related` |
| EPIC | `specs/prds/PRD-` followed by three digits, `-epics/EPIC-` followed by three digits, and `.md` | `id`, `title`, `type`, `status`, `created`, `updated`, `owner`, `parent`, `depends_on`, `requires`, `blockers`, `related` |
| User story | `specs/` feature packet directory and `user-story.md` | `id`, `title`, `type`, `status`, `created`, `updated`, `owner`, `parent`, `epic`, `feature`, `depends_on`, `requires`, `blockers`, `related` |
| Gherkin | `specs/` feature packet directory and `scenarios.feature` | A `# parent: US-` header followed by three digits and a `# status:` header containing an allowed lifecycle value |
| Requirements | `specs/` feature packet directory and `requirements.md` | `id`, `title`, `type`, `status`, `created`, `updated`, `owner`, `parent`, `depends_on`, `requires`, `blockers`, `related` |
| Design | `specs/` feature packet directory and `design.md` | `id`, `title`, `type`, `status`, `created`, `updated`, `owner`, `parent`, `depends_on`, `requires`, `blockers`, `related` |
| ADR | `specs/adr/ADR-` followed by three digits and `.md` | `id`, `title`, `type`, `status`, `created`, `updated`, `owner`, `supersedes`, `superseded_by`, `related` |
| TASK | `specs/` feature packet directory and `tasks.md` | `id`, `title`, `type`, `status`, `created`, `updated`, `owner`, `parent`, `depends_on`, `requires`, `blockers`, `related` |

### Required Document Structure

- PRDs require the product requirements heading, vision and problem, personas
  and journeys, success metrics, functional scope and epics, non-functional
  requirements, assumptions and out-of-scope boundaries, open questions,
  decision log, and review checklist sections.
- Epic briefs require the epic brief heading, outcome statement, capability
  boundaries, candidate vertical slices, success criteria, dependencies, open
  questions, and readiness checklist sections.
- User stories require the user story heading, story card, context and value,
  business rules, examples, acceptance criteria, scope boundaries, dependencies,
  open questions, and INVEST check sections.
- Gherkin files require one `Feature`, at least one `Scenario` or `Scenario
  Outline`, and behavioral `Given`, `When`, and `Then` steps.
- Requirements documents require purpose and actors, preconditions, inputs and
  outputs, functional requirements, postconditions and invariants, edge and
  failure behavior, quality requirements, and traceability sections.
- Design documents require context and constraints, proposed design, components
  and responsibilities, interfaces and contracts, data and state flow, security
  performance and operations, alternatives considered, risks and open
  decisions, and verification approach sections.
- Task documents require implementation approach, ordered tasks, test and
  verification plan, rollout and recovery, and definition of done sections.
- ADRs require context, decision, alternatives considered, consequences, and
  follow-up actions sections.
- Required checklist headings and entries must be present; their checked state
  is not a structural validation condition.

## Functional Requirements

| ID | Requirement | Priority | Traceability (Story & Scenario) |
| --- | --- | --- | --- |
| FR-001 | The validator shall discover only supported artifacts at the canonical active locations and exclude supporting documents, templates, and archives. | Must | US-001 / Scenarios: All active artifact types are valid; Template and archive files are excluded; Empty active artifact set succeeds |
| FR-002 | The validator shall enforce each supported type's required metadata, headers, lifecycle status values, document sections, checklist sections, and local ID/reference syntax. | Must | US-001 / Scenarios: All active artifact types are valid; Malformed or unrecognized frontmatter remains diagnosable |
| FR-003 | The validator shall report the reserved unresolved-value marker and unreplaced template markers as violations wherever they occur in active artifact metadata or normative content. | Must | US-001 / Scenarios: Invalid active artifacts produce a failed result; Multiple violations are listed separately |
| FR-004 | The validator shall produce one result per active artifact with status `ok` or `diagnostic`, and shall produce an overall success or failure result according to those statuses. | Must | US-001 / Scenarios: All active artifact types are valid; Invalid active artifacts produce a failed result; Empty active artifact set succeeds |
| FR-005 | The validator shall collect every applicable violation for each artifact and report each violation as a separate list entry with the required diagnostic fields. | Must | US-001 / Scenarios: Invalid active artifacts produce a failed result; Multiple violations are listed separately |
| FR-006 | The validator shall list a canonical active file with malformed or unrecognized frontmatter by path with status `diagnostic`, even when its type or ID cannot be recognized. | Must | US-001 / Scenario: Malformed or unrecognized frontmatter remains diagnosable |
| FR-007 | The validator shall order artifact results by the fixed type order, then ascending ID, placing no-ID results afterward in repository-relative path order; violations shall be ordered by rule ID and location. | Must | US-001 / Scenarios: All active artifact types are valid; Multiple violations are listed separately; Validation is deterministic and read-only |
| FR-008 | The validator shall use stable rule IDs in the `ARTIFACT` dot type token dot rule token namespace and retain warning severity while treating warnings as failures. | Must | US-001 / Scenarios: Invalid active artifacts produce a failed result; Multiple violations are listed separately |
| FR-009 | The validator shall produce the same ordered result for the same repository state without network access and shall not modify artifacts, lifecycle statuses, or persist validation results. | Must | US-001 / Scenario: Validation is deterministic and read-only |

## Postconditions And Invariants

- Every active artifact is represented exactly once in the result.
- No excluded file is represented as an active artifact.
- Every applicable violation is represented exactly once as a separate entry.
- Any warning or error violation produces artifact status `diagnostic` and
  overall result `failure`.
- A successful non-empty result contains only artifacts with status `ok`.
- A successful empty result contains an empty artifact list.
- No artifact content, lifecycle status, or persisted validation result changes.

## Edge And Failure Behavior

| Condition | Expected behavior | User-visible result |
| --- | --- | --- |
| No active artifacts exist | Complete validation without error | Overall `success` and an empty artifact list |
| A file is under `specs/templates/` or `specs/archive/` | Exclude it from active discovery | No artifact result for the file |
| A canonical active file has malformed or unrecognized frontmatter | Continue validation and retain the file by path | Artifact status `diagnostic` with the parsing or recognition violation |
| An artifact has several violations | Continue collecting applicable violations | One artifact result with a separate list entry for every violation |
| A violation has warning severity | Retain the warning severity | Artifact status `diagnostic` and overall `failure` |
| An artifact has no valid ID | Continue validation and sort it after identified artifacts by path | Path-based artifact result with status `diagnostic` |

## Quality Requirements

- Validation is deterministic for identical repository state and inputs.
- Validation requires no network, hosted service, or AI dependency.
- Validation is read-only and does not persist results.
- Validation reports all applicable diagnostics rather than stopping at the first
  violation.
- Result and violation ordering is stable and follows the defined ordering rules.
- Diagnostics contain enough location, rule, severity, message, and remediation
  context for an agent or CI check to correct the source artifact.
- The operation is suitable for execution on every pull request; the exact
  runtime threshold remains a product-level follow-up decision.

## Dependencies And Deferred Decisions

- Source dependency: approved `US-001`, `EPIC-001`, and `PRD-001`.
- Related decisions: `ADR-002`, `ADR-003`, and `ADR-004`.
- Global uniqueness and cross-artifact relationship resolution are deferred to
  EPIC-002.
- Lifecycle promotion and readiness evaluation are deferred to EPIC-003.
- Command interface, serialization, and CI-provider integration are deferred to
  EPIC-004.
- No runtime or external service dependencies are required.

## Traceability

- Source story: `US-001`
- Parent epic: `EPIC-001`
- Parent PRD: `PRD-001`
- Executable scenarios: `scenarios.feature`
- Covered scenarios: All active artifact types are valid; Invalid active artifacts produce a failed result; Multiple violations are listed separately; Malformed or unrecognized frontmatter remains diagnosable; Template and archive files are excluded; Empty active artifact set succeeds; Validation is deterministic and read-only.

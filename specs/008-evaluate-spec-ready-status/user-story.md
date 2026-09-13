---
id: US-008
title: "Evaluate Spec-Ready status"
type: user-story
status: implemented
created: 2026-09-13
updated: 2026-09-13
promoted_from: in-review
promoted_to: approved
promoted_by: Project owner
promoted_at: 1789317375
owner: TBD
parent: PRD-001
epic: EPIC-003
feature: 008-evaluate-spec-ready-status
depends_on: [US-007]
requires: [EPIC-003, EPIC-001, EPIC-002, US-007]
blockers: []
related: [ADR-002, ADR-007]
approval:
  approved_by: Project owner
  approved_on: 2026-09-13
---

# User Story

## Story Card

As an LLM coding agent or human reviewer,
I want to evaluate one implementation packet for Spec-Ready status,
so that I can determine whether it is ready for implementation without mutating
repository content.

## Context And Value

The workflow defines a Spec-Ready predicate that must pass before implementation
begins, but agents need a deterministic way to evaluate that predicate rather
than manually inspecting every packet artifact and dependency. A read-only,
complete readiness result lets agents and reviewers identify all unmet
conditions before implementation and avoids treating an incomplete packet as
ready.

## Business Rules

- The request targets one implementation packet by directory or packet
  reference.
- The evaluation includes the selected packet's colocated implementation
  artifacts and its implementation-packet supporting artifacts.
- The selected packet's five implementation artifacts — `user-story.md`,
  `scenarios.feature`, `requirements.md`, `design.md`, and `tasks.md` — must be
  exactly in the `approved` lifecycle state for the selected packet to be
  Spec-Ready.
- The evaluation applies every condition in the normative Spec-Ready predicate,
  including approved parent PRD and epic, valid cross-references, approved
  supporting ADRs and schemas, complete behavior coverage, resolved normative
  requirements, explicit design contracts, actionable red/green tasks, and no
  blocking questions.
- Gherkin scenario names declare behavior coverage with one of the exact
  case-sensitive prefixes `Happy:`, `Alternate:`, `Failure:`, or `Boundary:`;
  the evaluation requires at least one scenario in each category.
- Declared feature dependencies are evaluated recursively. A dependency packet
  satisfies its dependency condition when it is implemented and has valid
  implementation and verification evidence; its own dependencies are evaluated
  recursively without requiring its artifacts to remain in the approved state.
- A shared dependency is evaluated once even when reached through multiple
  paths.
- A dependency cycle produces one deterministic cycle diagnostic and causes the
  readiness result to fail.
- The result reports every applicable unmet condition in deterministic order,
  rather than stopping after the first failure.
- The evaluation is read-only, deterministic, offline, and does not perform AI
  judgment, promotion, file movement, release creation, or archival.

## Examples

| Example | Given | When | Expected outcome |
| --- | --- | --- | --- |
| EX-001 | A packet has five approved artifacts, satisfies every predicate, and all recursive dependencies are implemented with valid evidence | The agent evaluates the packet for readiness | The result reports the packet as Spec-Ready with no unmet conditions and repository content is unchanged |
| EX-002 | A packet has a missing approval, an unresolved reference, a normative `TBD`, and an incomplete task checklist | The reviewer evaluates the packet for readiness | The result reports not ready and includes all applicable diagnostics in deterministic order |
| EX-003 | A packet declares a dependency whose packet is not implemented | The agent evaluates the packet for readiness | The result reports the dependency as unmet and the packet is not Spec-Ready |
| EX-004 | Two dependency paths reach the same implemented packet | The agent evaluates the packet for readiness | The shared dependency is evaluated once and its findings are not duplicated |
| EX-005 | Recursive dependencies contain a cycle | The agent evaluates the packet for readiness | The result reports one deterministic cycle diagnostic and the packet is not Spec-Ready |
| EX-006 | The selected packet has already advanced one or more of its own artifacts beyond `approved` | The agent evaluates the packet for readiness | The result reports the corresponding lifecycle-state condition as unmet; no artifact is changed |

## Acceptance Criteria

- The capability accepts one implementation packet directory or reference and
  evaluates it without requiring a target lifecycle state.
- The selected packet's colocated artifacts and implementation-packet
  supporting artifacts are included, while unrelated repository artifacts are
  excluded.
- The selected packet passes only when all five implementation artifacts are
  exactly `approved` and every normative Spec-Ready predicate condition passes.
- Parent PRD and epic status, cross-references, supporting artifact status,
  scenario coverage, requirements completeness, design completeness, task
  readiness, blockers, and dependency conditions are all evaluated.
- Feature dependencies are evaluated recursively; implemented dependencies with
  valid evidence satisfy the dependency condition.
- Shared dependencies are deduplicated and dependency cycles are reported
  deterministically.
- A failed result contains every applicable unmet condition with actionable
  context and deterministic ordering.
- A successful result clearly identifies the packet as Spec-Ready and contains
  no unmet-condition diagnostics.
- Repeated evaluations of identical repository content return identical
  results.
- Evaluation performs no repository mutation, network access, AI judgment,
  promotion, file movement, release creation, or archival.

## Scope Boundaries

### In Scope

- Read-only Spec-Ready evaluation for one implementation packet.
- Normative predicate checks for packet artifacts, context, references,
  supporting artifacts, evidence, blockers, and recursive dependencies.
- Deterministic aggregate readiness results and diagnostics.
- Dependency deduplication and cycle reporting.

### Out Of Scope

- Promoting artifacts or packets through lifecycle states.
- Accepting a caller-supplied target state.
- Creating release records or archiving packets.
- CLI syntax, exit-code taxonomy, and machine-readable serialization.
- Authoring or repairing specifications.
- Human semantic judgment or proof that approval is genuine.
- Network access or hosted-service integration.

## Dependencies

- `US-007` implementation-packet context and recursive packet discovery behavior
  are available.
- `EPIC-001` structural validation, `EPIC-002` identity and relationship
  integrity, and `EPIC-003` lifecycle rules are available.
- `ADR-002` defines the strict promote-only lifecycle and Spec-Ready context.
- Existing implementation and verification evidence is available for implemented
  dependency packets.

## Open Questions

| ID | Question | Blocking? | Owner | Status |
| --- | --- | --- | --- | --- |
| — | No unresolved blocking questions remain. | No | Project owner | Resolved |

## INVEST Check

- [x] Independent
- [x] Negotiable
- [x] Valuable
- [x] Estimable
- [x] Small enough for roughly 1 to 3 days
- [x] Testable

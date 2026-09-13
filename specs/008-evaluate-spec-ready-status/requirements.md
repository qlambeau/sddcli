---
id: REQ-008
title: "Spec-Ready evaluation requirements"
type: feature-requirements
status: implemented
created: 2026-09-13
updated: 2026-09-13
promoted_from: in-review
promoted_to: approved
promoted_by: Project owner
promoted_at: 1789317770
owner: TBD
parent: US-008
depends_on: [REQ-007]
requires: [US-008, EPIC-003, REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-007, ADR-002, ADR-007]
blockers: []
related: [PRD-001, EPIC-003, ADR-002, ADR-007]
approval:
  approved_by: Project owner
  approved_on: 2026-09-13
---

# Requirements

## Purpose And Actors

### Purpose

Define the externally observable contract for determining whether one
implementation packet satisfies the normative Spec-Ready predicate before
implementation begins. The evaluation is read-only and returns either a
successful readiness result or a complete, deterministic set of unmet-condition
diagnostics.

### Actors And External Systems

- An LLM coding agent requesting a readiness evaluation.
- A human reviewer inspecting whether a packet is ready for implementation.
- The current repository and its local `specs/` tree as the evaluation boundary.
- Existing artifact validation, identity, relationship, cycle, packet-context,
  and verification-evidence capabilities.

The capability does not access a hosted service, perform AI judgment, or mutate
lifecycle status.

## Preconditions

- `US-008` and its `scenarios.feature` are approved.
- The parent PRD and epic brief are approved.
- The caller supplies one implementation packet directory or packet reference.
- The repository can be read without changing repository content.
- Dependency packet evidence, when required, is recorded in the dependency's
  implementation and verification artifacts.

## Inputs And Outputs

| Interaction | Inputs | Outputs | Validation |
| --- | --- | --- | --- |
| Evaluate one packet | One implementation packet directory or reference and the current repository state | A Spec-Ready result identifying the packet as ready or not ready, included evaluation scope, and ordered diagnostics when applicable | The packet reference must resolve to one implementation packet; no caller-supplied target lifecycle state is accepted or required |
| Inspect a successful result | A valid packet satisfying every predicate | A ready result with no unmet-condition diagnostics | All selected packet artifacts and applicable recursive dependency conditions pass |
| Inspect a failed result | A valid packet with one or more unmet conditions | A not-ready result containing every applicable actionable diagnostic in deterministic order | Evaluation continues after individual predicate failures |
| Resolve an invalid packet reference | A missing or unresolvable directory or packet reference | An operational packet-reference error distinct from a valid packet's not-ready result | No readiness result is reported for an unresolved root reference and no repository content is changed |

## Functional Requirements

| ID | Requirement | Priority | Traceability (Story & Scenario) |
| --- | --- | --- | --- |
| FR-001 | The capability shall accept exactly one implementation packet directory or packet reference and shall evaluate that packet without accepting or requiring a caller-supplied lifecycle target state. | Must | US-008 / Acceptance criteria; Scenarios: A complete approved packet with implemented dependencies is Spec-Ready; An invalid packet reference returns an operational error |
| FR-002 | The capability shall resolve the selected packet's colocated `user-story.md`, `scenarios.feature`, `requirements.md`, `design.md`, and `tasks.md` artifacts together with implementation-packet supporting artifacts in the packet context. | Must | US-008 / Acceptance criteria; Scenario: Packet context includes supporting artifacts but excludes unrelated artifacts |
| FR-003 | The capability shall exclude unrelated recognized artifacts, including artifacts reachable through repository relationships but outside the selected implementation-packet context, from the readiness evaluation. | Must | US-008 / Acceptance criteria; Scenario: Packet context includes supporting artifacts but excludes unrelated artifacts |
| FR-004 | The selected packet shall be considered eligible for readiness only when each of its five colocated implementation artifacts has exactly the `approved` lifecycle state. | Must | US-008 / Business rule and acceptance criteria; Scenario: An artifact beyond approval is not currently Spec-Ready |
| FR-005 | The capability shall evaluate the selected packet against every applicable normative Spec-Ready predicate condition: approved parent PRD and epic brief; resolved and valid cross-references; approved supporting ADRs and schemas; at least one Gherkin scenario whose name begins with each exact case-sensitive prefix `Happy:`, `Alternate:`, `Failure:`, and `Boundary:`; no unresolved normative `TBD` values; explicit design interfaces, data contracts, errors, and state transitions; actionable red- and green-test tasks; and resolved blockers. | Must | US-008 / Business rules and acceptance criteria; Scenarios: Happy: A complete approved packet with implemented dependencies is Spec-Ready; Failure: All applicable unmet conditions are reported deterministically |
| FR-006 | The capability shall recursively evaluate every declared feature dependency and shall treat a dependency as satisfied only when its packet is `implemented` with valid recorded implementation and verification evidence; the dependency's own declared dependencies shall be evaluated recursively without requiring its artifacts to remain `approved`. | Must | US-008 / Business rules and acceptance criteria; Scenarios: A complete approved packet with implemented dependencies is Spec-Ready; An unimplemented recursive dependency prevents readiness |
| FR-007 | The capability shall evaluate a shared dependency at most once per request, even when multiple dependency paths reach it, and shall not duplicate its diagnostics in the result. | Must | US-008 / Business rules and acceptance criteria; Scenario: A shared dependency is evaluated only once |
| FR-008 | The capability shall detect cycles in the recursive feature-dependency graph, report one deterministic dependency-cycle diagnostic for each distinct cycle, and make the readiness result not ready when a cycle is present. | Must | US-008 / Business rules and acceptance criteria; Scenario: A recursive dependency cycle prevents readiness |
| FR-009 | When all selected-packet and recursive dependency conditions pass, the capability shall return a successful result identifying the packet as Spec-Ready and containing no unmet-condition diagnostics. | Must | US-008 / Acceptance criteria; Scenario: A complete approved packet with implemented dependencies is Spec-Ready |
| FR-010 | When any applicable predicate condition fails, the capability shall return a not-ready result containing every applicable unmet-condition diagnostic, including lifecycle, reference, completeness, blocker, evidence, dependency, and cycle findings as applicable. | Must | US-008 / Business rules and acceptance criteria; Scenarios: All applicable unmet conditions are reported deterministically; An unimplemented recursive dependency prevents readiness; A recursive dependency cycle prevents readiness; An artifact beyond approval is not currently Spec-Ready |
| FR-011 | If the root packet directory or reference cannot be resolved, the capability shall return an operational packet-reference error that identifies the unresolved reference rather than a not-ready result. | Must | US-008 / Confirmed behavior; Scenario: An invalid packet reference returns an operational error |
| FR-012 | The capability shall produce the same readiness classification, evaluation scope, and diagnostic ordering for identical repository content and packet input, including when shared dependencies or cycles are encountered. | Must | US-008 / Acceptance criteria; Scenarios: All applicable unmet conditions are reported deterministically; A shared dependency is evaluated only once; A recursive dependency cycle prevents readiness; Repeated evaluation is deterministic and read-only |
| FR-013 | The capability shall not modify packet artifacts, dependency artifacts, unrelated artifacts, promotion metadata, release records, or filesystem locations during evaluation. | Must | US-008 / Business rules and acceptance criteria; Scenarios: A complete approved packet with implemented dependencies is Spec-Ready; All applicable unmet conditions are reported deterministically; An invalid packet reference returns an operational error; Repeated evaluation is deterministic and read-only |

## Postconditions And Invariants

- A successful result means every selected-packet predicate and recursive
  dependency condition passed at evaluation time.
- A not-ready result identifies every applicable unmet condition observed during
  the same read-only evaluation.
- An operational packet-reference error is distinct from a valid packet that is
  not Spec-Ready.
- The selected packet's own artifacts are never treated as Spec-Ready when any
  one of the five artifacts is outside the exact `approved` state.
- Shared dependency content contributes to the result at most once per request.
- Distinct dependency cycles remain distinguishable and are reported in stable
  order.
- Repeating an evaluation without repository changes does not change its result
  or mutate repository content.

## Edge And Failure Behavior

| Condition | Expected behavior | User-visible result |
| --- | --- | --- |
| The root packet directory or reference does not exist or cannot be resolved | Stop root resolution without mutation | Operational packet-reference error identifying the input reference |
| A valid packet is missing a required colocated artifact | Continue evaluating applicable conditions | Not-ready result with a missing-artifact diagnostic |
| A selected packet artifact is `draft`, `in-review`, `implemented`, `archived`, or another non-`approved` state | Continue evaluating the packet | Not-ready result identifying the artifact and unmet lifecycle state |
| A parent, epic, supporting artifact, or cross-reference fails its predicate | Continue evaluating all applicable conditions | One or more actionable diagnostics in deterministic order |
| Gherkin scenario names do not provide all four required category prefixes | Continue evaluating all applicable conditions | Not-ready result with a scenario-coverage diagnostic identifying the missing categories |
| A feature dependency is not implemented or lacks valid recorded evidence | Continue evaluating other dependencies | Not-ready result identifying the unmet dependency condition |
| Multiple dependency paths reach one dependency | Evaluate the dependency once | No duplicate dependency findings |
| The recursive dependency graph contains a cycle | Detect and report the cycle without unbounded traversal | Not-ready result with one deterministic diagnostic for each distinct cycle |
| Several independent predicate conditions fail | Do not stop at the first failure | Complete aggregate diagnostic result |
| The same request is evaluated repeatedly against unchanged content | Perform equivalent read-only evaluations | Identical classification, scope, and diagnostic order |
| Network or AI judgment would be required | Do not access the network or make a judgment call | Deterministic local result based only on repository evidence |

## Quality Requirements

- **Determinism:** Identical packet input and repository content shall produce
  identical classification, included scope, and diagnostic ordering.
- **Completeness:** The evaluator shall retain all applicable unmet conditions
  from one evaluation rather than returning only the first failure.
- **Read-only safety:** Evaluation shall not write, delete, rename, move, or
  otherwise alter repository content or lifecycle metadata.
- **Offline operation:** Evaluation shall require no network, hosted service, or
  AI service.
- **Repository boundary:** Evaluation shall operate only on the selected local
  repository and its recognized packet context.
- **Actionability:** Diagnostics shall identify the affected packet, dependency,
  artifact, or predicate condition sufficiently for an agent or reviewer to
  correct the issue.
- **Cycle safety:** Recursive dependency evaluation shall terminate for cyclic
  and shared graphs.

## Dependencies And Deferred Decisions

- Depends on `REQ-007` packet-context discovery and recursive relationship
  resolution behavior.
- Requires the approved validation, identity, relationship, and cycle contracts
  from `REQ-001` through `REQ-005`.
- Requires lifecycle and promotion policy from `ADR-002`.
- Uses recorded implementation and verification evidence established by the
  feature verification workflow.
- CLI syntax, exit-code taxonomy, and machine-readable serialization remain
  deferred to EPIC-004.

## Traceability

- Source story: `US-008`
- Executable scenarios: `specs/008-evaluate-spec-ready-status/scenarios.feature`
- Parent epic: `EPIC-003`
- Parent PRD: `PRD-001`
- Related decisions: `ADR-002`, `ADR-007`

Every functional requirement maps to one or more approved Gherkin scenarios,
and every scenario is covered by at least one functional requirement.

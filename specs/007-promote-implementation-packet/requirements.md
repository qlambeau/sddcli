---
id: REQ-007
title: "Atomic implementation packet promotion requirements"
type: feature-requirements
status: implemented
created: 2026-09-13
updated: 2026-09-13
promoted_from: in-review
promoted_to: approved
promoted_by: Project owner
promoted_at: 1789313229
owner: TBD
parent: US-007
depends_on: [REQ-006]
requires: [US-007, EPIC-003, REQ-001, REQ-005, REQ-006, ADR-002, ADR-007]
blockers: []
related: [PRD-001, EPIC-003, ADR-002, ADR-007]
approval:
  approved_by: Project owner
  approved_on: 2026-09-13
---

# Requirements

## Purpose And Actors

### Purpose

Define the externally observable contract for promoting one complete
implementation packet and its implementation-packet supporting artifacts one
lifecycle step atomically. The capability derives each included artifact's next
valid lifecycle transition, records skipped participants, and guarantees that a
failed request leaves every artifact unchanged.

### Actors And External Systems

- LLM coding agent requesting one packet promotion step.
- Human reviewer requesting one packet promotion step after external review
  confirmation.
- The current repository and its local `specs/` tree as the promotion source
  and persistence boundary.
- Existing artifact validation, relationship-integrity, cycle-validation, and
  single-artifact promotion capabilities.

The capability records supplied actor identity and per-artifact confirmations.
It does not classify actors, judge whether approval is genuinely human, access
hosted services, or perform AI judgment.

## Preconditions

- The target is one recognized implementation packet directory at a canonical
  repository location.
- The repository and every included artifact can be read, and every artifact
  selected for advancement can be safely updated if the request succeeds.
- The source story `US-007` and executable scenarios are approved.
- Existing artifact validation, relationship-integrity, cycle, and
  single-artifact promotion findings are available for preflight.
- The caller supplies one actor identity for the packet promotion request.
- When a selected artifact requires external confirmation or prerequisite facts,
  those facts are supplied by artifact ID or artifact type.
- Implementation-packet supporting artifacts can be distinguished from reachable
  artifacts outside the implementation packet context.

## Inputs And Outputs

| Interaction | Inputs | Outputs | Validation |
| --- | --- | --- | --- |
| Request one packet promotion step | Implementation packet reference, actor identity, repository state, and per-artifact confirmations or prerequisite facts keyed by artifact ID or artifact type | Success result with advanced and skipped participants, or failure result with ordered diagnostics and no mutation | The packet is recognized, included artifacts are discovered, next states are derived, all selected artifacts pass prerequisite checks, and commit protection succeeds before mutation is finalized |
| Inspect packet promotion result | Result of the packet request and resulting repository state | Advanced participant records, skipped participant records, latest promotion metadata for advanced artifacts, and diagnostics for failures | Successful results identify every advanced and skipped participant; failed results expose every applicable failure in deterministic order |

A successful real advancement records the latest promotion metadata on each
advanced artifact: source state, target state, actor identity, and promotion
timestamp. Skipped artifacts do not receive new promotion metadata.

## Functional Requirements

| ID | Requirement | Priority | Traceability (Story & Scenario) |
| --- | --- | --- | --- |
| FR-001 | The capability shall accept one implementation packet reference and interpret the request as "promote this packet one step" without accepting or requiring a packet-level target state. | Must | US-007 / Scenarios: A complete draft packet enters review atomically; Mixed packet artifacts advance to their own next valid states |
| FR-002 | The capability shall include the colocated packet artifacts `user-story.md`, `scenarios.feature`, `requirements.md`, `design.md`, and `tasks.md` when they exist at recognized packet locations. | Must | US-007 / Scenario: A complete draft packet enters review atomically |
| FR-003 | The capability shall include directly referenced implementation-packet supporting artifacts, including ADRs and schemas, when they are part of the implementation packet context. | Must | US-007 / Scenario: Implementation-packet supporting artifacts are included through direct references |
| FR-004 | The capability shall recursively include artifacts reached through `requires`, `depends_on`, and `related` only while those artifacts remain part of the implementation packet context. | Must | US-007 / Scenario: Implementation-packet supporting artifacts are included recursively |
| FR-005 | The capability shall exclude reachable artifacts outside the implementation packet context, and shall exclude parent PRDs and epic briefs by default unless they are implementation-packet supporting artifacts. | Must | US-007 / Scenarios: Artifacts outside the implementation packet context are excluded; Parent PRD and epic are not included by default |
| FR-006 | The capability shall derive each included artifact's own next valid lifecycle state from its current state and the strict promote-only lifecycle rules. | Must | US-007 / Scenario: Mixed packet artifacts advance to their own next valid states |
| FR-007 | The capability shall select advanceable included artifacts for mutation and evaluate each selected artifact against its applicable validation, prerequisite, confirmation, blocker, relationship, evidence, and conflict checks before any artifact is changed. | Must | US-007 / Scenarios: Missing per-artifact confirmation fails the whole packet promotion; Multiple prerequisite failures are reported without mutation; Source conflict prevents all packet mutations |
| FR-008 | The capability shall match per-artifact confirmations and prerequisite facts by artifact ID or artifact type rather than by filesystem path. | Must | US-007 / Scenarios: Mixed packet artifacts advance to their own next valid states; Missing per-artifact confirmation fails the whole packet promotion |
| FR-009 | The capability shall treat included artifacts that cannot advance one next valid lifecycle state as skipped participants, leave them unchanged, and report them as skipped without producing an error. | Must | US-007 / Scenario: Included not-advanceable artifacts are reported as skipped without error |
| FR-010 | The capability shall apply all accepted artifact advancements atomically: if any selected artifact fails preflight or commit protection, no included artifact shall be mutated. | Must | US-007 / Scenarios: Missing per-artifact confirmation fails the whole packet promotion; Multiple prerequisite failures are reported without mutation; Source conflict prevents all packet mutations |
| FR-011 | For every successful artifact advancement, the capability shall update only that artifact's lifecycle state and latest promotion metadata containing source state, target state, actor identity, and promotion timestamp. | Must | US-007 / Scenarios: A complete draft packet enters review atomically; Mixed packet artifacts advance to their own next valid states; Successful packet promotion changes only advancing artifacts |
| FR-012 | A successful packet promotion shall leave skipped artifacts, excluded artifacts, unrelated artifacts, unrelated files, and file locations unchanged, and shall not create release records. | Must | US-007 / Scenarios: Artifacts outside the implementation packet context are excluded; Included not-advanceable artifacts are reported as skipped without error; Successful packet promotion changes only advancing artifacts |
| FR-013 | A failed packet promotion shall report every applicable failure as actionable diagnostics in deterministic order and leave all artifact state, content, and promotion metadata unchanged. | Must | US-007 / Scenarios: Multiple prerequisite failures are reported without mutation; Repeated failed packet promotion requests are deterministic and read-only |
| FR-014 | The capability shall detect source conflicts between preflight and commit and fail the whole packet request without applying partial updates. | Must | US-007 / Scenario: Source conflict prevents all packet mutations |
| FR-015 | The capability shall perform no network access, hosted-service interaction, artifact movement, release-record creation, or AI judgment. | Must | US-007 / Scenarios: Successful packet promotion changes only advancing artifacts; Repeated failed packet promotion requests are deterministic and read-only |

## Postconditions And Invariants

- A successful packet promotion result identifies every included artifact that
  advanced and every included artifact that was skipped.
- Every advanced artifact is left in its own derived next valid lifecycle state.
- Every advanced artifact has exactly one latest promotion record with source
  state, target state, actor identity, and promotion timestamp.
- Skipped artifacts remain byte-equivalent with respect to lifecycle state,
  content, and promotion metadata.
- Excluded artifacts, unrelated artifacts, unrelated files, and file locations
  remain unchanged.
- A failed request leaves every artifact's state, content, and promotion
  metadata unchanged.
- The operation never moves artifacts into or out of archive locations.
- The operation never creates or promotes a release record.
- The result for identical repository state and request inputs is identical and
  ordered identically.
- No network, AI service, or persisted validation cache is required for a packet
  promotion decision.

## Edge And Failure Behavior

| Condition | Expected behavior | User-visible result |
| --- | --- | --- |
| The packet reference is not recognized | Reject the request without mutation | Actionable packet-recognition diagnostic |
| A colocated packet artifact is absent | Continue with discovered recognized artifacts unless the absence creates an applicable prerequisite failure | Included set and any applicable diagnostics identify the condition |
| A reachable artifact is outside implementation packet context | Exclude it without mutation | It is not reported as advanced or skipped |
| Parent PRD or epic is reachable but not a packet-supporting artifact | Exclude it without mutation | Parent artifact remains unchanged |
| An included artifact has no valid next lifecycle step | Report it as skipped and leave it unchanged | Skipped participant entry, no error |
| A selected artifact lacks required per-artifact confirmation | Reject the whole packet request without mutation | Confirmation diagnostic for that artifact ID or type |
| A selected artifact has unresolved blockers, invalid relationships, failed validation, or missing required evidence | Reject the whole packet request without mutation | Complete deterministic diagnostics |
| Multiple selected artifacts fail different prerequisites | Continue evaluation and report every applicable failure | Complete ordered diagnostics and no mutation |
| A selected artifact source changes after preflight | Reject the whole packet request without mutation | Conflict diagnostic and retry guidance |
| The same failed request is repeated against unchanged repository state | Produce the same result without mutation | Identical diagnostics in identical order |
| A request succeeds with advancing and skipped participants | Mutate only advancing artifacts and record skipped participants | Success result with advanced and skipped participant lists |
| Every included artifact is skipped and zero artifacts advance | Perform no mutation and report every included artifact as skipped | Successful no-op result with skipped participant list |

## Quality Requirements

- Packet promotion decisions are deterministic for identical repository state
  and request inputs.
- Failed packet promotion requests are atomic and do not partially update any
  artifact.
- Successful packet promotion changes only lifecycle state and latest promotion
  metadata for advancing artifacts.
- Promotion preflight reports all applicable diagnostics rather than stopping at
  the first failure.
- Diagnostic, advanced participant, and skipped participant ordering is stable
  across repeated evaluations.
- Diagnostics identify the affected artifact and provide enough context for an
  agent or reviewer to correct the prerequisite.
- Promotion requires no network, hosted service, AI dependency, or repository
  state outside the local `specs/` tree.
- The capability does not determine whether approval decisions are genuinely
  human.
- CLI syntax, exit-code taxonomy, and machine-readable serialization remain
  deferred to EPIC-004.

## Dependencies And Deferred Decisions

- Source story: approved `US-007`.
- Executable scenarios: approved `scenarios.feature` for `US-007`.
- Parent epic: approved `EPIC-003`.
- Parent PRD: approved `PRD-001`.
- Existing validation and relationship-integrity contracts: `REQ-001` through
  `REQ-005`.
- Single-artifact promotion contract: `REQ-006`.
- Strict promote-only lifecycle policy: approved `ADR-002`.
- Atomic promotion metadata and persistence decision: approved `ADR-007`.
- All-skipped packet behavior is resolved: if every included artifact is
  skipped and zero artifacts advance, the request returns a successful no-op
  result with all included artifacts reported as skipped.
- CLI syntax, stable exit codes, and machine-readable output are deferred to
  EPIC-004.

## Traceability

- Source story: `US-007`
- Parent epic: `EPIC-003`
- Parent PRD: `PRD-001`
- Executable scenarios: `scenarios.feature`
- Covered scenarios: A complete draft packet enters review atomically; Mixed
  packet artifacts advance to their own next valid states; Implementation-packet
  supporting artifacts are included through direct references;
  Implementation-packet supporting artifacts are included recursively; Artifacts
  outside the implementation packet context are excluded; Parent PRD and epic
  are not included by default; Included not-advanceable artifacts are reported
  as skipped without error; Missing per-artifact confirmation fails the whole
  packet promotion; Multiple prerequisite failures are reported without
  mutation; Source conflict prevents all packet mutations; Successful packet
  promotion changes only advancing artifacts; Repeated failed packet promotion
  requests are deterministic and read-only.

# parent: US-007
# status: implemented
# promoted_from: in-review
# promoted_to: approved
# promoted_by: Project owner
# promoted_at: 1789313059

Feature: Promote a complete implementation packet one lifecycle step atomically
  LLM coding agents and human reviewers need to advance a complete
  implementation packet and its implementation-packet supporting artifacts as a
  single atomic operation so that packet lifecycle progress cannot become
  partially applied or misleading.

  Scenario: A complete draft packet enters review atomically
    Given a complete implementation packet contains colocated story, scenarios, requirements, design, and tasks artifacts in `draft`
    And each colocated artifact satisfies its review-entry checks
    When an LLM coding agent requests one packet promotion step with actor `agent-1`
    Then each colocated artifact state becomes `in-review`
    And each advanced artifact records source `draft`, target `in-review`, and actor `agent-1`
    And each advanced artifact records a promotion timestamp
    And the packet promotion result reports each colocated artifact as advanced

  Scenario: Mixed packet artifacts advance to their own next valid states
    Given a complete implementation packet contains some included artifacts in `draft`
    And the same packet contains other included artifacts in `in-review`
    And required per-artifact confirmations are supplied by artifact ID or artifact type
    And every included artifact selected for advancement satisfies its prerequisites
    When a human reviewer requests one packet promotion step with actor `reviewer-1`
    Then the `draft` artifacts become `in-review`
    And the `in-review` artifacts become `approved`
    And all advanced artifacts record their own source state, target state, actor `reviewer-1`, and promotion timestamp
    And all advancement mutations are committed atomically

  Scenario: Implementation-packet supporting artifacts are included through direct references
    Given a complete implementation packet references an implementation-packet ADR through `related`
    And the referenced ADR is active and can advance one valid lifecycle state
    And all colocated packet artifacts selected for advancement satisfy their prerequisites
    When an LLM coding agent requests one packet promotion step with actor `agent-1`
    Then the referenced ADR is included in the packet promotion set
    And the referenced ADR advances one valid lifecycle state
    And the colocated packet artifacts selected for advancement advance one valid lifecycle state
    And the promotion result reports the ADR and colocated artifacts as advanced

  Scenario: Implementation-packet supporting artifacts are included recursively
    Given a complete implementation packet references an implementation-packet supporting artifact through `requires`
    And that supporting artifact references another implementation-packet supporting artifact through `depends_on`
    And both supporting artifacts are active and can advance one valid lifecycle state
    When an LLM coding agent requests one packet promotion step with actor `agent-1`
    Then both supporting artifacts are included in the packet promotion set
    And both supporting artifacts advance one valid lifecycle state atomically with the packet

  Scenario: Artifacts outside the implementation packet context are excluded
    Given a complete implementation packet has a reachable relationship to an artifact outside the implementation packet context
    And the outside artifact can advance one valid lifecycle state
    When an LLM coding agent requests one packet promotion step with actor `agent-1`
    Then the outside artifact is excluded from the packet promotion set
    And the outside artifact state and promotion metadata remain unchanged
    And the packet promotion result does not report the outside artifact as advanced or skipped

  Scenario: Parent PRD and epic are not included by default
    Given a complete implementation packet has an approved parent PRD
    And the packet has an approved parent epic brief
    And neither parent artifact is an implementation-packet supporting artifact
    When an LLM coding agent requests one packet promotion step with actor `agent-1`
    Then the parent PRD is excluded from the packet promotion set
    And the parent epic brief is excluded from the packet promotion set
    And both parent artifacts remain unchanged

  Scenario: Included not-advanceable artifacts are reported as skipped without error
    Given a complete implementation packet includes an artifact that cannot advance one next valid lifecycle state
    And at least one other included artifact can advance one valid lifecycle state
    And every artifact selected for advancement satisfies its prerequisites
    When an LLM coding agent requests one packet promotion step with actor `agent-1`
    Then the not-advanceable artifact is reported as skipped
    And the skipped artifact state and promotion metadata remain unchanged
    And the packet promotion succeeds
    And the advanceable included artifacts are updated atomically

  Scenario: Missing per-artifact confirmation fails the whole packet promotion
    Given a complete implementation packet contains an artifact in `in-review`
    And that artifact requires human approval confirmation to advance to `approved`
    And no required confirmation is supplied for that artifact ID or artifact type
    When a human reviewer requests one packet promotion step with actor `reviewer-1`
    Then the packet promotion fails with a confirmation diagnostic
    And every included artifact state and promotion metadata remain unchanged
    And no unrelated artifact or file is modified

  Scenario: Multiple prerequisite failures are reported without mutation
    Given a complete implementation packet contains multiple artifacts selected for advancement
    And one selected artifact has an unresolved blocker
    And another selected artifact has an invalid required relationship
    And another selected artifact is missing required verification evidence
    When an LLM coding agent requests one packet promotion step with actor `agent-1`
    Then the packet promotion fails
    And diagnostics identify the unresolved blocker, invalid relationship, and missing evidence
    And the diagnostics are reported in deterministic order
    And every included artifact state and promotion metadata remain unchanged

  Scenario: Source conflict prevents all packet mutations
    Given a complete implementation packet has all advancement decisions accepted
    And one included artifact source changes after preflight but before commit
    When an LLM coding agent requests one packet promotion step with actor `agent-1`
    Then the packet promotion fails with a conflict diagnostic
    And no included artifact state or promotion metadata is updated
    And the caller can retry after reloading repository state

  Scenario: Successful packet promotion changes only advancing artifacts
    Given a complete implementation packet includes advancing artifacts and skipped artifacts
    And unrelated recognized artifacts and files are captured before promotion
    When an LLM coding agent requests one packet promotion step with actor `agent-1`
    Then every advancing artifact state and promotion metadata are updated
    And skipped artifact content and promotion metadata remain unchanged
    And unrelated artifacts and files remain unchanged
    And no artifact is moved
    And no release record is created

  Scenario: Repeated failed packet promotion requests are deterministic and read-only
    Given a complete implementation packet has a fixed invalid packet promotion request
    And the repository state is captured before promotion
    When an LLM coding agent submits the same packet promotion request twice without network access
    Then both failure results contain identical diagnostics in identical order
    And the repository state remains identical to the captured state
    And no AI judgment is required

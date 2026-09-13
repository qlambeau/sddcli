# parent: US-008
# status: implemented

Feature: Evaluate Spec-Ready status for an implementation packet
  LLM coding agents and human reviewers need a deterministic, read-only result
  that identifies whether one implementation packet satisfies the normative
  Spec-Ready predicate before implementation begins.

  Scenario: Happy: A complete approved packet with implemented dependencies is Spec-Ready
    Given an implementation packet is referenced by its directory
    And its story, scenarios, requirements, design, and tasks artifacts are all `approved`
    And its parent PRD and epic brief are `approved`
    And its supporting ADRs and schemas are approved and all cross-references resolve
    And its scenarios cover happy, alternate, failure, and boundary behavior
    And its requirements contain no unresolved normative `TBD` values
    And its design contains explicit external interfaces, data contracts, errors, and state transitions
    And its tasks contain actionable red and green test work
    And its blockers are resolved
    And every recursively declared dependency packet is implemented with valid implementation and verification evidence
    When an LLM coding agent evaluates the packet for Spec-Ready status
    Then the readiness result reports the packet as Spec-Ready
    And the readiness result contains no unmet-condition diagnostics
    And the repository content remains unchanged

  Scenario: Alternate: Packet context includes supporting artifacts but excludes unrelated artifacts
    Given an implementation packet is referenced by its directory
    And the packet contains its five colocated implementation artifacts
    And the packet references an implementation-packet supporting ADR
    And the supporting ADR references another supporting artifact required by the packet
    And an unrelated recognized artifact is reachable through a repository relationship but is outside the packet context
    When a human reviewer evaluates the packet for Spec-Ready status
    Then the readiness evaluation includes the colocated artifacts and both supporting artifacts
    And the unrelated artifact is excluded from the evaluation
    And the repository content remains unchanged

  Scenario: Failure: All applicable unmet conditions are reported deterministically
    Given an implementation packet is referenced by its directory
    And one packet artifact is not `approved`
    And the parent epic brief is not `approved`
    And a supporting reference is unresolved
    And the scenarios do not cover all required behavior paths
    And the requirements contain an unresolved normative `TBD`
    And the design omits a required data contract
    And the tasks omit actionable green-test work
    And the packet has an unresolved blocker
    When an LLM coding agent evaluates the packet for Spec-Ready status
    Then the readiness result reports the packet as not Spec-Ready
    And the result contains diagnostics for every applicable unmet condition
    And the diagnostics are ordered deterministically
    And no packet or unrelated repository content is changed

  Scenario: Failure: An unimplemented recursive dependency prevents readiness
    Given an implementation packet is otherwise complete and approved
    And the packet declares a dependency packet that is not implemented
    When a human reviewer evaluates the packet for Spec-Ready status
    Then the readiness result reports the packet as not Spec-Ready
    And the result identifies the unimplemented dependency as an unmet condition
    And no packet or dependency content is changed

  Scenario: Alternate: A shared dependency is evaluated only once
    Given an implementation packet is otherwise complete and approved
    And two dependency paths reach the same implemented dependency packet
    And the shared dependency has valid implementation and verification evidence
    When an LLM coding agent evaluates the packet for Spec-Ready status
    Then the shared dependency is evaluated once
    And the readiness result does not duplicate findings for the shared dependency

  Scenario: Failure: A recursive dependency cycle prevents readiness
    Given an implementation packet is otherwise complete and approved
    And its recursive dependency graph contains a cycle
    When a human reviewer evaluates the packet for Spec-Ready status
    Then the readiness result reports the packet as not Spec-Ready
    And the result contains one deterministic dependency-cycle diagnostic
    And no packet or dependency content is changed

  Scenario: Boundary: An artifact beyond approval is not currently Spec-Ready
    Given an implementation packet has one colocated artifact in `implemented`
    And all other packet artifacts and predicate conditions are valid
    When an LLM coding agent evaluates the packet for Spec-Ready status
    Then the readiness result reports the packet as not Spec-Ready
    And the result identifies the artifact's lifecycle state as unmet
    And the artifact remains unchanged

  Scenario: Failure: An invalid packet reference returns an operational error
    Given the requested packet directory or reference does not exist
    When an LLM coding agent evaluates the packet for Spec-Ready status
    Then the operation returns an operational packet-reference error
    And the error identifies that the packet reference cannot be resolved
    And no repository content is changed

  Scenario: Boundary: Repeated evaluation is deterministic and read-only
    Given an implementation packet and its dependencies have a fixed repository state
    And the repository content is captured before evaluation
    When an LLM coding agent evaluates the same packet twice without network access
    Then both readiness results are identical
    And all diagnostics appear in the same order
    And the repository content remains identical to the captured state
    And no AI judgment is required

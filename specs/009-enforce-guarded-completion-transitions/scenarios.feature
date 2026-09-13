# parent: US-009
# status: implemented
# promoted_from: in-review
# promoted_to: approved
# promoted_by: Project owner
# promoted_at: 1789325618

Feature: Enforce guarded completion transitions
  LLM coding agents and human reviewers need completion transitions to enforce
  their prerequisites one transition at a time so that artifacts, releases,
  and feature packets cannot be incorrectly closed.

  Scenario: Happy: An approved artifact is superseded by a valid successor
    Given an approved artifact has an approved successor
    And the predecessor and successor have valid reciprocal supersession links
    When a reviewer requests the predecessor to become `superseded`
    Then the predecessor becomes `superseded`
    And the predecessor remains at its existing path
    And the successor remains approved

  Scenario: Happy: A complete release record becomes released
    Given an approved release record includes only implemented or archived features
    And the release record contains verification evidence
    And the release record contains a release commit
    When a reviewer requests the release record to become `released`
    Then the release record becomes `released`
    And every included feature remains unchanged

  Scenario: Happy: A relocated feature packet becomes archived
    Given an implemented feature packet already resides under `specs/archive/`
    And a corresponding validated release record is `released`
    When a reviewer requests the feature packet to become `archived`
    Then the feature packet becomes `archived`
    And no file is moved by the completion transition

  Scenario: Failure: Invalid supersession prerequisites are aggregated
    Given an approved artifact has no approved successor
    And its supersession links are missing or non-reciprocal
    When an agent requests the artifact to become `superseded`
    Then the request is rejected
    And diagnostics identify every applicable supersession prerequisite failure
    And the artifact remains approved
    And its promotion metadata remains unchanged

  Scenario: Failure: An incomplete release is rejected without mutation
    Given an approved release record includes an unfinished feature
    And the release record lacks verification evidence
    And the release record lacks a release commit
    When an agent requests the release record to become `released`
    Then the request is rejected
    And diagnostics identify every applicable release prerequisite failure
    And the release record remains approved
    And its promotion metadata remains unchanged

  Scenario Outline: Failure: A packet lacking an archival prerequisite cannot be archived
    Given an implemented feature packet is <packet_location>
    And its corresponding release record is <release_state>
    When an agent requests the feature packet to become `archived`
    Then the request is rejected
    And diagnostics identify the unmet archival prerequisite
    And the feature packet remains implemented
    And no file is moved

    Examples:
      | packet_location                 | release_state       |
      | outside `specs/archive/`        | `released`          |
      | under `specs/archive/`          | not `released`      |

  Scenario: Boundary: Terminal completion states are immutable
    Given an artifact or release record is already `superseded`, `released`, or `archived`
    When a reviewer requests another completion transition for the same target
    Then the request is rejected as an immutable terminal-state operation
    And the target remains unchanged

  Scenario: Boundary: Skipped and backward completion transitions are rejected
    Given a target is in an active lifecycle state
    When an agent requests an unsupported, skipped, or backward completion transition
    Then the request is rejected
    And the target remains unchanged

  Scenario: Boundary: One request cannot batch completion transitions
    Given two targets require different completion transitions
    When an agent submits one request for both targets
    Then the request is rejected
    And neither target is changed

  Scenario: Boundary: Repeated failed evaluation is deterministic and offline
    Given a target and its prerequisite content remain unchanged
    When an agent evaluates the same failed completion request twice without network access
    Then both decisions are rejected
    And both diagnostic collections are identical and identically ordered
    And no AI judgment is required

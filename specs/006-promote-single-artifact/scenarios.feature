# parent: US-006
# status: implemented

Feature: Promote a single SDD artifact through its lifecycle
  LLM coding agents and human reviewers need to advance one recognized SDD
  artifact safely so that lifecycle rules, review gates, and promotion history
  cannot be bypassed.

  Scenario Outline: A recognized active artifact enters review
    Given a recognized active <artifact_type> artifact exists in `draft`
    And the artifact satisfies all review-entry checks
    When an LLM coding agent requests promotion to `in-review` with actor `agent-1`
    Then the artifact state becomes `in-review`
    And the promotion metadata records source `draft`, target `in-review`, and actor `agent-1`
    And the promotion timestamp is recorded

    Examples:
      | artifact_type      |
      | PRD                |
      | epic brief         |
      | user story         |
      | Gherkin scenario   |
      | requirements       |
      | design             |
      | tasks              |
      | ADR                |

  Scenario: An artifact under review is approved after external human confirmation
    Given a recognized active artifact exists in `in-review`
    And the surrounding workflow has confirmed human approval
    And all approval checks pass
    When the supplied actor `reviewer-1` requests promotion to `approved`
    Then the artifact state becomes `approved`
    And the promotion metadata records source `in-review`, target `approved`, and actor `reviewer-1`
    And the promotion timestamp is recorded
    And the promotion capability does not classify the actor or judge whether approval is genuinely human

  Scenario: An approved artifact becomes implemented after verification
    Given a recognized active artifact exists in `approved`
    And the required implementation and verification evidence is recorded
    When an LLM coding agent requests promotion to `implemented` with actor `agent-1`
    Then the artifact state becomes `implemented`
    And the promotion metadata records source `approved`, target `implemented`, and actor `agent-1`
    And the promotion timestamp is recorded

  Scenario: An implemented artifact is archived from its canonical archive location
    Given a recognized artifact exists in `implemented`
    And the artifact is already in its canonical archive location
    And the closed release conditions are satisfied
    When an LLM coding agent requests promotion to `archived` with actor `agent-1`
    Then the artifact state becomes `archived`
    And the promotion metadata records source `implemented`, target `archived`, and actor `agent-1`
    And no artifact is moved by the promotion

  Scenario: An approved artifact becomes superseded by an approved successor
    Given a recognized active artifact exists in `approved`
    And an approved successor artifact exists
    And the required supersession links between the artifacts are valid
    When an LLM coding agent requests promotion to `superseded` with actor `agent-1`
    Then the artifact state becomes `superseded`
    And the promotion metadata records source `approved`, target `superseded`, and actor `agent-1`
    And the promotion timestamp is recorded

  Scenario: A same-state promotion request is idempotent
    Given a recognized active artifact exists in `approved`
    And the artifact content and promotion metadata are captured
    When an LLM coding agent requests promotion to `approved` with actor `agent-1`
    Then the promotion succeeds as an idempotent no-op
    And the artifact content and promotion metadata are unchanged

  Scenario Outline: An invalid lifecycle transition is rejected
    Given a recognized artifact exists in `<source_state>`
    When an LLM coding agent requests promotion to `<target_state>` with actor `agent-1`
    Then the promotion fails with an invalid-transition diagnostic
    And the artifact state, content, and promotion metadata remain unchanged

    Examples:
      | source_state | target_state |
      | draft        | approved     |
      | approved     | draft        |
      | implemented  | in-review    |
      | archived     | approved     |
      | superseded   | approved     |

  Scenario: Archival is rejected when the artifact has not been relocated
    Given a recognized artifact exists in `implemented`
    And the artifact is not in its canonical archive location
    When an LLM coding agent requests promotion to `archived` with actor `agent-1`
    Then the promotion fails with an archive-location diagnostic
    And the artifact state and promotion metadata remain unchanged
    And no artifact is moved by the promotion

  Scenario: Supersession is rejected without an approved successor and valid links
    Given a recognized active artifact exists in `approved`
    And no approved successor with valid supersession links exists
    When an LLM coding agent requests promotion to `superseded` with actor `agent-1`
    Then the promotion fails with a supersession-prerequisite diagnostic
    And the artifact state and promotion metadata remain unchanged

  Scenario: Missing implementation evidence blocks promotion
    Given a recognized active artifact exists in `approved`
    And required implementation or verification evidence is missing
    When an LLM coding agent requests promotion to `implemented` with actor `agent-1`
    Then the promotion fails with an evidence diagnostic
    And the artifact state and promotion metadata remain unchanged

  Scenario: Multiple applicable failures are reported without mutation
    Given a recognized active artifact exists in `approved`
    And required implementation evidence is missing
    And the artifact has an unresolved blocker
    And the artifact has an invalid required relationship
    When an LLM coding agent requests promotion to `implemented` with actor `agent-1`
    Then the promotion fails
    And diagnostics identify the missing evidence, unresolved blocker, and invalid relationship
    And the diagnostics are reported in deterministic order
    And the artifact state, content, and promotion metadata remain unchanged

  Scenario: A successful promotion changes only the requested artifact state and metadata
    Given a recognized active artifact exists in `draft`
    And its review-entry checks pass
    And unrelated recognized artifacts and files are captured
    When an LLM coding agent requests promotion to `in-review` with actor `agent-1`
    Then only the requested artifact state and promotion metadata change
    And unrelated artifacts and files remain unchanged
    And no network access or AI judgment is required

  Scenario: Repeated failed promotion requests are deterministic and read-only
    Given a recognized active artifact has a fixed invalid promotion request
    And the repository state is captured before promotion
    When an LLM coding agent submits the same request twice without network access
    Then both failure results contain identical diagnostics in identical order
    And the repository state remains identical to the captured state

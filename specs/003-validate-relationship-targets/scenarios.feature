# parent: US-003
# status: approved

Feature: Validate relationship targets
  LLM coding agents and CI checks need local artifact relationships resolved
  against the complete repository so that missing and wrong-kind targets are
  found before lifecycle or downstream actions.

  Scenario: Valid relationships resolve across current and historical artifacts
    Given the repository contains recognized active, archived, and superseded artifacts
    And every concrete parent, epic, dependency, requirement, blocker, related, and supersession reference points to an existing artifact of the expected kind
    When an LLM coding agent or CI check validates relationship targets
    Then no missing-target or wrong-kind diagnostic is reported
    And every valid relationship remains represented
    And the overall relationship result succeeds

  Scenario: Empty relationship collections do not create target failures
    Given the repository contains recognized artifacts with empty relationship collections
    When an LLM coding agent or CI check validates relationship targets
    Then no missing-target or wrong-kind diagnostic is reported for the empty collections
    And the overall relationship result succeeds

  Scenario: Missing targets are reported for every invalid relationship entry
    Given a recognized artifact has concrete dependency and requirement references to absent IDs
    When an LLM coding agent or CI check validates relationship targets
    Then the referencing artifact contains one actionable missing-target diagnostic for each absent ID
    And each missing-target diagnostic identifies its relationship field and target ID
    And the overall relationship result fails

  Scenario Outline: Wrong-kind targets remain diagnosable
    Given a recognized <source_kind> has a concrete <relationship_field> reference to an existing <actual_kind> artifact
    And the expected target kind for that relationship is <expected_kind>
    When an LLM coding agent or CI check validates relationship targets
    Then the referencing artifact contains an actionable wrong-kind diagnostic
    And the diagnostic identifies expected kind "<expected_kind>" and actual kind "<actual_kind>"
    And the overall relationship result fails

    Examples:
      | source_kind | relationship_field | expected_kind | actual_kind |
      | user story  | parent              | PRD           | ADR         |
      | user story  | epic                | epic          | requirements |
      | design      | parent              | user story    | epic        |
      | ADR         | supersedes          | ADR           | user story  |

  Scenario: Archived and superseded targets resolve
    Given a recognized artifact references an archived target
    And another recognized artifact references a superseded target
    When an LLM coding agent or CI check validates relationship targets
    Then both historical targets resolve successfully
    And neither reference receives a missing-target diagnostic

  Scenario: Excluded files cannot satisfy a relationship
    Given a template or supporting file contains an ID referenced by a canonical artifact
    And no eligible canonical artifact contains that ID
    When an LLM coding agent or CI check validates relationship targets
    Then the excluded file is not considered a relationship target
    And the referencing artifact receives a missing-target diagnostic

  Scenario: Relationship failures do not suppress unrelated artifacts
    Given one canonical artifact has missing or wrong-kind relationship targets
    And other canonical artifacts have valid relationship targets
    When an LLM coding agent or CI check validates relationship targets
    Then each invalid relationship entry is diagnosed on its referencing artifact
    And the valid unrelated artifacts remain represented
    And valid unrelated artifacts have no relationship target diagnostic

  Scenario: Structural relationship failures are not duplicated as target failures
    Given a canonical artifact has malformed, empty, or unresolved relationship values
    When an LLM coding agent or CI check validates relationship targets
    Then the EPIC-001 structural diagnostics remain represented
    And no additional missing-target or wrong-kind diagnostic is reported for those values

  Scenario: Relationship validation is deterministic and read-only
    Given the repository has a fixed set of active, archived, and superseded artifacts and relationships
    And the repository state is captured before validation
    When an LLM coding agent or CI check validates relationship targets twice without network access
    Then both relationship results are identical and ordered identically
    And no artifact content, lifecycle status, or persisted result is changed

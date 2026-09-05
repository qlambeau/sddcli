# parent: US-001
# status: approved

Feature: Validate active artifacts

  Scenario: Valid input
    Given an active repository
    When validation runs
    Then the report succeeds

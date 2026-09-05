# parent: US-001
# status: approved

Feature: Validate active SDD artifacts
  LLM coding agents and CI checks need a complete, read-only validation result
  for active SDD artifacts so that invalid documents can be corrected before
  downstream review or use.

  Scenario: All active artifact types are valid
    Given the active repository contains valid PRD, epic brief, user story, scenario, requirements, design, tasks, and ADR artifacts
    When an LLM coding agent or CI check validates the active artifacts
    Then each active artifact is listed exactly once with status "ok"
    And the overall validation result is successful

  Scenario: Invalid active artifacts produce a failed result
    Given one or more active artifacts violate their applicable structural rules
    When an LLM coding agent or CI check validates the active artifacts
    Then each affected artifact is listed with status "diagnostic"
    And the applicable violations are reported
    And the overall validation result fails

  Scenario: Multiple violations are listed separately
    Given an active artifact has multiple applicable violations
    When an LLM coding agent or CI check validates the active artifacts
    Then the artifact is listed with status "diagnostic"
    And each violation is reported as a separate list entry
    And each violation includes its artifact path, location when available, rule ID, severity, message, and remediation guidance

  Scenario: Malformed or unrecognized frontmatter remains diagnosable
    Given an active file has malformed or unrecognized frontmatter
    When an LLM coding agent or CI check validates the active artifacts
    Then the file is listed by path with status "diagnostic"
    And the parsing or recognition violation is reported

  Scenario: Template and archive files are excluded
    Given matching artifact files exist under "specs/templates/" or "specs/archive/"
    When an LLM coding agent or CI check validates the active artifacts
    Then those files are not listed as active artifacts

  Scenario: Empty active artifact set succeeds
    Given the repository contains no files at the canonical active artifact locations
    When an LLM coding agent or CI check validates the active artifacts
    Then the artifact result list is empty
    And the overall validation result is successful

  Scenario: Validation is deterministic and read-only
    Given the repository has a fixed set of active artifacts
    When an LLM coding agent or CI check validates the active artifacts twice without network access
    Then both validation results are identical
    And no artifact files, lifecycle statuses, or persisted results are changed

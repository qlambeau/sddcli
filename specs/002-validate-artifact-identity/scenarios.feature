# parent: US-002
# status: approved

Feature: Validate repository-wide artifact identity
  LLM coding agents and CI checks need artifact identities to remain unique and
  aligned with path-encoded names across current and historical specifications
  so that relationships and lifecycle actions refer to an unambiguous source.

  Scenario: All recognized current and historical identities are valid
    Given the repository contains active, archived, and superseded canonical artifacts
    And every recognized artifact ID is unique across those artifacts
    And each path-encoded PRD, epic, and ADR ID matches its frontmatter ID
    When an LLM coding agent or CI check validates artifact identities
    Then each recognized artifact identity is represented exactly once
    And no identity diagnostic is reported
    And the overall identity result succeeds

  Scenario: Empty identity set succeeds
    Given the repository contains no eligible recognized artifact identities
    When an LLM coding agent or CI check validates artifact identities
    Then the identity result list is empty
    And the overall identity result succeeds

  Scenario: Duplicate IDs are reported for every conflicting artifact
    Given an active artifact and an archived artifact both have the recognized ID "US-001"
    When an LLM coding agent or CI check validates artifact identities
    Then both conflicting artifact paths are listed with status "diagnostic"
    And each conflicting result contains an actionable duplicate-ID diagnostic for "US-001"
    And the overall identity result fails

  Scenario Outline: Path-encoded identity mismatches remain diagnosable
    Given a canonical <artifact_kind> file named "<path>" has recognized frontmatter ID "<frontmatter_id>"
    When an LLM coding agent or CI check validates artifact identities
    Then the file is listed with status "diagnostic"
    And an actionable path-identity diagnostic reports filename ID "<filename_id>" and frontmatter ID "<frontmatter_id>"
    And identity validation continues for other recognized artifacts

    Examples:
      | artifact_kind | path                                          | filename_id | frontmatter_id |
      | PRD           | specs/prds/PRD-003.md                        | PRD-003     | PRD-004        |
      | epic          | specs/prds/PRD-001-epics/EPIC-003.md         | EPIC-003    | EPIC-004       |
      | ADR           | specs/adr/ADR-003.md                         | ADR-003     | ADR-004        |

  Scenario: Excluded files do not create identity collisions
    Given an active artifact has the recognized ID "US-001"
    And a template or supporting file also contains the text "US-001"
    When an LLM coding agent or CI check validates artifact identities
    Then the excluded file is not represented in the identity results
    And the active artifact is not diagnosed as conflicting with the excluded file

  Scenario: Identity conflicts do not suppress unrelated artifacts
    Given one canonical artifact has an identity conflict
    And other canonical artifacts have unique recognized IDs and matching path identities
    When an LLM coding agent or CI check validates artifact identities
    Then the conflicting artifact is listed with status "diagnostic"
    And the unrelated artifacts are also represented in the result
    And valid unrelated artifacts have no identity diagnostic

  Scenario: Malformed frontmatter remains an EPIC-001 structural diagnostic
    Given a canonical artifact has malformed frontmatter and no recognized ID
    When an LLM coding agent or CI check validates artifact identities
    Then the file remains represented by its path with its structural diagnostic
    And identity validation does not reparse the malformed frontmatter

  Scenario: Identity validation is deterministic and read-only
    Given the repository has a fixed set of active, archived, and superseded canonical artifacts
    And the repository state is captured before validation
    When an LLM coding agent or CI check validates artifact identities twice without network access
    Then both identity results are identical and ordered identically
    And no artifact content, lifecycle status, or persisted result is changed

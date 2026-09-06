# parent: US-004
# status: approved

Feature: Validate reciprocal relationships
  LLM coding agents and CI checks need reciprocal artifact relationships
  validated so that asymmetric traceability is detected before downstream
  actions.

  Scenario: Reciprocal related links resolve across current and historical artifacts
    Given recognized active and archived artifacts A and B contain reciprocal `related` links
    When an LLM coding agent or CI check validates reciprocal relationships
    Then no reciprocity diagnostic is reported for the links
    And both artifact results remain represented
    And the overall reciprocal-consistency result succeeds

  Scenario: A missing related counterpart is reported on its source
    Given recognized artifacts A and B exist
    And artifact A has a concrete resolved `related` reference to B
    And artifact B does not contain A in its `related` collection
    When an LLM coding agent or CI check validates reciprocal relationships
    Then artifact A receives one actionable reciprocity diagnostic
    And the diagnostic identifies A's `related` field, target B, and the expected reverse link
    And the overall reciprocal-consistency result fails

  Scenario: A different reverse related link does not satisfy the original link
    Given recognized artifacts A, B, and C exist
    And artifact A has a concrete resolved `related` reference to B
    And artifact B has a concrete resolved `related` reference to C
    And artifact C has a concrete resolved `related` reference to B
    And artifact B does not contain A in its `related` collection
    When an LLM coding agent or CI check validates reciprocal relationships
    Then artifact A receives one actionable reciprocity diagnostic for its link to B
    And artifact B's link to C is checked independently
    And the valid B-to-C relationship receives no reciprocity diagnostic

  Scenario: Reciprocal supersession links resolve across historical artifacts
    Given an active artifact A supersedes an archived artifact B
    And archived artifact B contains A in its `superseded_by` field
    When an LLM coding agent or CI check validates reciprocal relationships
    Then both supersession entries are reciprocal
    And neither entry receives a reciprocity diagnostic
    And the overall reciprocal-consistency result succeeds

  Scenario Outline: Missing supersession counterparts are reported on their source
    Given recognized artifacts A and B exist
    And artifact A has a concrete resolved `<source_field>` reference to B
    And artifact B does not contain A in its `<reverse_field>` field
    When an LLM coding agent or CI check validates reciprocal relationships
    Then artifact A receives one actionable reciprocity diagnostic
    And the diagnostic identifies `<source_field>`, target B, and expected `<reverse_field>`
    And the overall reciprocal-consistency result fails

    Examples:
      | source_field  | reverse_field  |
      | supersedes    | superseded_by  |
      | superseded_by | supersedes     |

  Scenario: Multiple reciprocity failures do not suppress valid unrelated artifacts
    Given one recognized artifact has an unmatched `related` entry
    And another recognized artifact has an unmatched supersession entry
    And other recognized artifacts contain valid reciprocal relationships
    When an LLM coding agent or CI check validates reciprocal relationships
    Then one diagnostic is reported for each unmatched directed entry
    And valid unrelated artifact results remain represented
    And valid unrelated relationships receive no reciprocity diagnostic
    And the overall reciprocal-consistency result fails

  Scenario: Excluded files cannot satisfy a reciprocal counterpart
    Given a canonical artifact A references an ID that exists only in a template or supporting file
    And no eligible recognized canonical artifact contains that ID
    And US-003 has represented the reference as a missing target
    When an LLM coding agent or CI check validates reciprocal relationships
    Then the excluded file is not considered a reciprocal counterpart
    And no additional reciprocity diagnostic is reported for the unresolved target

  Scenario: Empty relationship collections do not create reciprocity failures
    Given recognized artifacts have empty `related`, `supersedes`, and `superseded_by` values
    When an LLM coding agent or CI check validates reciprocal relationships
    Then no reciprocity diagnostic is reported for the empty values
    And the overall reciprocal-consistency result succeeds

  Scenario: Earlier structural and target failures are not duplicated
    Given a canonical artifact contains malformed, unresolved, missing-target, or wrong-kind relationship values
    And EPIC-001 or US-003 has already represented the applicable structural or target diagnostics
    When an LLM coding agent or CI check validates reciprocal relationships
    Then the earlier diagnostics remain represented
    And no additional reciprocity diagnostic is reported for those values

  Scenario: Relationship membership ignores order and duplicate occurrences
    Given artifact A relates to artifact B
    And artifact B contains A in its `related` collection in a different position or more than once
    When an LLM coding agent or CI check validates reciprocal relationships
    Then the reciprocal membership is satisfied
    And no reciprocity diagnostic is reported

  Scenario: Reciprocity validation is deterministic and read-only
    Given the repository has a fixed set of active, archived, and superseded artifacts and relationships
    And the repository state is captured before validation
    When an LLM coding agent or CI check validates reciprocal relationships twice without network access
    Then both reciprocal-consistency results are identical and ordered identically
    And no artifact content, lifecycle status, or persisted result is changed

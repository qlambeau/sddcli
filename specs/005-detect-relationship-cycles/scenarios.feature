# parent: US-005
# status: approved

Feature: Detect relationship cycles
  LLM coding agents and CI checks need cycles detected in artifact relationships
  so that invalid traceability graphs are identified before downstream actions.

  Scenario: A valid repository with no cycles succeeds
    Given recognized active, archived, and superseded artifacts have valid parent, dependency, and supersession relationships
    And no relationship family contains a closed directed path
    When an LLM coding agent or CI check validates relationship cycles
    Then no cycle diagnostic is reported
    And the overall cycle result succeeds

  Scenario: An eligible parent cycle is reported within the parent graph
    Given eligible parent relationships form the directed path A to B to C to A
    And no dependency or supersession relationship closes a cycle
    When an LLM coding agent or CI check validates relationship cycles
    Then the overall cycle result fails
    And one parent-cycle diagnostic is attached to each participating relationship entry
    And each diagnostic identifies its source entry and the same canonical cycle identity

  Scenario Outline: Each dependency relationship field participates in cycle detection
    Given recognized artifacts A, B, and C exist
    And artifact A has a concrete resolved `<relationship_field>` reference to B
    And artifact B has a concrete resolved `<relationship_field>` reference to C
    And artifact C has a concrete resolved `<relationship_field>` reference to A
    When an LLM coding agent or CI check validates relationship cycles
    Then the overall cycle result fails
    And the dependency cycle contains all three directed relationship entries
    And each participating entry receives one dependency-cycle diagnostic

    Examples:
      | relationship_field |
      | depends_on         |
      | requires           |
      | blockers            |

  Scenario: Supersession fields are normalized to one logical direction
    Given active artifact A has a concrete resolved `supersedes` reference to archived artifact B
    And superseded artifact C has a concrete resolved `superseded_by` reference to B
    And superseded artifact C has a concrete resolved `supersedes` reference to active artifact A
    When an LLM coding agent or CI check validates relationship cycles
    Then the overall cycle result fails
    And one supersession cycle is identified with logical edges A to B, B to C, and C to A
    And each participating supersession entry receives one cycle diagnostic

  Scenario: A target-valid self-loop is reported
    Given recognized artifact A has a concrete resolved `depends_on` reference to itself
    And the `depends_on` target-kind rule accepts artifact A's kind
    When an LLM coding agent or CI check validates relationship cycles
    Then artifact A's dependency entry receives one cycle diagnostic
    And the diagnostic identifies a cycle containing only A's directed dependency edge
    And the overall cycle result fails

  Scenario: A wrong-kind self-loop is not duplicated as a cycle
    Given recognized artifact A has a concrete resolved `parent` reference to itself
    And the `parent` target-kind rule rejects artifact A's kind
    And the existing wrong-kind diagnostic is represented for A's `parent` entry
    When an LLM coding agent or CI check validates relationship cycles
    Then the existing wrong-kind diagnostic remains represented
    And no cycle diagnostic is added for A's `parent` entry

  Scenario: Different relationship families do not form a cycle together
    Given artifact A has a concrete resolved `depends_on` reference to B
    And artifact B has a concrete resolved `parent` reference to C
    And artifact C has a concrete resolved `depends_on` reference to A
    And no single relationship family contains a closed directed path
    When an LLM coding agent or CI check validates relationship cycles
    Then no cycle diagnostic is reported for the mixed-family path
    And the overall cycle result succeeds

  Scenario: A directed cycle is reported once regardless of traversal rotation
    Given recognized artifacts A, B, and C have dependency edges A to B, B to C, and C to A
    And the same directed cycle can be reached from different starting artifacts or rotations
    When an LLM coding agent or CI check validates relationship cycles
    Then exactly one dependency cycle identity is reported
    And each of the three participating dependency entries receives exactly one cycle diagnostic
    And no additional diagnostic is created for a traversal rotation

  Scenario: Distinct overlapping cycles receive distinct findings
    Given recognized artifacts A, B, C, and D have dependency edges A to B, B to C, C to A, B to D, and D to A
    And the cycle A to B to C to A and the cycle A to B to D to A are distinct
    When an LLM coding agent or CI check validates relationship cycles
    Then both distinct dependency cycles are reported
    And A's dependency entry to B receives one diagnostic for each cycle
    And the entries B to C, C to A, B to D, and D to A receive one diagnostic for their respective cycle

  Scenario Outline: Ineligible relationship values retain existing diagnostics without cycle findings
    Given a recognized artifact has a `<value_kind>` `<relationship_field>` value
    And the existing `<existing_diagnostic>` diagnostic is represented for that value
    When an LLM coding agent or CI check validates relationship cycles
    Then the existing `<existing_diagnostic>` diagnostic remains represented
    And no cycle diagnostic is added for the value

    Examples:
      | value_kind             | relationship_field | existing_diagnostic       |
      | malformed              | depends_on         | structural               |
      | unresolved             | requires           | structural               |
      | missing-target         | blockers           | missing-target            |
      | wrong-kind             | parent             | wrong-kind               |

  Scenario: A target-valid non-reciprocal relationship remains eligible
    Given same-kind recognized artifacts A, B, and C have supersession edges A to B, B to C, and C to A
    And A's `supersedes` entry to B lacks its required reverse counterpart
    And the existing non-reciprocal diagnostic is represented for A's entry
    When an LLM coding agent or CI check validates relationship cycles
    Then the existing non-reciprocal diagnostic remains represented
    And A's `supersedes` entry also receives one cycle diagnostic
    And the other participating supersession entries receive their cycle diagnostics
    And the overall cycle result fails

  Scenario: Historical recognized artifacts participate while excluded files do not
    Given active artifact A, archived artifact B, and superseded artifact C have dependency edges A to B, B to C, and C to A
    And a template or supporting file contains an ID referenced by a recognized artifact
    And no recognized canonical artifact contains that excluded ID
    When an LLM coding agent or CI check validates relationship cycles
    Then the cycle across A, B, and C is reported
    And the excluded file cannot provide a graph edge or satisfy a cycle
    And all three recognized artifact results remain represented

  Scenario: Cycle findings do not suppress valid unrelated artifacts
    Given recognized artifacts A, B, and C form a dependency cycle
    And recognized artifact D has valid relationships that do not participate in any cycle
    When an LLM coding agent or CI check validates relationship cycles
    Then the cycle findings for A, B, and C are retained
    And D remains represented in the ordered result
    And D receives no cycle diagnostic
    And the overall cycle result fails

  Scenario: Cycle validation is deterministic and read-only
    Given the repository has a fixed set of active, archived, and superseded artifacts and relationships
    And the repository state is captured before validation
    When an LLM coding agent or CI check validates relationship cycles twice without network access
    Then both complete cycle results are identical and ordered identically
    And no artifact content, lifecycle status, or persisted result is changed

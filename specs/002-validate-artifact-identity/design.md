---
id: DES-002
title: "Repository-wide artifact identity design"
type: feature-design
status: approved
created: 2026-09-05
updated: 2026-09-05
owner: TBD
parent: US-002
depends_on: []
requires: [REQ-002]
blockers: []
related:
  - REQ-002
  - EPIC-002
  - ADR-002
  - ADR-003
  - ADR-004
  - ADR-005
  - ADR-006
approval:
  approved_by: Project owner
  approved_on: 2026-09-05
---

# Design

## Context And Constraints

EPIC-001 already recognizes active artifacts, normalizes parser output, applies
structural rules, and returns an ordered validation report. US-002 extends that
capability to identity integrity across active, archived, and superseded
artifacts without changing the active-only behavior of EPIC-001.

The design must preserve the approved constraints:

- Identity checks are deterministic, offline, complete, and read-only.
- Templates and non-artifact supporting documents remain outside the identity
  set.
- Malformed or unrecognized frontmatter remains an EPIC-001 structural finding;
  identity validation does not reparse it.
- Domain logic remains pure and parser-independent.
- Application code owns orchestration and ports; adapters own filesystem access
  and parser translation, following ADR-005.
- The existing validation-report shape is reused rather than introducing a
  second result contract.
- No CLI, serialization, persistence, schema, release, or relationship-graph
  behavior is introduced by this story.

## Proposed Design

Keep the existing active `ArtifactSource` contract unchanged. Add a separate
application-owned identity source port whose result includes all recognized
canonical artifacts in active locations and `specs/archive/`. The existing
filesystem source implements both source roles, with separate discovery paths
so an active validation cannot accidentally include historical artifacts.

The identity validation use case performs the following flow:

1. Request the repository-wide candidate set through the identity source port.
2. Preserve per-file source failures as path-based candidates and return a
   typed operational error only when repository discovery cannot be established.
3. Build structural artifact results from normalized snapshots using the existing
   EPIC-001 rules.
4. Pass all snapshots with recognized IDs to the pure domain identity index.
5. Add duplicate-ID and path-identity diagnostics to the affected artifact
   results without discarding structural findings or valid unrelated results.
6. Construct the existing ordered validation report, deriving overall failure
   from any identity or structural diagnostic.

The identity index uses ordered domain collections keyed by recognized
`ArtifactId`. It emits one duplicate finding for every conflicting artifact
path. For PRDs, epics, and ADRs it also derives the path-encoded ID from the
repository-relative path and compares it with the recognized frontmatter ID.
Other packet filenames do not encode an artifact ID and therefore receive no
path-identity comparison.

## Components And Responsibilities

| Component | Responsibility | Depends on |
| --- | --- | --- |
| Domain identity index | Collect recognized identities, detect global duplicates, compare path-encoded IDs, and return stable diagnostics without I/O | Domain value objects and standard collections |
| Existing domain report model | Merge structural and identity diagnostics, calculate per-artifact and overall status, and apply established ordering | Domain diagnostics and artifact snapshots |
| `ArtifactIdentitySource` port | Supply active and historical normalized candidates or a typed repository-level discovery error | Application error and candidate types |
| Identity validation use case | Orchestrate source loading, structural results, identity evaluation, diagnostic merging, and final report creation | `ArtifactIdentitySource`, domain identity index, existing report model |
| Filesystem identity source | Discover eligible active and archived canonical paths, exclude templates/supporting files, read each candidate, and map parser failures | Filesystem and parser adapters; application port |
| In-memory identity source | Supply deterministic candidates and discovery failures for application tests | Application port |

## Interfaces And Contracts

| Interface | Inputs | Outputs | Errors |
| --- | --- | --- | --- |
| `ArtifactIdentitySource` | Repository context held by the concrete source | `Result<Vec<ArtifactCandidate>, ValidationError>` containing active and historical candidates in any order | `ValidationError::Discovery` only when the repository-wide candidate set cannot be established; per-file failures remain candidates |
| Domain identity evaluation | Normalized snapshots with recognized IDs and repository-relative paths | Identity diagnostics attributable to artifact paths, in stable order | No operational errors; invalid identity state becomes diagnostics |
| Identity validation use case | An injected `ArtifactIdentitySource` | Existing `ValidationReport`, including structural and identity findings | Propagates repository-level discovery failure; does not abort on candidate-level failure |
| Filesystem identity discovery | Repository root | Candidates for eligible active and historical canonical paths | Maps unreadable files to source diagnostics and maps root/specs failures to `ValidationError::Discovery` |

The identity diagnostics use the existing diagnostic value and ordering contract
from `REQ-001` with stable rules in the `ARTIFACT.IDENTITY.*` namespace. The
duplicate-ID rule includes the repeated ID and all conflicting paths. The
path-identity rule includes the filename ID and frontmatter ID. Both are error
findings, so their affected artifact and the overall report are diagnostic and
failure respectively.

## Data And State Flow

The operation is read-only and has no persisted state:

1. The source verifies the repository root and locates the `specs/` tree.
2. Discovery includes eligible active paths and archived packet paths, excludes
   templates/supporting files, and sorts paths before reading.
3. Each eligible file is read once and converted to either a normalized snapshot
   or a path-based source diagnostic.
4. The application retains all candidates and builds the base structural report.
5. The domain creates an identity index from snapshots with recognized IDs,
   emitting findings for duplicate IDs and path-encoded mismatches.
6. Findings are merged by repository-relative path into the existing artifact
   results and sorted using the established type, ID, path, rule, and location
   ordering.
7. An empty eligible set returns a successful empty report. Mixed valid and
   invalid candidates return a complete report. A repository discovery failure
   returns the typed operational error and no partial success claim.

No rollback is needed because the operation performs no mutation. Rerunning it
against the same repository state is the recovery behavior.

## Security, Performance, And Operations

- Security: Restrict reads to the supplied repository's local `specs/` tree;
  exclude templates and supporting files; do not execute content or access the
  network; write no source or result files.
- Performance: Read and normalize each eligible candidate once, index IDs in
  ordered collections, and sort normalized results. Work is linear for indexing
  plus sorting for deterministic output. The product runtime target remains
  unspecified by PRD-001.
- Operations: Preserve per-file read and recognition failures in the completed
  report. Treat only failure to establish the repository-wide candidate set as a
  terminal operational error. No migration, cache, retry, or background work is
  introduced.
- Compatibility: The existing active validation source and report contract stay
  compatible. Historical discovery is opt-in through the new identity source
  port, as recorded in ADR-006.

## Alternatives Considered

| Alternative | Why not chosen |
| --- | --- |
| Add an active/history scope argument to `ArtifactSource` | Changes EPIC-001's established port contract and allows unrelated callers to request a broader set accidentally |
| Make the existing source return historical files and filter them in application code | Breaks the active-only source responsibility and moves discovery policy into orchestration |
| Create a separate identity report type | Duplicates the approved validation result contract and complicates later command serialization |
| Evaluate identity rules in the filesystem adapter | Places a business rule at the I/O boundary and prevents pure identity tests |
| Add a new crate or parser dependency | Adds no value for an in-memory identity index and violates the bounded slice |

## Risks And Open Decisions

- Historical packet layouts must remain recognizable by the canonical path
  policy; adapter integration tests cover active and archived forms.
- An unreadable or malformed file cannot contribute a frontmatter identity, so
  EPIC-001's path-based structural diagnostic remains the source of truth for
  that candidate.
- Exact human-readable diagnostic wording remains flexible; stable rule IDs and
  required diagnostic fields are the contract.
- Relationship target resolution, reciprocal links, graph cycles, schema links,
  and release links remain deferred to later EPIC-002 stories or future epics.
- No unresolved or blocking technical decision remains after ADR-006.

## Verification Approach

- Write domain tests first for identity indexing, duplicate grouping, path/ID
  comparisons, empty input, stable ordering, and complete diagnostic retention.
- Write application tests against an in-memory `ArtifactIdentitySource` for empty,
  valid, duplicate, mismatch, mixed, and source-failure cases.
- Run adapter integration tests against active and archived canonical layouts,
  excluded files, malformed input, unreadable files, and read-only snapshots.
- Run all approved US-002 scenarios through the identity use case and real
  filesystem source.
- Add or extend parser fuzz coverage without changing parser dependencies.
- Run `cargo xtask ci`, `cargo machete`, and the fuzz-target smoke check; record
  coverage and observed output in the feature task artifact.

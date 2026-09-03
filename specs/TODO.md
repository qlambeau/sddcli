# Engineering Observations / TODO Catalogue

This document records post-release engineering observations, defects, and
technical debt. Use `specs/templates/supporting/observation.md` for dedicated
observation records when an entry needs more detail than this catalogue.

Record each observation with the next free `OBS-NNN` identifier. Confirm the
behavior and its code locations, assess impact and priority, and decide whether
it should be rejected, deferred, closed, or promoted into a PRD epic. A promoted
observation keeps its record and points to the destination in `promoted_to`.

## SDD process review and questions to be answered

- Does updating an approved artifact put it back to `draft` or `in-review` status? Once reapproved, should following artifacts also be reviewed, or only artifacts dependent on the updated artifact?

## CLI tool command update

- Add an `init` command that bootstraps a full SDD process set of files (skills, folders, templates, and related files).
- Add a `skill` command that outputs a valid skill (frontmatter header plus skill text) that teaches an LLM how to use the tool.

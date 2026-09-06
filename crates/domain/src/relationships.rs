use std::collections::{BTreeMap, BTreeSet};

use crate::{ArtifactKind, ArtifactPath, ArtifactSnapshot, Diagnostic, MetadataValue, Severity};

const MISSING_TARGET_RULE: &str = "ARTIFACT.RELATIONSHIP.MISSING_TARGET";
const WRONG_KIND_RULE: &str = "ARTIFACT.RELATIONSHIP.WRONG_KIND";
const RELATIONSHIP_FIELDS: [&str; 8] = [
    "parent",
    "epic",
    "depends_on",
    "requires",
    "blockers",
    "related",
    "supersedes",
    "superseded_by",
];
const REFERENCE_PREFIXES: [&str; 12] =
    ["PRD", "EPIC", "US", "REQ", "DES", "ADR", "TASK", "CHART", "DB", "TABLE", "OBS", "REL"];

/// Evaluates concrete local relationship targets across one repository snapshot.
///
/// The evaluation only considers normalized values that pass the structural
/// local-reference boundary. Structural validation remains responsible for
/// malformed, empty, and unresolved values.
///
/// Covers: REQ-003 FR-002 through FR-010.
#[must_use]
pub fn validate_relationships(snapshots: &[ArtifactSnapshot]) -> Vec<Diagnostic> {
    let targets = target_index(snapshots);
    let mut diagnostics = Vec::new();

    for snapshot in snapshots {
        for field in RELATIONSHIP_FIELDS {
            let Some(value) = snapshot.metadata().get(field) else { continue };
            let policy = relationship_policy(snapshot.kind(), field);
            if matches!(policy, RelationshipPolicy::Ignore) {
                continue;
            }
            for reference in value_strings(value) {
                let Some(target_id) = RelationshipTargetId::try_new(reference) else { continue };
                evaluate_reference(snapshot, field, &target_id, policy, &targets, &mut diagnostics);
            }
        }
    }

    diagnostics.sort_by(|left, right| {
        left.rule_id()
            .cmp(right.rule_id())
            .then_with(|| left.path().cmp(right.path()))
            .then_with(|| left.message().cmp(right.message()))
    });
    diagnostics
}

fn target_index(
    snapshots: &[ArtifactSnapshot],
) -> BTreeMap<RelationshipTargetId, BTreeSet<ArtifactKind>> {
    let mut targets = BTreeMap::new();
    for snapshot in snapshots {
        let Some(id) = snapshot.id() else { continue };
        targets
            .entry(RelationshipTargetId::from_id(id.as_str()))
            .or_insert_with(BTreeSet::new)
            .insert(snapshot.kind());
    }
    targets
}

fn evaluate_reference(
    snapshot: &ArtifactSnapshot,
    field: &str,
    target_id: &RelationshipTargetId,
    policy: RelationshipPolicy,
    targets: &BTreeMap<RelationshipTargetId, BTreeSet<ArtifactKind>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(target_kinds) = targets.get(target_id) else {
        diagnostics.push(missing_target(snapshot.path(), field, target_id));
        return;
    };
    let RelationshipPolicy::Expected(expected_kind) = policy else { return };
    if target_kinds.contains(&expected_kind) {
        return;
    }
    let Some(actual_kind) = target_kinds.iter().next().copied() else { return };
    diagnostics.push(wrong_kind(snapshot.path(), field, target_id, expected_kind, actual_kind));
}

fn missing_target(
    path: &ArtifactPath,
    field: &str,
    target_id: &RelationshipTargetId,
) -> Diagnostic {
    Diagnostic::new(
        path.clone(),
        None,
        MISSING_TARGET_RULE,
        Severity::Error,
        format!("relationship `{field}` references missing target `{}`", target_id.as_str()),
        format!(
            "add a recognized artifact with identifier `{}` or correct the `{field}` relationship",
            target_id.as_str()
        ),
    )
}

fn wrong_kind(
    path: &ArtifactPath,
    field: &str,
    target_id: &RelationshipTargetId,
    expected_kind: ArtifactKind,
    actual_kind: ArtifactKind,
) -> Diagnostic {
    Diagnostic::new(
        path.clone(),
        None,
        WRONG_KIND_RULE,
        Severity::Error,
        format!(
            "relationship `{field}` target `{}` expects kind `{}` but resolved kind `{}`",
            target_id.as_str(),
            kind_label(expected_kind),
            kind_label(actual_kind)
        ),
        format!("change `{field}` to a recognized {} identifier", kind_label(expected_kind)),
    )
}

#[derive(Clone, Copy)]
pub(crate) enum RelationshipPolicy {
    Ignore,
    Any,
    Expected(ArtifactKind),
}

pub(crate) fn relationship_policy(kind: ArtifactKind, field: &str) -> RelationshipPolicy {
    match field {
        "parent" => match kind {
            ArtifactKind::Epic | ArtifactKind::UserStory => {
                RelationshipPolicy::Expected(ArtifactKind::Prd)
            }
            ArtifactKind::Requirements | ArtifactKind::Design | ArtifactKind::Task => {
                RelationshipPolicy::Expected(ArtifactKind::UserStory)
            }
            _ => RelationshipPolicy::Ignore,
        },
        "epic" if kind == ArtifactKind::UserStory => {
            RelationshipPolicy::Expected(ArtifactKind::Epic)
        }
        "depends_on" | "requires" | "blockers" | "related" => RelationshipPolicy::Any,
        "supersedes" | "superseded_by" => RelationshipPolicy::Expected(kind),
        _ => RelationshipPolicy::Ignore,
    }
}

pub(crate) fn value_strings(value: &MetadataValue) -> Vec<&str> {
    match value {
        MetadataValue::Scalar(value) => vec![value],
        MetadataValue::Sequence(values) => values.iter().map(String::as_str).collect(),
        MetadataValue::Null | MetadataValue::Mapping => Vec::new(),
    }
}

fn kind_label(kind: ArtifactKind) -> String {
    let token = kind.token();
    let display = match kind {
        ArtifactKind::Prd => "PRD",
        ArtifactKind::Epic => "epic",
        ArtifactKind::UserStory => "user story",
        ArtifactKind::Gherkin => "gherkin",
        ArtifactKind::Requirements => "requirements",
        ArtifactKind::Design => "design",
        ArtifactKind::Adr => "ADR",
        ArtifactKind::Task => "task",
    };
    format!("{} ({display})", token.to_ascii_uppercase())
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct RelationshipTargetId(String);

impl RelationshipTargetId {
    fn try_new(value: &str) -> Option<Self> {
        if !is_concrete_reference(value) {
            return None;
        }
        Some(Self(value.to_owned()))
    }

    fn from_id(value: &str) -> Self {
        Self(value.to_owned())
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

fn valid_reference(value: &str) -> bool {
    REFERENCE_PREFIXES.iter().any(|prefix| valid_identifier_text(value, prefix))
}

pub(crate) fn is_concrete_reference(value: &str) -> bool {
    !value.is_empty() && !is_unresolved(value) && valid_reference(value)
}

fn valid_identifier_text(value: &str, prefix: &str) -> bool {
    let Some(number) = value.strip_prefix(prefix).and_then(|value| value.strip_prefix('-')) else {
        return false;
    };
    number.len() == 3 && number.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_unresolved(value: &str) -> bool {
    value.split_whitespace().any(|word| word.eq_ignore_ascii_case("TBD"))
}

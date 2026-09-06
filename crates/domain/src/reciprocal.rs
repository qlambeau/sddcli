use std::collections::{BTreeMap, BTreeSet};

use crate::{ArtifactKind, ArtifactSnapshot, Diagnostic, MetadataValue, Severity};

const NON_RECIPROCAL_RULE: &str = "ARTIFACT.RELATIONSHIP.NON_RECIPROCAL";

/// Evaluates reciprocal membership for concrete relationship entries.
///
/// The evaluator considers only recognized targets and structurally valid local
/// references. Structural and target validators retain ownership of all other
/// values.
///
/// Covers: REQ-004 FR-001 through FR-010.
#[must_use]
pub fn validate_reciprocal_relationships(snapshots: &[ArtifactSnapshot]) -> Vec<Diagnostic> {
    let targets = counterpart_index(snapshots);
    let mut diagnostics = Vec::new();

    for snapshot in snapshots {
        let Some(source_id) = snapshot.id().map(crate::ArtifactId::as_str) else { continue };
        for field in ["related", "supersedes", "superseded_by"] {
            let Some(value) = snapshot.metadata().get(field) else { continue };
            let Some(policy) = reciprocal_policy(snapshot.kind(), field) else { continue };
            if !has_expected_shape(field, value) {
                continue;
            }
            let references = value_strings(value)
                .into_iter()
                .filter(|reference| super::relationships::is_concrete_reference(reference))
                .collect::<BTreeSet<_>>();
            for reference in references {
                let Some(target) = targets.get(reference) else { continue };
                if policy.expected_target_kind.is_some_and(|kind| !target.kinds.contains(&kind)) {
                    continue;
                }
                if !target.memberships.contains(policy.reverse_field, source_id) {
                    diagnostics.push(non_reciprocal(
                        snapshot,
                        field,
                        reference,
                        policy.reverse_field,
                    ));
                }
            }
        }
    }

    diagnostics.sort_by(|left, right| {
        left.path().cmp(right.path()).then_with(|| left.message().cmp(right.message()))
    });
    diagnostics
}

fn counterpart_index(snapshots: &[ArtifactSnapshot]) -> BTreeMap<String, Target> {
    let mut targets = BTreeMap::new();
    for snapshot in snapshots {
        let Some(id) = snapshot.id() else { continue };
        let target = targets.entry(id.as_str().to_owned()).or_insert_with(Target::default);
        target.kinds.insert(snapshot.kind());
        for field in ["related", "supersedes", "superseded_by"] {
            let Some(value) = snapshot.metadata().get(field) else { continue };
            if !has_expected_shape(field, value) {
                continue;
            }
            for reference in value_strings(value) {
                if super::relationships::is_concrete_reference(reference) {
                    target.memberships.insert(field, reference.to_owned());
                }
            }
        }
    }
    targets
}

fn reciprocal_policy(kind: ArtifactKind, field: &str) -> Option<ReciprocalPolicy<'static>> {
    match field {
        "related" => {
            Some(ReciprocalPolicy { reverse_field: "related", expected_target_kind: None })
        }
        "supersedes" => Some(ReciprocalPolicy {
            reverse_field: "superseded_by",
            expected_target_kind: Some(kind),
        }),
        "superseded_by" => {
            Some(ReciprocalPolicy { reverse_field: "supersedes", expected_target_kind: Some(kind) })
        }
        _ => None,
    }
}

fn has_expected_shape(field: &str, value: &MetadataValue) -> bool {
    match field {
        "related" => matches!(value, MetadataValue::Sequence(_)),
        "supersedes" | "superseded_by" => matches!(value, MetadataValue::Scalar(_)),
        _ => false,
    }
}

fn value_strings(value: &MetadataValue) -> Vec<&str> {
    match value {
        MetadataValue::Scalar(value) => vec![value],
        MetadataValue::Sequence(values) => values.iter().map(String::as_str).collect(),
        MetadataValue::Null | MetadataValue::Mapping => Vec::new(),
    }
}

fn non_reciprocal(
    snapshot: &ArtifactSnapshot,
    source_field: &str,
    target_id: &str,
    reverse_field: &str,
) -> Diagnostic {
    let source_id = snapshot.id().map_or("the source artifact", |id| id.as_str());
    Diagnostic::new(
        snapshot.path().clone(),
        None,
        NON_RECIPROCAL_RULE,
        Severity::Error,
        format!(
            "relationship `{source_field}` to `{target_id}` is not reciprocal; target must contain `{source_id}` in its reverse `{reverse_field}` field"
        ),
        format!(
            "add `{source_id}` to `{reverse_field}` on artifact `{target_id}` to complete the reciprocal relationship"
        ),
    )
}

#[derive(Clone, Copy)]
struct ReciprocalPolicy<'a> {
    reverse_field: &'a str,
    expected_target_kind: Option<ArtifactKind>,
}

#[derive(Default)]
struct Target {
    kinds: BTreeSet<ArtifactKind>,
    memberships: Memberships,
}

#[derive(Default)]
struct Memberships {
    related: BTreeSet<String>,
    supersedes: BTreeSet<String>,
    superseded_by: BTreeSet<String>,
}

impl Memberships {
    fn insert(&mut self, field: &str, value: String) {
        match field {
            "related" => {
                self.related.insert(value);
            }
            "supersedes" => {
                self.supersedes.insert(value);
            }
            "superseded_by" => {
                self.superseded_by.insert(value);
            }
            _ => {}
        }
    }

    fn contains(&self, field: &str, value: &str) -> bool {
        match field {
            "related" => self.related.contains(value),
            "supersedes" => self.supersedes.contains(value),
            "superseded_by" => self.superseded_by.contains(value),
            _ => false,
        }
    }
}

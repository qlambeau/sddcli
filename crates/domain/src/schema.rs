use std::collections::BTreeMap;

use crate::{ArtifactKind, ArtifactSnapshot, Diagnostic, MetadataValue, Severity};

const MISSING_TARGET_RULE: &str = "ARTIFACT.SCHEMA.MISSING_TARGET";
const WRONG_KIND_RULE: &str = "ARTIFACT.SCHEMA.WRONG_KIND";
const NON_RECIPROCAL_RULE: &str = "ARTIFACT.SCHEMA.NON_RECIPROCAL";

/// Validates the bidirectional links between database and table schema documents.
///
/// The rule inspects normalized metadata only. It does not execute SQL or access
/// persistence, and malformed metadata remains the responsibility of structural
/// validation.
///
/// Covers: REQ-008 FR-005 and ADR-008 decision 2.
#[must_use]
pub fn validate_schema_links(snapshots: &[ArtifactSnapshot]) -> Vec<Diagnostic> {
    let targets = snapshots
        .iter()
        .filter_map(|snapshot| snapshot.id().map(|id| (id.as_str().to_owned(), snapshot)))
        .collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();

    for snapshot in snapshots {
        match snapshot.kind() {
            ArtifactKind::Database => validate_database(snapshot, &targets, &mut diagnostics),
            ArtifactKind::Table => validate_table(snapshot, &targets, &mut diagnostics),
            _ => {}
        }
    }

    diagnostics.sort_by(|left, right| {
        left.path()
            .cmp(right.path())
            .then_with(|| left.rule_id().cmp(right.rule_id()))
            .then_with(|| left.message().cmp(right.message()))
    });
    diagnostics
}

fn validate_database(
    database: &ArtifactSnapshot,
    targets: &BTreeMap<String, &ArtifactSnapshot>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(database_id) = database.id().map(crate::ArtifactId::as_str) else { return };
    let Some(MetadataValue::Sequence(tables)) = database.metadata().get("tables") else {
        return;
    };
    for table_id in tables {
        let Some(table) = targets.get(table_id) else {
            diagnostics.push(Diagnostic::new(
                database.path().clone(),
                None,
                MISSING_TARGET_RULE,
                Severity::Error,
                format!("database `{database_id}` references missing table `{table_id}`"),
                format!("add recognized table schema `{table_id}` or remove the database entry"),
            ));
            continue;
        };
        if table.kind() != ArtifactKind::Table {
            diagnostics.push(Diagnostic::new(
                database.path().clone(),
                None,
                WRONG_KIND_RULE,
                Severity::Error,
                format!("database `{database_id}` table catalog entry `{table_id}` is not a table schema"),
                format!("replace `{table_id}` with a TABLE-NNN artifact identifier"),
            ));
            continue;
        }
        let points_back = matches!(
            table.metadata().get("database"),
            Some(MetadataValue::Scalar(value)) if value == database_id
        );
        if !points_back {
            diagnostics.push(Diagnostic::new(
                database.path().clone(),
                None,
                NON_RECIPROCAL_RULE,
                Severity::Error,
                format!("table `{table_id}` does not point back to database `{database_id}`"),
                format!("set `{table_id}` metadata `database` to `{database_id}`"),
            ));
        }
    }
}

fn validate_table(
    table: &ArtifactSnapshot,
    targets: &BTreeMap<String, &ArtifactSnapshot>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(table_id) = table.id().map(crate::ArtifactId::as_str) else { return };
    let Some(MetadataValue::Scalar(database_id)) = table.metadata().get("database") else {
        return;
    };
    let Some(database) = targets.get(database_id) else {
        diagnostics.push(Diagnostic::new(
            table.path().clone(),
            None,
            MISSING_TARGET_RULE,
            Severity::Error,
            format!("table `{table_id}` references missing database `{database_id}`"),
            format!("add recognized database schema `{database_id}` or correct the table link"),
        ));
        return;
    };
    if database.kind() != ArtifactKind::Database {
        diagnostics.push(Diagnostic::new(
            table.path().clone(),
            None,
            WRONG_KIND_RULE,
            Severity::Error,
            format!("table `{table_id}` database target `{database_id}` is not a database schema"),
            format!("replace `{database_id}` with a DB-NNN artifact identifier"),
        ));
        return;
    }
    let contains_table = matches!(
        database.metadata().get("tables"),
        Some(MetadataValue::Sequence(values)) if values.iter().any(|value| value == table_id)
    );
    if !contains_table {
        diagnostics.push(Diagnostic::new(
            table.path().clone(),
            None,
            NON_RECIPROCAL_RULE,
            Severity::Error,
            format!("database `{database_id}` does not list table `{table_id}`"),
            format!("add `{table_id}` to database `{database_id}` metadata `tables`"),
        ));
    }
}

use std::collections::{BTreeMap, BTreeSet};

use crate::{ArtifactKind, ArtifactPath, ArtifactSnapshot, Diagnostic, MetadataValue, Severity};

const CYCLE_RULE: &str = "ARTIFACT.RELATIONSHIP.CYCLE";

/// Finds every distinct eligible relationship cycle in the supplied snapshots.
///
/// The evaluator constructs independent parent, dependency, and supersession
/// graphs. It emits only cycle diagnostics; structural and target diagnostics
/// remain owned by the existing validators.
///
/// Covers: REQ-005 FR-001 through FR-014.
#[must_use]
pub fn validate_relationship_cycles(snapshots: &[ArtifactSnapshot]) -> Vec<Diagnostic> {
    let targets = target_index(snapshots);
    let mut diagnostics = Vec::new();

    for family in CycleFamily::ALL {
        let edges = eligible_edges(snapshots, &targets, family);
        for cycle in find_cycles(&edges, family) {
            let identity = cycle.identity();
            diagnostics.extend(
                edges
                    .iter()
                    .filter(|edge| cycle.contains(edge))
                    .map(|edge| cycle_diagnostic(edge, family, identity)),
            );
        }
    }

    diagnostics.sort_by(|left, right| {
        left.path().cmp(right.path()).then_with(|| left.message().cmp(right.message()))
    });
    diagnostics
}

fn target_index(snapshots: &[ArtifactSnapshot]) -> BTreeMap<String, BTreeSet<ArtifactKind>> {
    let mut targets = BTreeMap::new();
    for snapshot in snapshots {
        let Some(id) = snapshot.id() else { continue };
        targets.entry(id.as_str().to_owned()).or_insert_with(BTreeSet::new).insert(snapshot.kind());
    }
    targets
}

fn eligible_edges(
    snapshots: &[ArtifactSnapshot],
    targets: &BTreeMap<String, BTreeSet<ArtifactKind>>,
    family: CycleFamily,
) -> Vec<Edge> {
    let mut edges = Vec::new();
    for snapshot in snapshots {
        let Some(source_id) = snapshot.id() else { continue };
        for &field in family.fields() {
            let Some(value) = snapshot.metadata().get(field) else { continue };
            if !has_expected_shape(field, value) {
                continue;
            }
            let policy = super::relationships::relationship_policy(snapshot.kind(), field);
            if matches!(policy, super::relationships::RelationshipPolicy::Ignore) {
                continue;
            }
            for reference in super::relationships::value_strings(value) {
                if !super::relationships::is_concrete_reference(reference) {
                    continue;
                }
                let Some(target_kinds) = targets.get(reference) else { continue };
                if let super::relationships::RelationshipPolicy::Expected(expected) = policy
                    && !target_kinds.contains(&expected)
                {
                    continue;
                }
                let (graph_source, graph_target) =
                    if family == CycleFamily::Supersession && field == "superseded_by" {
                        (reference.to_owned(), source_id.as_str().to_owned())
                    } else {
                        (source_id.as_str().to_owned(), reference.to_owned())
                    };
                edges.push(Edge {
                    source: graph_source,
                    target: graph_target,
                    path: snapshot.path().clone(),
                    field,
                    reference: reference.to_owned(),
                });
            }
        }
    }
    edges.sort_by(Edge::cmp_for_traversal);
    edges
}

fn has_expected_shape(field: &str, value: &MetadataValue) -> bool {
    match field {
        "depends_on" | "requires" | "blockers" => matches!(value, MetadataValue::Sequence(_)),
        "parent" | "supersedes" | "superseded_by" => matches!(value, MetadataValue::Scalar(_)),
        _ => false,
    }
}

fn find_cycles(edges: &[Edge], family: CycleFamily) -> Vec<Cycle> {
    let mut outgoing: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (index, edge) in edges.iter().enumerate() {
        outgoing.entry(edge.source.as_str()).or_default().push(index);
    }
    for indexes in outgoing.values_mut() {
        indexes.sort_by(|left, right| match (edges.get(*left), edges.get(*right)) {
            (Some(left), Some(right)) => left.cmp_for_traversal(right),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
    }

    let mut found = BTreeMap::new();
    for start in outgoing.keys() {
        let mut search = CycleSearch {
            start,
            outgoing: &outgoing,
            edges,
            family,
            visited: BTreeSet::from([(*start).to_owned()]),
            path: Vec::new(),
            cycles: BTreeMap::new(),
        };
        search.visit(start);
        found.extend(search.cycles);
    }
    found.into_values().collect()
}

struct CycleSearch<'a> {
    start: &'a str,
    outgoing: &'a BTreeMap<&'a str, Vec<usize>>,
    edges: &'a [Edge],
    family: CycleFamily,
    visited: BTreeSet<String>,
    path: Vec<usize>,
    cycles: BTreeMap<String, Cycle>,
}

impl CycleSearch<'_> {
    fn visit(&mut self, current: &str) {
        let Some(next_edges) = self.outgoing.get(current).cloned() else { return };
        for edge_index in next_edges {
            let Some(edge) = self.edges.get(edge_index) else { continue };
            if edge.target == self.start {
                self.path.push(edge_index);
                let cycle = Cycle::new(self.family, &self.path, self.edges);
                let identity = cycle.identity().to_owned();
                self.cycles.entry(identity).or_insert(cycle);
                self.path.pop();
                continue;
            }
            if edge.target.as_str() < self.start || self.visited.contains(edge.target.as_str()) {
                continue;
            }
            self.visited.insert(edge.target.clone());
            self.path.push(edge_index);
            self.visit(&edge.target);
            self.path.pop();
            self.visited.remove(edge.target.as_str());
        }
    }
}

fn cycle_diagnostic(edge: &Edge, family: CycleFamily, identity: &str) -> Diagnostic {
    Diagnostic::new(
        edge.path.clone(),
        None,
        CYCLE_RULE,
        Severity::Error,
        format!(
            "{} relationship `{}` to `{}` participates in cycle `{identity}`",
            family.label(),
            edge.field,
            edge.reference
        ),
        format!("break the {} relationship cycle identified by `{identity}`", family.label()),
    )
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum CycleFamily {
    Parent,
    Dependency,
    Supersession,
}

impl CycleFamily {
    const ALL: [Self; 3] = [Self::Parent, Self::Dependency, Self::Supersession];

    fn fields(self) -> &'static [&'static str] {
        match self {
            Self::Parent => &["parent"],
            Self::Dependency => &["depends_on", "requires", "blockers"],
            Self::Supersession => &["supersedes", "superseded_by"],
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Parent => "parent",
            Self::Dependency => "dependency",
            Self::Supersession => "supersession",
        }
    }
}

#[derive(Clone, Debug)]
struct Edge {
    source: String,
    target: String,
    path: ArtifactPath,
    field: &'static str,
    reference: String,
}

impl Edge {
    fn key(&self) -> EdgeKey {
        EdgeKey { source: self.source.clone(), field: self.field, target: self.target.clone() }
    }

    fn cmp_for_traversal(&self, other: &Self) -> std::cmp::Ordering {
        self.target
            .cmp(&other.target)
            .then_with(|| self.field.cmp(other.field))
            .then_with(|| self.path.cmp(&other.path))
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct EdgeKey {
    source: String,
    field: &'static str,
    target: String,
}

struct Cycle {
    keys: Vec<EdgeKey>,
    identity: String,
}

impl Cycle {
    fn new(family: CycleFamily, path: &[usize], edges: &[Edge]) -> Self {
        let keys =
            path.iter().filter_map(|index| edges.get(*index).map(Edge::key)).collect::<Vec<_>>();
        let identity = cycle_identity(family, &keys);
        Self { keys, identity }
    }

    fn identity(&self) -> &str {
        &self.identity
    }

    fn contains(&self, edge: &Edge) -> bool {
        self.keys.contains(&edge.key())
    }
}

fn cycle_identity(family: CycleFamily, keys: &[EdgeKey]) -> String {
    let edges = keys
        .iter()
        .map(|key| format!("{}-{}-{}", key.source, key.field, key.target))
        .collect::<Vec<_>>()
        .join(",");
    format!("{}:{edges}", family.label())
}

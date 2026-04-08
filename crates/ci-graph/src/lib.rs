use std::collections::BTreeMap;

use ci_ir::{RepoIr, RepoStats};
use ci_resolver::{ImpactReport, ResolutionResult, SymbolContext, resolve};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphNodeKind {
    Repo,
    File,
    Symbol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphEdgeKind {
    Contains,
    Imports,
    Calls,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub kind: GraphNodeKind,
    pub label: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub kind: GraphEdgeKind,
    pub source: String,
    pub target: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphBundle {
    pub schema_version: u32,
    pub repo_name: String,
    pub repo_root: String,
    pub stats: RepoStats,
    pub ir: RepoIr,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub symbol_contexts: IndexMap<String, SymbolContext>,
    pub impact_reports: IndexMap<String, ImpactReport>,
}

impl GraphBundle {
    pub fn build(ir: RepoIr) -> Self {
        let resolution = resolve(&ir);
        build_graph(ir, resolution)
    }

    pub fn symbol_matches(&self, query: &str) -> Vec<&SymbolContext> {
        let mut results = self
            .symbol_contexts
            .values()
            .filter(|symbol| {
                symbol.name == query
                    || symbol.qualified_name == query
                    || symbol.qualified_name.ends_with(&format!("::{query}"))
            })
            .collect::<Vec<_>>();
        results.sort_by(|left, right| left.qualified_name.cmp(&right.qualified_name));
        results
    }

    pub fn impact_for(&self, query: &str) -> Vec<&ImpactReport> {
        let mut results = self
            .impact_reports
            .values()
            .filter(|impact| {
                impact.name == query
                    || impact.qualified_name == query
                    || impact.qualified_name.ends_with(&format!("::{query}"))
            })
            .collect::<Vec<_>>();
        results.sort_by(|left, right| left.qualified_name.cmp(&right.qualified_name));
        results
    }
}

fn build_graph(ir: RepoIr, resolution: ResolutionResult) -> GraphBundle {
    let repo_id = format!("repo:{}", ir.repo_name);
    let mut nodes = vec![GraphNode {
        id: repo_id.clone(),
        kind: GraphNodeKind::Repo,
        label: ir.repo_name.clone(),
        path: Some(ir.repo_root.to_string()),
    }];
    let mut edges = Vec::new();
    let mut symbol_lookup = BTreeMap::new();

    for file in &ir.files {
        nodes.push(GraphNode {
            id: file.file_id.clone(),
            kind: GraphNodeKind::File,
            label: file.rel_path.to_string(),
            path: Some(file.rel_path.to_string()),
        });
        edges.push(GraphEdge {
            kind: GraphEdgeKind::Contains,
            source: repo_id.clone(),
            target: file.file_id.clone(),
            evidence: file.module_path.clone(),
        });

        for symbol in &file.symbols {
            symbol_lookup.insert(symbol.symbol_id.clone(), symbol.qualified_name.clone());
            nodes.push(GraphNode {
                id: symbol.symbol_id.clone(),
                kind: GraphNodeKind::Symbol,
                label: symbol.qualified_name.clone(),
                path: Some(format!("{}:{}", symbol.file_path, symbol.span.start_line)),
            });
            edges.push(GraphEdge {
                kind: GraphEdgeKind::Contains,
                source: file.file_id.clone(),
                target: symbol.symbol_id.clone(),
                evidence: format!("{:?}", symbol.kind).to_lowercase(),
            });
        }
    }

    for import_link in &resolution.import_links {
        edges.push(GraphEdge {
            kind: GraphEdgeKind::Imports,
            source: import_link.importer_file_id.clone(),
            target: import_link.imported_file_id.clone(),
            evidence: import_link.via.clone(),
        });
    }

    for call_link in &resolution.call_links {
        edges.push(GraphEdge {
            kind: GraphEdgeKind::Calls,
            source: call_link.caller_symbol_id.clone(),
            target: call_link.callee_symbol_id.clone(),
            evidence: call_link.target_name.clone(),
        });
    }

    GraphBundle {
        schema_version: 1,
        repo_name: ir.repo_name.clone(),
        repo_root: ir.repo_root.to_string(),
        stats: ir.stats(),
        ir,
        nodes,
        edges,
        symbol_contexts: resolution.symbol_contexts,
        impact_reports: resolution.impact_reports,
    }
}

use ci_graph::GraphBundle;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub qualified_name: String,
    pub file_path: String,
    pub line: usize,
    pub reason: String,
}

pub fn search_symbols(graph: &GraphBundle, query: &str) -> Vec<SearchHit> {
    let mut hits = graph
        .symbol_contexts
        .values()
        .filter(|symbol| {
            symbol.name.contains(query)
                || symbol.qualified_name.contains(query)
                || symbol.file_path.contains(query)
        })
        .map(|symbol| SearchHit {
            qualified_name: symbol.qualified_name.clone(),
            file_path: symbol.file_path.clone(),
            line: symbol.line,
            reason: format!("matched symbol {}", symbol.name),
        })
        .collect::<Vec<_>>();
    hits.sort_by(|left, right| left.qualified_name.cmp(&right.qualified_name));
    hits
}

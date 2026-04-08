use ci_graph::GraphBundle;
use ci_search::search_symbols;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainAnswer {
    pub query: String,
    pub summary: String,
    pub evidence: Vec<String>,
}

pub fn explain_query(graph: &GraphBundle, query: &str) -> ExplainAnswer {
    let symbols = graph.symbol_matches(query);
    if !symbols.is_empty() {
        let symbol = symbols[0];
        return ExplainAnswer {
            query: query.to_string(),
            summary: format!(
                "{} is defined in {}:{} with {} caller(s), {} callee(s), and {} importing file(s).",
                symbol.qualified_name,
                symbol.file_path,
                symbol.line,
                symbol.callers.len(),
                symbol.callees.len(),
                symbol.importing_files.len()
            ),
            evidence: vec![
                format!("definition: {}:{}", symbol.file_path, symbol.line),
                format!("callers: {}", symbol.callers.join(", ")),
                format!("callees: {}", symbol.callees.join(", ")),
            ],
        };
    }

    let hits = search_symbols(graph, query);
    ExplainAnswer {
        query: query.to_string(),
        summary: format!(
            "No exact symbol match found for {query}. {} related symbol(s) matched.",
            hits.len()
        ),
        evidence: hits
            .into_iter()
            .take(5)
            .map(|hit| format!("{} at {}:{}", hit.qualified_name, hit.file_path, hit.line))
            .collect(),
    }
}

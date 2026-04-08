use std::collections::{BTreeMap, BTreeSet};

use ci_ir::{RepoIr, SymbolIr};
use indexmap::IndexMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedReference {
    pub caller_symbol_id: String,
    pub callee_symbol_id: String,
    pub target_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileImportLink {
    pub importer_file_id: String,
    pub importer_path: String,
    pub imported_file_id: String,
    pub imported_path: String,
    pub via: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolContext {
    pub symbol_id: String,
    pub name: String,
    pub qualified_name: String,
    pub kind: String,
    pub file_path: String,
    pub line: usize,
    pub signature: Option<String>,
    pub callers: Vec<String>,
    pub callees: Vec<String>,
    pub same_file_symbols: Vec<String>,
    pub importing_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub symbol_id: String,
    pub name: String,
    pub qualified_name: String,
    pub summary: String,
    pub blast_radius_files: Vec<String>,
    pub blast_radius_symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResult {
    pub call_links: Vec<ResolvedReference>,
    pub import_links: Vec<FileImportLink>,
    pub symbol_contexts: IndexMap<String, SymbolContext>,
    pub impact_reports: IndexMap<String, ImpactReport>,
}

pub fn resolve(ir: &RepoIr) -> ResolutionResult {
    let mut symbols_by_name: BTreeMap<String, Vec<&SymbolIr>> = BTreeMap::new();
    let mut symbols_by_id: BTreeMap<String, &SymbolIr> = BTreeMap::new();
    let mut file_lookup = BTreeMap::new();

    for file in &ir.files {
        file_lookup.insert(file.file_id.clone(), file.rel_path.to_string());
        for symbol in &file.symbols {
            symbols_by_name
                .entry(symbol.name.clone())
                .or_default()
                .push(symbol);
            symbols_by_id.insert(symbol.symbol_id.clone(), symbol);
        }
    }

    let mut import_links = Vec::new();
    for file in &ir.files {
        for import in &file.imports {
            let candidates = ir
                .files
                .iter()
                .filter(|candidate| candidate.file_id != file.file_id)
                .filter(|candidate| {
                    import_matches(
                        &candidate.rel_path.to_string(),
                        import.source.as_deref(),
                        &import.names,
                    )
                })
                .collect_vec();

            if candidates.len() == 1 {
                let candidate = candidates[0];
                import_links.push(FileImportLink {
                    importer_file_id: file.file_id.clone(),
                    importer_path: file.rel_path.to_string(),
                    imported_file_id: candidate.file_id.clone(),
                    imported_path: candidate.rel_path.to_string(),
                    via: import
                        .source
                        .clone()
                        .unwrap_or_else(|| import.names.join(",")),
                });
            }
        }
    }

    let mut call_links = Vec::new();
    for file in &ir.files {
        for callsite in &file.callsites {
            let Some(caller_symbol_id) = callsite.caller_symbol_id.clone() else {
                continue;
            };
            let Some(candidates) = symbols_by_name.get(&callsite.target_name) else {
                continue;
            };
            if candidates.len() == 1 {
                call_links.push(ResolvedReference {
                    caller_symbol_id,
                    callee_symbol_id: candidates[0].symbol_id.clone(),
                    target_name: callsite.target_name.clone(),
                });
            }
        }
    }

    let mut reverse_callers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut forward_callees: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for link in &call_links {
        reverse_callers
            .entry(link.callee_symbol_id.clone())
            .or_default()
            .insert(link.caller_symbol_id.clone());
        forward_callees
            .entry(link.caller_symbol_id.clone())
            .or_default()
            .insert(link.callee_symbol_id.clone());
    }

    let mut importing_files_by_file: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for link in &import_links {
        importing_files_by_file
            .entry(link.imported_file_id.clone())
            .or_default()
            .insert(link.importer_path.clone());
    }

    let mut symbol_contexts = IndexMap::new();
    let mut impact_reports = IndexMap::new();

    for file in &ir.files {
        let same_file_symbols = file
            .symbols
            .iter()
            .map(|symbol| symbol.name.clone())
            .collect::<Vec<_>>();

        for symbol in &file.symbols {
            let callers = reverse_callers
                .get(&symbol.symbol_id)
                .into_iter()
                .flatten()
                .filter_map(|id| symbols_by_id.get(id))
                .map(|item| item.qualified_name.clone())
                .collect::<Vec<_>>();
            let callees = forward_callees
                .get(&symbol.symbol_id)
                .into_iter()
                .flatten()
                .filter_map(|id| symbols_by_id.get(id))
                .map(|item| item.qualified_name.clone())
                .collect::<Vec<_>>();
            let importing_files = importing_files_by_file
                .get(&symbol.file_id)
                .into_iter()
                .flatten()
                .cloned()
                .collect::<Vec<_>>();

            symbol_contexts.insert(
                symbol.symbol_id.clone(),
                SymbolContext {
                    symbol_id: symbol.symbol_id.clone(),
                    name: symbol.name.clone(),
                    qualified_name: symbol.qualified_name.clone(),
                    kind: format!("{:?}", symbol.kind).to_lowercase(),
                    file_path: symbol.file_path.to_string(),
                    line: symbol.span.start_line,
                    signature: symbol.signature.clone(),
                    callers: callers.clone(),
                    callees: callees.clone(),
                    same_file_symbols: same_file_symbols
                        .iter()
                        .filter(|candidate| *candidate != &symbol.name)
                        .cloned()
                        .collect(),
                    importing_files: importing_files.clone(),
                },
            );

            let mut blast_radius_files = BTreeSet::new();
            blast_radius_files.insert(symbol.file_path.to_string());
            for importing in &importing_files {
                blast_radius_files.insert(importing.clone());
            }
            for caller in &callers {
                if let Some(caller_symbol) = symbols_by_id
                    .values()
                    .find(|candidate| candidate.qualified_name == *caller)
                {
                    blast_radius_files.insert(caller_symbol.file_path.to_string());
                }
            }

            let mut blast_radius_symbols = BTreeSet::new();
            blast_radius_symbols.extend(callers.iter().cloned());
            blast_radius_symbols.extend(callees.iter().cloned());
            blast_radius_symbols.extend(
                same_file_symbols
                    .iter()
                    .filter(|candidate| *candidate != &symbol.name)
                    .cloned(),
            );

            impact_reports.insert(
                symbol.symbol_id.clone(),
                ImpactReport {
                    symbol_id: symbol.symbol_id.clone(),
                    name: symbol.name.clone(),
                    qualified_name: symbol.qualified_name.clone(),
                    summary: format!(
                        "{} is defined in {} and is connected to {} caller(s), {} callee(s), and {} importing file(s).",
                        symbol.qualified_name,
                        symbol.file_path,
                        callers.len(),
                        callees.len(),
                        importing_files.len()
                    ),
                    blast_radius_files: blast_radius_files.into_iter().collect(),
                    blast_radius_symbols: blast_radius_symbols.into_iter().collect(),
                },
            );
        }
    }

    ResolutionResult {
        call_links,
        import_links,
        symbol_contexts,
        impact_reports,
    }
}

fn import_matches(candidate_path: &str, source_hint: Option<&str>, names: &[String]) -> bool {
    let Some(source_hint) = source_hint else {
        return names.iter().any(|name| {
            candidate_path.ends_with(&format!("/{name}.rs")) || candidate_path.contains(name)
        });
    };

    let normalized = source_hint.trim_matches('.');
    candidate_path.contains(normalized)
        || candidate_path.ends_with(&format!("{normalized}.rs"))
        || candidate_path.ends_with(&format!("{normalized}.ts"))
        || candidate_path.ends_with(&format!("{normalized}.js"))
        || candidate_path.ends_with(&format!("{normalized}.py"))
        || names.iter().any(|name| candidate_path.contains(name))
}

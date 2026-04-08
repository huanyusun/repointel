use anyhow::{Result, anyhow};
use ci_ir::{
    CallsiteIr, FileIr, ImportIr, Language, RepoIr, SourceFile, Span, SymbolIr, SymbolKind,
};
use ci_loader::LoadOptions;
use ci_loader::load_local_repository;
use indexmap::IndexMap;
use tree_sitter::{Node, Parser, TreeCursor};

#[derive(Debug, Clone, Default)]
pub struct NativeParser;

impl NativeParser {
    pub fn parse_local_repository(path: &camino::Utf8Path) -> Result<RepoIr> {
        let snapshot = load_local_repository(path, &LoadOptions::default())?;
        Self::parse_snapshot(snapshot)
    }

    pub fn parse_snapshot(snapshot: ci_ir::RepositorySnapshot) -> Result<RepoIr> {
        let mut files = Vec::new();
        let mut language_counts = IndexMap::new();

        for source in snapshot.files {
            *language_counts.entry(source.language).or_insert(0) += 1;
            files.push(parse_file(&source)?);
        }

        Ok(RepoIr {
            repo_name: snapshot.repo_name,
            repo_root: snapshot.repo_root,
            files,
            language_counts,
        })
    }
}

fn parse_file(source: &SourceFile) -> Result<FileIr> {
    let tree = parse_tree(source)?;
    let root = tree.root_node();
    let mut state = FileState::new(source);
    let mut context = ParseContext::default();
    visit_node(root, &source.content, &mut state, &mut context);

    Ok(FileIr {
        file_id: source.file_id.clone(),
        rel_path: source.rel_path.clone(),
        module_path: source.module_path.clone(),
        language: source.language,
        digest: source.digest.clone(),
        symbols: state.symbols,
        imports: state.imports,
        callsites: state.callsites,
        is_test: is_test_path(source.rel_path.as_str()),
    })
}

fn parse_tree(source: &SourceFile) -> Result<tree_sitter::Tree> {
    let mut parser = Parser::new();
    match source.language {
        Language::Rust => parser
            .set_language(tree_sitter_rust::language())
            .map_err(|error| anyhow!("failed to configure Rust parser: {error}"))?,
        Language::TypeScript => parser
            .set_language(tree_sitter_typescript::language_typescript())
            .map_err(|error| anyhow!("failed to configure TypeScript parser: {error}"))?,
        Language::JavaScript => parser
            .set_language(tree_sitter_javascript::language())
            .map_err(|error| anyhow!("failed to configure JavaScript parser: {error}"))?,
        Language::Python => parser
            .set_language(tree_sitter_python::language())
            .map_err(|error| anyhow!("failed to configure Python parser: {error}"))?,
        Language::Unknown => {
            return Err(anyhow!("cannot parse unknown language {}", source.rel_path));
        }
    }

    parser
        .parse(&source.content, None)
        .ok_or_else(|| anyhow!("failed to parse {}", source.rel_path))
}

#[derive(Debug)]
struct FileState<'a> {
    source: &'a SourceFile,
    symbols: Vec<SymbolIr>,
    imports: Vec<ImportIr>,
    callsites: Vec<CallsiteIr>,
}

impl<'a> FileState<'a> {
    fn new(source: &'a SourceFile) -> Self {
        Self {
            source,
            symbols: Vec::new(),
            imports: Vec::new(),
            callsites: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ParseContext {
    symbol_stack: Vec<(String, String)>,
}

fn visit_node(node: Node<'_>, source: &str, state: &mut FileState<'_>, context: &mut ParseContext) {
    let symbol = extract_symbol(node, source, state.source, context);
    if let Some(symbol) = symbol.as_ref() {
        context
            .symbol_stack
            .push((symbol.symbol_id.clone(), symbol.qualified_name.clone()));
        state.symbols.push(symbol.clone());
    }

    if let Some(import) = extract_import(node, source, state.source) {
        state.imports.push(import);
    }
    if let Some(callsite) = extract_callsite(node, source, state.source, context) {
        state.callsites.push(callsite);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit_node(child, source, state, context);
    }

    if symbol.is_some() {
        context.symbol_stack.pop();
    }
}

fn extract_symbol(
    node: Node<'_>,
    source: &str,
    file: &SourceFile,
    context: &ParseContext,
) -> Option<SymbolIr> {
    let (kind, name) = match file.language {
        Language::Rust => rust_symbol(node, source),
        Language::TypeScript | Language::JavaScript => js_symbol(node, source),
        Language::Python => python_symbol(node, source),
        Language::Unknown => None,
    }?;

    let parent_qualified = context
        .symbol_stack
        .last()
        .map(|(_, qualified)| qualified.clone());
    let qualified_name = if let Some(parent) = &parent_qualified {
        format!("{parent}::{name}")
    } else {
        format!("{}::{name}", file.module_path)
    };
    let span = node_span(node);
    let symbol_id = format!(
        "symbol:{}:{}:{}",
        file.rel_path, qualified_name, span.start_line
    );

    Some(SymbolIr {
        symbol_id,
        name,
        qualified_name,
        kind,
        file_id: file.file_id.clone(),
        file_path: file.rel_path.clone(),
        span,
        container: parent_qualified,
        signature: Some(
            node.utf8_text(source.as_bytes())
                .ok()?
                .lines()
                .next()?
                .trim()
                .to_string(),
        ),
    })
}

fn rust_symbol(node: Node<'_>, source: &str) -> Option<(SymbolKind, String)> {
    let kind = match node.kind() {
        "function_item" => SymbolKind::Function,
        "struct_item" => SymbolKind::Struct,
        "enum_item" => SymbolKind::Enum,
        "trait_item" => SymbolKind::Trait,
        "impl_item" => SymbolKind::Impl,
        "mod_item" => SymbolKind::Module,
        "const_item" => SymbolKind::Constant,
        "type_item" => SymbolKind::TypeAlias,
        _ => return None,
    };
    let name = node.child_by_field_name("name")?;
    Some((kind, node_text(name, source)))
}

fn js_symbol(node: Node<'_>, source: &str) -> Option<(SymbolKind, String)> {
    match node.kind() {
        "function_declaration" => Some((
            SymbolKind::Function,
            node_text(node.child_by_field_name("name")?, source),
        )),
        "class_declaration" => Some((
            SymbolKind::Class,
            node_text(node.child_by_field_name("name")?, source),
        )),
        "method_definition" => Some((
            SymbolKind::Method,
            node_text(node.child_by_field_name("name")?, source),
        )),
        "interface_declaration" => Some((
            SymbolKind::Interface,
            node_text(node.child_by_field_name("name")?, source),
        )),
        "lexical_declaration" | "variable_declaration" => {
            let declarator = find_first_named_descendant(node, "variable_declarator")?;
            let name = declarator.child_by_field_name("name")?;
            Some((SymbolKind::Variable, node_text(name, source)))
        }
        _ => None,
    }
}

fn python_symbol(node: Node<'_>, source: &str) -> Option<(SymbolKind, String)> {
    match node.kind() {
        "function_definition" => Some((
            SymbolKind::Function,
            node_text(node.child_by_field_name("name")?, source),
        )),
        "class_definition" => Some((
            SymbolKind::Class,
            node_text(node.child_by_field_name("name")?, source),
        )),
        _ => None,
    }
}

fn extract_import(node: Node<'_>, source: &str, file: &SourceFile) -> Option<ImportIr> {
    let is_reexport = matches!(node.kind(), "export_statement");
    let should_capture = match file.language {
        Language::Rust => node.kind() == "use_declaration",
        Language::TypeScript | Language::JavaScript => {
            node.kind() == "import_statement" || node.kind() == "export_statement"
        }
        Language::Python => {
            node.kind() == "import_statement" || node.kind() == "import_from_statement"
        }
        Language::Unknown => false,
    };
    if !should_capture {
        return None;
    }

    let raw = node_text(node, source);
    let names = raw
        .replace(['{', '}', ',', ';', '(', ')'], " ")
        .split_whitespace()
        .filter(|token| !matches!(*token, "use" | "import" | "from" | "as" | "export"))
        .filter(|token| token.chars().any(|character| character.is_alphabetic()))
        .map(str::to_string)
        .collect::<Vec<_>>();

    let source_hint = raw
        .split(['"', '\''])
        .nth(1)
        .map(str::to_string)
        .or_else(|| {
            raw.split_whitespace()
                .last()
                .map(|segment| segment.trim_end_matches(';').to_string())
        });
    let span = node_span(node);

    Some(ImportIr {
        import_id: format!("import:{}:{}:{}", file.rel_path, span.start_line, raw),
        file_id: file.file_id.clone(),
        source: source_hint,
        names,
        is_reexport,
        span,
    })
}

fn extract_callsite(
    node: Node<'_>,
    source: &str,
    file: &SourceFile,
    context: &ParseContext,
) -> Option<CallsiteIr> {
    if node.kind() != "call_expression" {
        return None;
    }

    let function_node = node.child_by_field_name("function")?;
    let target_name = extract_callable_name(function_node, source)?;
    let span = node_span(node);
    Some(CallsiteIr {
        call_id: format!("call:{}:{}:{}", file.rel_path, span.start_line, target_name),
        file_id: file.file_id.clone(),
        caller_symbol_id: context.symbol_stack.last().map(|(id, _)| id.clone()),
        target_name,
        span,
    })
}

fn extract_callable_name(node: Node<'_>, source: &str) -> Option<String> {
    match node.kind() {
        "identifier" | "property_identifier" | "field_identifier" => Some(node_text(node, source)),
        "scoped_identifier" | "qualified_identifier" | "attribute" | "member_expression" => {
            let text = node_text(node, source);
            text.split(['.', ':']).last().map(str::to_string)
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if let Some(name) = extract_callable_name(child, source) {
                    return Some(name);
                }
            }
            None
        }
    }
}

fn find_first_named_descendant<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    if node.kind() == kind {
        return Some(node);
    }
    let mut cursor: TreeCursor<'_> = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_first_named_descendant(child, kind) {
            return Some(found);
        }
    }
    None
}

fn node_text(node: Node<'_>, source: &str) -> String {
    node.utf8_text(source.as_bytes())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn node_span(node: Node<'_>) -> Span {
    Span::from_positions(
        node.start_position().row,
        node.start_position().column,
        node.end_position().row,
        node.end_position().column,
        node.start_byte(),
        node.end_byte(),
    )
}

fn is_test_path(path: &str) -> bool {
    path.contains("/tests/")
        || path.ends_with("_test.py")
        || path.ends_with(".test.ts")
        || path.ends_with(".test.js")
        || path.ends_with("_spec.ts")
        || path.ends_with("_spec.js")
}

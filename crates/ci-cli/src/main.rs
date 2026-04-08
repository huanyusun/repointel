use std::fs;

use anyhow::{Context, Result, anyhow, bail};
use camino::{Utf8Path, Utf8PathBuf};
use ci_agent::explain_query;
use ci_graph::GraphBundle;
use ci_loader::{LoadOptions, load_local_repository};
use ci_parser_native::NativeParser;
use ci_search::search_symbols;
use clap::{Args, Parser, Subcommand};
use serde::Serialize;

const INDEX_DIR: &str = ".repointel";
const INDEX_FILE: &str = "index.json";

#[derive(Parser, Debug)]
#[command(name = "repointel")]
#[command(about = "Repository intelligence for humans and AI coding agents")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Index(IndexArgs),
    Status(JsonFlag),
    Search(QueryArgs),
    Symbol(QueryArgs),
    Impact(QueryArgs),
    Callers(QueryArgs),
    Callees(QueryArgs),
    Trace(QueryArgs),
    Graph(GraphCommand),
    Explain(QueryArgs),
}

#[derive(Args, Debug)]
struct IndexArgs {
    path: Utf8PathBuf,
    #[arg(long)]
    watch: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct QueryArgs {
    query: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct JsonFlag {
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum GraphSubcommand {
    Export(GraphExportArgs),
}

#[derive(Args, Debug)]
struct GraphCommand {
    #[command(subcommand)]
    command: GraphSubcommand,
}

#[derive(Args, Debug)]
struct GraphExportArgs {
    #[arg(long, default_value = "json")]
    format: String,
}

#[derive(Debug, Serialize)]
struct IndexResponse {
    repo_root: String,
    repo_name: String,
    index_path: String,
    files: usize,
    symbols: usize,
    imports: usize,
    callsites: usize,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    repo_root: String,
    repo_name: String,
    index_path: String,
    files: usize,
    symbols: usize,
    imports: usize,
    callsites: usize,
    schema_version: u32,
}

#[derive(Debug, Serialize)]
struct SymbolResponse {
    matches: Vec<ci_resolver::SymbolContext>,
}

#[derive(Debug, Serialize)]
struct ImpactResponse {
    matches: Vec<ci_resolver::ImpactReport>,
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    hits: Vec<ci_search::SearchHit>,
}

#[derive(Debug, Serialize)]
struct CallChainResponse {
    query: String,
    entries: Vec<String>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Index(args) => handle_index(args),
        Command::Status(args) => handle_status(args.json),
        Command::Search(args) => handle_search(&args.query, args.json),
        Command::Symbol(args) => handle_symbol(&args.query, args.json),
        Command::Impact(args) => handle_impact(&args.query, args.json),
        Command::Callers(args) => handle_callers(&args.query, args.json),
        Command::Callees(args) => handle_callees(&args.query, args.json),
        Command::Trace(args) => handle_trace(&args.query, args.json),
        Command::Graph(command) => match command.command {
            GraphSubcommand::Export(args) => handle_graph_export(&args.format),
        },
        Command::Explain(args) => handle_explain(&args.query, args.json),
    }
}

fn handle_index(args: IndexArgs) -> Result<()> {
    if args.watch {
        bail!("watch mode is planned but not implemented in the thin slice");
    }

    let repo_root = args
        .path
        .canonicalize_utf8()
        .with_context(|| format!("failed to resolve {}", args.path))?;
    let snapshot = load_local_repository(&repo_root, &LoadOptions::default())?;
    let ir = NativeParser::parse_snapshot(snapshot)?;
    let graph = GraphBundle::build(ir);
    persist_graph(&repo_root, &graph)?;

    let response = IndexResponse {
        repo_root: repo_root.to_string(),
        repo_name: graph.repo_name.clone(),
        index_path: index_path(&repo_root).to_string(),
        files: graph.stats.files,
        symbols: graph.stats.symbols,
        imports: graph.stats.imports,
        callsites: graph.stats.callsites,
    };
    render(&response, args.json, |value| {
        println!(
            "Indexed {} at {}\nfiles: {}\nsymbols: {}\nimports: {}\ncallsites: {}\nindex: {}",
            value.repo_name,
            value.repo_root,
            value.files,
            value.symbols,
            value.imports,
            value.callsites,
            value.index_path
        )
    })
}

fn handle_status(json: bool) -> Result<()> {
    let (repo_root, graph) = load_graph_from_cwd()?;
    let response = StatusResponse {
        repo_root: repo_root.to_string(),
        repo_name: graph.repo_name.clone(),
        index_path: index_path(&repo_root).to_string(),
        files: graph.stats.files,
        symbols: graph.stats.symbols,
        imports: graph.stats.imports,
        callsites: graph.stats.callsites,
        schema_version: graph.schema_version,
    };
    render(&response, json, |value| {
        println!(
            "{}\nfiles: {}\nsymbols: {}\nimports: {}\ncallsites: {}\nschema: {}",
            value.repo_name,
            value.files,
            value.symbols,
            value.imports,
            value.callsites,
            value.schema_version
        )
    })
}

fn handle_search(query: &str, json: bool) -> Result<()> {
    let (_, graph) = load_graph_from_cwd()?;
    let response = SearchResponse {
        hits: search_symbols(&graph, query),
    };
    render(&response, json, |value| {
        for hit in &value.hits {
            println!(
                "{} [{}:{}] {}",
                hit.qualified_name, hit.file_path, hit.line, hit.reason
            );
        }
    })
}

fn handle_symbol(query: &str, json: bool) -> Result<()> {
    let (_, graph) = load_graph_from_cwd()?;
    let response = SymbolResponse {
        matches: graph.symbol_matches(query).into_iter().cloned().collect(),
    };
    render(&response, json, |value| {
        for symbol in &value.matches {
            println!(
                "{}\n  file: {}:{}\n  callers: {}\n  callees: {}\n  importing files: {}",
                symbol.qualified_name,
                symbol.file_path,
                symbol.line,
                symbol.callers.join(", "),
                symbol.callees.join(", "),
                symbol.importing_files.join(", ")
            );
        }
    })
}

fn handle_impact(query: &str, json: bool) -> Result<()> {
    let (_, graph) = load_graph_from_cwd()?;
    let response = ImpactResponse {
        matches: graph.impact_for(query).into_iter().cloned().collect(),
    };
    render(&response, json, |value| {
        for impact in &value.matches {
            println!(
                "{}\n  {}\n  files: {}\n  symbols: {}",
                impact.qualified_name,
                impact.summary,
                impact.blast_radius_files.join(", "),
                impact.blast_radius_symbols.join(", ")
            );
        }
    })
}

fn handle_callers(query: &str, json: bool) -> Result<()> {
    let (_, graph) = load_graph_from_cwd()?;
    let entries = graph
        .symbol_matches(query)
        .into_iter()
        .flat_map(|symbol| symbol.callers.clone())
        .collect::<Vec<_>>();
    let response = CallChainResponse {
        query: query.to_string(),
        entries,
    };
    render(&response, json, |value| {
        for entry in &value.entries {
            println!("{entry}");
        }
    })
}

fn handle_callees(query: &str, json: bool) -> Result<()> {
    let (_, graph) = load_graph_from_cwd()?;
    let entries = graph
        .symbol_matches(query)
        .into_iter()
        .flat_map(|symbol| symbol.callees.clone())
        .collect::<Vec<_>>();
    let response = CallChainResponse {
        query: query.to_string(),
        entries,
    };
    render(&response, json, |value| {
        for entry in &value.entries {
            println!("{entry}");
        }
    })
}

fn handle_trace(query: &str, json: bool) -> Result<()> {
    let (_, graph) = load_graph_from_cwd()?;
    let matches = graph.symbol_matches(query);
    let response = CallChainResponse {
        query: query.to_string(),
        entries: matches
            .first()
            .map(|symbol| {
                let mut entries = Vec::new();
                entries.push(format!("definition {}:{} ", symbol.file_path, symbol.line));
                entries.extend(
                    symbol
                        .callees
                        .iter()
                        .map(|callee| format!("calls {callee}")),
                );
                entries
            })
            .unwrap_or_default(),
    };
    render(&response, json, |value| {
        for entry in &value.entries {
            println!("{entry}");
        }
    })
}

fn handle_graph_export(format: &str) -> Result<()> {
    let (_, graph) = load_graph_from_cwd()?;
    if format != "json" {
        bail!("only json export is supported in the thin slice");
    }
    println!("{}", serde_json::to_string_pretty(&graph)?);
    Ok(())
}

fn handle_explain(query: &str, json: bool) -> Result<()> {
    let (_, graph) = load_graph_from_cwd()?;
    let answer = explain_query(&graph, query);
    render(&answer, json, |value| {
        println!("{}", value.summary);
        for item in &value.evidence {
            println!("- {item}");
        }
    })
}

fn render<T>(value: &T, json: bool, human: impl FnOnce(&T)) -> Result<()>
where
    T: Serialize,
{
    if json {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        human(value);
    }
    Ok(())
}

fn persist_graph(repo_root: &Utf8Path, graph: &GraphBundle) -> Result<()> {
    let directory = repo_root.join(INDEX_DIR);
    fs::create_dir_all(directory.as_std_path())
        .with_context(|| format!("failed to create {}", directory))?;
    fs::write(
        index_path(repo_root).as_std_path(),
        serde_json::to_vec_pretty(graph)?,
    )
    .with_context(|| format!("failed to write {}", index_path(repo_root)))?;
    Ok(())
}

fn load_graph_from_cwd() -> Result<(Utf8PathBuf, GraphBundle)> {
    let cwd = std::env::current_dir().context("failed to read current directory")?;
    let cwd = Utf8PathBuf::from_path_buf(cwd).map_err(|_| anyhow!("cwd was not valid UTF-8"))?;

    for candidate in cwd.ancestors() {
        let index = index_path(candidate);
        if index.exists() {
            let bytes = fs::read(index.as_std_path())
                .with_context(|| format!("failed to read {}", index))?;
            let graph = serde_json::from_slice::<GraphBundle>(&bytes)
                .with_context(|| format!("failed to parse {}", index))?;
            return Ok((candidate.to_path_buf(), graph));
        }
    }

    bail!(
        "could not find {} in the current directory or its ancestors",
        INDEX_FILE
    )
}

fn index_path(repo_root: &Utf8Path) -> Utf8PathBuf {
    repo_root.join(INDEX_DIR).join(INDEX_FILE)
}

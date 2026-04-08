use std::collections::BTreeSet;

use camino::Utf8PathBuf;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Unknown,
}

impl Language {
    pub fn from_extension(path: &str) -> Self {
        if path.ends_with(".rs") {
            Self::Rust
        } else if path.ends_with(".ts") || path.ends_with(".tsx") {
            Self::TypeScript
        } else if path.ends_with(".js") || path.ends_with(".jsx") || path.ends_with(".mjs") {
            Self::JavaScript
        } else if path.ends_with(".py") {
            Self::Python
        } else {
            Self::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Module,
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Impl,
    Class,
    Interface,
    TypeAlias,
    Constant,
    Variable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl Span {
    pub fn from_positions(
        start_row: usize,
        start_column: usize,
        end_row: usize,
        end_column: usize,
        start_byte: usize,
        end_byte: usize,
    ) -> Self {
        Self {
            start_line: start_row + 1,
            start_column: start_column + 1,
            end_line: end_row + 1,
            end_column: end_column + 1,
            start_byte,
            end_byte,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySnapshot {
    pub repo_root: Utf8PathBuf,
    pub repo_name: String,
    pub files: Vec<SourceFile>,
    pub ignored_paths: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    pub file_id: String,
    pub abs_path: Utf8PathBuf,
    pub rel_path: Utf8PathBuf,
    pub module_path: String,
    pub language: Language,
    pub content: String,
    pub digest: String,
}

impl SourceFile {
    pub fn new(
        repo_root: &camino::Utf8Path,
        abs_path: Utf8PathBuf,
        rel_path: Utf8PathBuf,
        content: String,
    ) -> Self {
        let digest = sha256(&content);
        let module_path = module_path_from_rel(&rel_path);
        let file_id = format!("file:{}", rel_path);

        Self {
            file_id,
            abs_path: repo_root.join(abs_path.strip_prefix(repo_root).unwrap_or(&rel_path)),
            rel_path: rel_path.clone(),
            module_path,
            language: Language::from_extension(rel_path.as_str()),
            content,
            digest,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoIr {
    pub repo_name: String,
    pub repo_root: Utf8PathBuf,
    pub files: Vec<FileIr>,
    pub language_counts: IndexMap<Language, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIr {
    pub file_id: String,
    pub rel_path: Utf8PathBuf,
    pub module_path: String,
    pub language: Language,
    pub digest: String,
    pub symbols: Vec<SymbolIr>,
    pub imports: Vec<ImportIr>,
    pub callsites: Vec<CallsiteIr>,
    pub is_test: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolIr {
    pub symbol_id: String,
    pub name: String,
    pub qualified_name: String,
    pub kind: SymbolKind,
    pub file_id: String,
    pub file_path: Utf8PathBuf,
    pub span: Span,
    pub container: Option<String>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportIr {
    pub import_id: String,
    pub file_id: String,
    pub source: Option<String>,
    pub names: Vec<String>,
    pub is_reexport: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallsiteIr {
    pub call_id: String,
    pub file_id: String,
    pub caller_symbol_id: Option<String>,
    pub target_name: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStats {
    pub files: usize,
    pub symbols: usize,
    pub imports: usize,
    pub callsites: usize,
}

impl RepoIr {
    pub fn stats(&self) -> RepoStats {
        let files = self.files.len();
        let symbols = self.files.iter().map(|file| file.symbols.len()).sum();
        let imports = self.files.iter().map(|file| file.imports.len()).sum();
        let callsites = self.files.iter().map(|file| file.callsites.len()).sum();
        RepoStats {
            files,
            symbols,
            imports,
            callsites,
        }
    }
}

pub fn module_path_from_rel(rel_path: &camino::Utf8Path) -> String {
    let normalized = rel_path
        .as_str()
        .trim_end_matches(".rs")
        .trim_end_matches(".ts")
        .trim_end_matches(".tsx")
        .trim_end_matches(".js")
        .trim_end_matches(".jsx")
        .trim_end_matches(".mjs")
        .trim_end_matches(".py")
        .replace('/', "::");
    normalized.trim_end_matches("::mod").to_string()
}

pub fn sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

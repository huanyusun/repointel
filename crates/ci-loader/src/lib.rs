use std::collections::BTreeSet;
use std::fs;

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use ci_ir::{Language, RepositorySnapshot, SourceFile, module_path_from_rel, sha256};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadOptions {
    pub ignore_dirs: BTreeSet<String>,
    pub include_hidden: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            ignore_dirs: [
                ".git",
                "node_modules",
                "target",
                "dist",
                "build",
                ".next",
                "__pycache__",
                ".venv",
                "vendor",
                "coverage",
                ".turbo",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            include_hidden: false,
        }
    }
}

pub fn load_local_repository(root: &Utf8Path, options: &LoadOptions) -> Result<RepositorySnapshot> {
    let repo_root = root
        .canonicalize_utf8()
        .with_context(|| format!("failed to canonicalize repository root {}", root))?;
    let repo_name = repo_root
        .file_name()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "repository".to_string());

    let mut walk = WalkBuilder::new(repo_root.as_std_path());
    walk.hidden(!options.include_hidden);
    walk.git_ignore(true);
    walk.git_exclude(true);
    walk.git_global(true);
    walk.parents(true);
    for ignored in &options.ignore_dirs {
        walk.add_custom_ignore_filename(ignored);
    }

    let mut files = Vec::new();
    let mut ignored_paths = BTreeSet::new();

    for result in walk.build() {
        let entry = match result {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        if entry.file_type().is_some_and(|kind| kind.is_dir()) {
            let name = entry.file_name().to_string_lossy();
            if options.ignore_dirs.contains(name.as_ref()) {
                ignored_paths.insert(name.into_owned());
            }
            continue;
        }

        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }

        let abs_path = Utf8PathBuf::from_path_buf(entry.path().to_path_buf())
            .map_err(|_| anyhow::anyhow!("path was not valid UTF-8"))?;
        let rel_path = abs_path
            .strip_prefix(&repo_root)
            .with_context(|| format!("failed to strip prefix {}", abs_path))?
            .to_path_buf();
        let language = Language::from_extension(rel_path.as_str());
        if matches!(language, Language::Unknown) {
            continue;
        }

        let content = fs::read_to_string(abs_path.as_std_path())
            .with_context(|| format!("failed to read {}", abs_path))?;
        let file = SourceFile {
            file_id: format!("file:{}", rel_path),
            abs_path: abs_path.clone(),
            rel_path: rel_path.clone(),
            module_path: module_path_from_rel(&rel_path),
            language,
            digest: sha256(&content),
            content,
        };
        files.push(file);
    }

    files.sort_by(|left, right| left.rel_path.cmp(&right.rel_path));

    Ok(RepositorySnapshot {
        repo_root,
        repo_name,
        files,
        ignored_paths,
    })
}

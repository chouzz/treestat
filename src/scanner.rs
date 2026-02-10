use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::Cli;
use crate::model::{DirData, ScanResult};

pub fn load_gitignore_patterns(root: &Path) -> Vec<String> {
    let mut out = vec![];
    let path = root.join(".gitignore");
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        out.push(trimmed.to_string());
    }
    out
}

pub fn scan_tree(
    root: &Path,
    extensions: &HashSet<String>,
    cli: &Cli,
    gitignore: &[String],
) -> Result<ScanResult, String> {
    let default_skip = [
        ".git",
        "target",
        "build",
        "out",
        "node_modules",
        "third_party",
        "dist",
    ];

    let mut dirs = HashMap::new();
    dirs.insert(
        root.to_path_buf(),
        DirData {
            name: root
                .file_name()
                .unwrap_or_else(|| OsStr::new("."))
                .to_string_lossy()
                .to_string(),
            ..DirData::default()
        },
    );

    let mut total_files = 0usize;
    let mut dirs_with_files = HashSet::new();
    let mut visited = HashSet::new();

    struct WalkCtx<'a> {
        root: &'a Path,
        extensions: &'a HashSet<String>,
        cli: &'a Cli,
        gitignore: &'a [String],
        default_skip: &'a [&'a str],
        dirs: &'a mut HashMap<PathBuf, DirData>,
        total_files: &'a mut usize,
        dirs_with_files: &'a mut HashSet<PathBuf>,
        visited: &'a mut HashSet<PathBuf>,
    }

    fn walk(current: &Path, depth: usize, ctx: &mut WalkCtx<'_>) -> Result<(), String> {
        let canon = current
            .canonicalize()
            .unwrap_or_else(|_| current.to_path_buf());
        if ctx.cli.follow_symlinks && !ctx.visited.insert(canon) {
            return Ok(());
        }

        let entries = fs::read_dir(current)
            .map_err(|e| format!("failed to read {}: {e}", current.display()))?;
        for entry in entries {
            let entry = match entry {
                Ok(v) => v,
                Err(_) => continue,
            };
            let path = entry.path();
            let rel = path.strip_prefix(ctx.root).unwrap_or(&path);
            if should_exclude(rel, ctx.cli, ctx.gitignore, ctx.default_skip) {
                continue;
            }

            let metadata = if ctx.cli.follow_symlinks {
                fs::metadata(&path)
            } else {
                fs::symlink_metadata(&path)
            };
            let Ok(meta) = metadata else { continue };
            let ft = meta.file_type();

            if ft.is_symlink() && !ctx.cli.follow_symlinks {
                continue;
            }

            if ft.is_dir() {
                if ctx.cli.max_depth.is_some_and(|max| depth + 1 > max) {
                    continue;
                }

                ctx.dirs.entry(path.clone()).or_insert_with(|| DirData {
                    name: path
                        .file_name()
                        .unwrap_or_else(|| OsStr::new("."))
                        .to_string_lossy()
                        .to_string(),
                    ..DirData::default()
                });
                if let Some(parent) = path.parent() {
                    ctx.dirs
                        .entry(parent.to_path_buf())
                        .or_default()
                        .children
                        .insert(path.clone());
                }

                walk(&path, depth + 1, ctx)?;
                continue;
            }

            if ft.is_file() {
                let ext = path
                    .extension()
                    .and_then(|v| v.to_str())
                    .map(|v| v.to_ascii_lowercase());
                if let Some(ext) = ext
                    && ctx.extensions.contains(&ext)
                {
                    *ctx.total_files += 1;
                    let parent = path.parent().unwrap_or(ctx.root).to_path_buf();
                    ctx.dirs.entry(parent.clone()).or_default().direct_files += 1;
                    if parent != ctx.root {
                        ctx.dirs_with_files.insert(parent);
                    }
                }
            }
        }
        Ok(())
    }

    let mut ctx = WalkCtx {
        root,
        extensions,
        cli,
        gitignore,
        default_skip: &default_skip,
        dirs: &mut dirs,
        total_files: &mut total_files,
        dirs_with_files: &mut dirs_with_files,
        visited: &mut visited,
    };
    walk(root, 0, &mut ctx)?;

    Ok(ScanResult {
        root: root.to_path_buf(),
        dirs,
        total_files,
        dirs_with_files: dirs_with_files.len(),
    })
}

fn should_exclude(rel: &Path, cli: &Cli, gitignore: &[String], default_skip: &[&str]) -> bool {
    let rel_str = rel.to_string_lossy();
    let comps = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>();

    if !cli.hidden && comps.iter().any(|c| c.starts_with('.')) {
        return true;
    }
    if comps.iter().any(|c| default_skip.contains(&c.as_ref())) {
        return true;
    }
    if cli.exclude.iter().any(|pat| simple_match(&rel_str, pat)) {
        return true;
    }
    if gitignore.iter().any(|pat| simple_match(&rel_str, pat)) {
        return true;
    }

    false
}

fn simple_match(path: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("/*") {
        return path.starts_with(prefix);
    }
    if let Some(suffix) = pattern.strip_prefix("*.") {
        return path.ends_with(suffix);
    }
    path.contains(pattern.trim_matches('/'))
}

pub fn compute_tree_counts(
    root: &Path,
    dirs: &HashMap<PathBuf, DirData>,
) -> HashMap<PathBuf, usize> {
    fn dfs(
        path: &Path,
        dirs: &HashMap<PathBuf, DirData>,
        memo: &mut HashMap<PathBuf, usize>,
    ) -> usize {
        if let Some(v) = memo.get(path) {
            return *v;
        }
        let mut sum = dirs.get(path).map_or(0, |d| d.direct_files);
        if let Some(dir) = dirs.get(path) {
            for child in &dir.children {
                sum += dfs(child, dirs, memo);
            }
        }
        memo.insert(path.to_path_buf(), sum);
        sum
    }

    let mut memo = HashMap::new();
    dfs(root, dirs, &mut memo);
    memo
}

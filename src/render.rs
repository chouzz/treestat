use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::cli::{Cli, CountMode};
use crate::lang::display_langs;
use crate::model::ScanResult;

fn display_count(
    path: &Path,
    scan: &ScanResult,
    tree_counts: &HashMap<PathBuf, usize>,
    mode: CountMode,
) -> usize {
    match mode {
        CountMode::Direct => scan.dirs.get(path).map_or(0, |d| d.direct_files),
        CountMode::Tree => *tree_counts.get(path).unwrap_or(&0),
    }
}

fn should_show(
    path: &Path,
    scan: &ScanResult,
    tree_counts: &HashMap<PathBuf, usize>,
    cli: &Cli,
) -> bool {
    if path == scan.root {
        return true;
    }
    if cli.show_empty {
        return true;
    }
    let subtree = *tree_counts.get(path).unwrap_or(&0);
    if subtree == 0 {
        return false;
    }

    let current = display_count(path, scan, tree_counts, cli.count_mode);
    if current >= cli.min_count {
        return true;
    }

    scan.dirs.get(path).is_some_and(|d| {
        d.children
            .iter()
            .any(|c| should_show(c, scan, tree_counts, cli))
    })
}

pub fn render_text(
    scan: &ScanResult,
    tree_counts: &HashMap<PathBuf, usize>,
    extensions: &HashSet<String>,
    langs: &[String],
    cli: &Cli,
) -> String {
    let mut out = String::new();
    let title = format!("{} file statistics (Tree View):", display_langs(langs));

    out.push_str(&title);
    out.push('\n');
    out.push_str("============================================================\n");
    let root_name = scan
        .root
        .file_name()
        .unwrap_or_else(|| OsStr::new("."))
        .to_string_lossy();
    out.push_str(&format!(
        "{root_name}/ ({} files)\n",
        display_count(&scan.root, scan, tree_counts, cli.count_mode)
    ));

    let children = scan
        .dirs
        .get(&scan.root)
        .map(|d| d.children.iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let visible = children
        .into_iter()
        .filter(|c| should_show(c, scan, tree_counts, cli))
        .collect::<Vec<_>>();
    for (idx, child) in visible.iter().enumerate() {
        render_text_node(
            &mut out,
            child,
            scan,
            tree_counts,
            cli,
            "",
            idx + 1 == visible.len(),
        );
    }

    out.push_str("============================================================\n");
    out.push_str(&format!("Total matching files: {}\n", scan.total_files));
    out.push_str(&format!(
        "Directories containing files: {}\n",
        scan.dirs_with_files
    ));
    let mut exts = extensions.iter().cloned().collect::<Vec<_>>();
    exts.sort();
    out.push_str(&format!("Extensions: {}\n", exts.join(",")));
    out
}

fn render_text_node(
    out: &mut String,
    path: &Path,
    scan: &ScanResult,
    tree_counts: &HashMap<PathBuf, usize>,
    cli: &Cli,
    prefix: &str,
    is_last: bool,
) {
    let Some(dir) = scan.dirs.get(path) else {
        return;
    };
    let connector = if is_last { "└── " } else { "├── " };
    out.push_str(&format!(
        "{prefix}{connector}{}/ ({} files)\n",
        dir.name,
        display_count(path, scan, tree_counts, cli.count_mode)
    ));

    let next_prefix = if is_last {
        format!("{prefix}    ")
    } else {
        format!("{prefix}│   ")
    };
    let children = dir
        .children
        .iter()
        .filter(|c| should_show(c, scan, tree_counts, cli))
        .cloned()
        .collect::<Vec<_>>();
    for (idx, child) in children.iter().enumerate() {
        render_text_node(
            out,
            child,
            scan,
            tree_counts,
            cli,
            &next_prefix,
            idx + 1 == children.len(),
        );
    }
}

pub fn render_json(
    scan: &ScanResult,
    tree_counts: &HashMap<PathBuf, usize>,
    extensions: &HashSet<String>,
    langs: &[String],
    cli: &Cli,
    pretty: bool,
) -> String {
    fn node(
        path: &Path,
        scan: &ScanResult,
        tree_counts: &HashMap<PathBuf, usize>,
        cli: &Cli,
        pretty: bool,
        indent: usize,
    ) -> String {
        let d = scan.dirs.get(path).expect("node exists");
        let children = d
            .children
            .iter()
            .filter(|c| should_show(c, scan, tree_counts, cli))
            .cloned()
            .collect::<Vec<_>>();

        let mut child_json = vec![];
        for child in &children {
            child_json.push(node(child, scan, tree_counts, cli, pretty, indent + 2));
        }
        let pad = if pretty {
            " ".repeat(indent)
        } else {
            String::new()
        };
        let sep = if pretty { "\n" } else { "" };
        let inner = if pretty {
            " ".repeat(indent + 2)
        } else {
            String::new()
        };
        let children_str = if child_json.is_empty() {
            "[]".to_string()
        } else if pretty {
            format!("[\n{}\n{}]", child_json.join(",\n"), inner)
        } else {
            format!("[{}]", child_json.join(","))
        };

        format!(
            "{pad}{{{sep}{inner}\"name\":\"{}\",{sep}{inner}\"path\":\"{}\",{sep}{inner}\"files\":{},\n{inner}\"children\":{}{sep}{pad}}}",
            escape_json(&d.name),
            escape_json(&path.to_string_lossy()),
            display_count(path, scan, tree_counts, cli.count_mode),
            children_str
        )
    }

    let mut exts = extensions.iter().cloned().collect::<Vec<_>>();
    exts.sort();
    let ext_json = if pretty {
        exts.iter()
            .map(|e| format!("\"{}\"", escape_json(e)))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        exts.iter()
            .map(|e| format!("\"{}\"", escape_json(e)))
            .collect::<Vec<_>>()
            .join(",")
    };
    let lang_str = display_langs(langs);

    if pretty {
        format!(
            "{{\n  \"root\": \"{}\",\n  \"path\": \"{}\",\n  \"count_mode\": \"{}\",\n  \"lang\": \"{}\",\n  \"extensions\": [{}],\n  \"max_depth\": {},\n  \"total_files\": {},\n  \"dirs_with_files\": {},\n  \"tree\": {}\n}}",
            escape_json(
                &scan
                    .root
                    .file_name()
                    .unwrap_or_else(|| OsStr::new("."))
                    .to_string_lossy()
            ),
            escape_json(&scan.root.to_string_lossy()),
            match cli.count_mode {
                CountMode::Direct => "direct",
                CountMode::Tree => "tree",
            },
            lang_str,
            ext_json,
            cli.max_depth
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_string()),
            scan.total_files,
            scan.dirs_with_files,
            node(&scan.root, scan, tree_counts, cli, true, 2)
        )
    } else {
        format!(
            "{{\"root\":\"{}\",\"path\":\"{}\",\"count_mode\":\"{}\",\"lang\":\"{}\",\"extensions\":[{}],\"max_depth\":{},\"total_files\":{},\"dirs_with_files\":{},\"tree\":{}}}",
            escape_json(
                &scan
                    .root
                    .file_name()
                    .unwrap_or_else(|| OsStr::new("."))
                    .to_string_lossy()
            ),
            escape_json(&scan.root.to_string_lossy()),
            match cli.count_mode {
                CountMode::Direct => "direct",
                CountMode::Tree => "tree",
            },
            lang_str,
            ext_json,
            cli.max_depth
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_string()),
            scan.total_files,
            scan.dirs_with_files,
            node(&scan.root, scan, tree_counts, cli, false, 0)
        )
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

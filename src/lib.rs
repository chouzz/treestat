pub mod cli;
pub mod lang;
pub mod model;
pub mod render;
pub mod scanner;

use std::path::Path;

use cli::{Cli, Format};
use lang::build_extensions;
use render::{render_json, render_text};
use scanner::{compute_tree_counts, scan_tree};

pub fn run(cli: Cli) -> Result<String, String> {
    let root = cli
        .path
        .canonicalize()
        .map_err(|e| format!("failed to resolve root path {:?}: {e}", cli.path))?;
    if !root.is_dir() {
        return Err(format!("path is not a directory: {}", root.display()));
    }

    let extensions = build_extensions(cli.lang, &cli.ext, cli.headers);
    if extensions.is_empty() {
        return Err("no extensions selected; provide --lang or --ext".to_string());
    }

    let gitignore_patterns = if cli.no_gitignore {
        vec![]
    } else {
        scanner::load_gitignore_patterns(Path::new(&root))
    };

    let scan = scan_tree(Path::new(&root), &extensions, &cli, &gitignore_patterns)?;
    let tree_counts = compute_tree_counts(Path::new(&scan.root), &scan.dirs);

    let output = match cli.format {
        Format::Text => render_text(&scan, &tree_counts, &extensions, cli.lang, &cli),
        Format::Json => render_json(
            &scan,
            &tree_counts,
            &extensions,
            cli.lang,
            &cli,
            cli.json_pretty,
        ),
    };

    Ok(output)
}

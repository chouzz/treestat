# treestat

`treestat` is a Rust CLI tool that prints a `tree`-style directory view and shows only one number per directory: matching files `(N files)`.

- Filter by language presets (`--lang`) or custom extensions (`--ext`).
- Choose counting mode: direct files only (`direct`) or full subtree (`tree`).
- Output as human-readable text or stable JSON for CI and scripts.

## Installation

```bash
cargo install treestat
```

## Quick Start

```bash
treestat . --lang c,cpp
```

```bash
treestat . --lang rust --count-mode direct --max-depth 3
```

```bash
treestat . --ext c,cc,cpp,h,hpp --headers exclude
```

```bash
treestat . --lang python --format json --json-pretty
```

## CLI

```text
treestat [PATH] [OPTIONS]
```

- `PATH`: target directory (default: `.`)
- `--lang <LANG[,LANG...]>`: language preset(s) from embedded Linguist data (repeatable, aliases supported)
- `--ext <LIST>`: custom extensions (comma-separated, supports `rs` or `.rs`)
- `--headers <include|exclude|only>`: header-file policy (default: `include`)
- `--count-mode <direct|tree>`: counting mode (default: `tree`)
- `--max-depth <N>`: maximum directory depth (root=0)
- `--min-count <N>`: hide directories below this count
- `--show-empty`: include `0 files` directories
- `--follow-symlinks`: follow symlinks (default: disabled)
- `--exclude <PATTERN>`: exclude path pattern (repeatable)
- `--no-gitignore`: disable `.gitignore`-based filtering
- `--hidden`: include hidden files/directories
- `--format <text|json>`: output format (default: `text`)
- `--json-pretty`: pretty-print JSON

## Default Behavior

- `.gitignore` patterns are enabled by default.
- Hidden entries are excluded by default unless `--hidden` is set.
- Common build/output directories are excluded by default:
  `.git`, `target`, `build`, `out`, `node_modules`, `third_party`, `dist`.
- `dirs_with_files` does not include the root directory.

## JSON Output Example

```json
{
  "root": "project",
  "path": "/abs/path/project",
  "count_mode": "tree",
  "lang": "c,c++",
  "extensions": ["c", "cc", "cpp", "cxx", "h", "hh", "hpp", "hxx"],
  "max_depth": 3,
  "total_files": 333,
  "dirs_with_files": 6,
  "tree": {
    "name": "project",
    "path": "/abs/path/project",
    "files": 333,
    "children": []
  }
}
```

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

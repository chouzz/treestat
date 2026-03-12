# treestat

[![Crates.io](https://img.shields.io/crates/v/treestat?label=crates.io)](https://crates.io/crates/treestat)
[![PyPI](https://img.shields.io/pypi/v/treestat-cli.svg?label=PyPI)](https://pypi.org/project/treestat-cli/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Command-line tool that shows **source file counts per directory and language** in a tree view.

- **Flexible filtering**: language presets via `--lang` or custom extensions via `--ext`.
- **Configurable counting**: only direct files in each directory (`direct`) or full subtree aggregation (`tree`).
- **Script‑friendly output**: human‑readable text or stable JSON for CI, dashboards, and scripts.

---

## ⚙️ Installation

The recommended way to install `treestat` is via **uv** so that it is managed as a Python tool:

```bash
uv tool install treestat-cli
```

Or install from PyPI:

```bash
pip install treestat-cli
```

You can also install the native Rust binary from crates.io:

```bash
cargo install treestat
```

---

## 🚀 Usage

Count C/C++ files up to depth 3 under the current directory:

```bash
treestat . --lang c,cpp --max-depth 3
```

Example (text) output:

```text
all file statistics (Tree View):
============================================================
llvm-project/ (52970 files)
├── bolt/ (241 files)
├── clang/ (18264 files)
├── clang-tools-extra/ (3003 files)
├── compiler-rt/ (3417 files)
├── cross-project-tests/ (228 files)
├── flang/ (665 files)
├── libc/ (1959 files)
├── libclc/ (248 files)
├── libcxx/ (9179 files)
├── libcxxabi/ (105 files)
├── libunwind/ (43 files)
├── lld/ (219 files)
├── lldb/ (4765 files)
├── llvm/ (6866 files)
├── llvm-libgcc/ (2 files)
├── mlir/ (1873 files)
├── openmp/ (744 files)
├── polly/ (893 files)
├── pstl/ (97 files)
├── third-party/ (154 files)
└── utils/ (5 files)
============================================================
Total matching files: 52970
Directories containing files: 6516
Extensions: c,js,cpp,py
```

To get JSON for automation:

```bash
treestat . --lang rust --format json --json-pretty
```

---

## 📚 CLI reference

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

---

## ⚖️ Default behavior

- `.gitignore` patterns are **enabled by default**.
- Hidden entries are **excluded by default** unless `--hidden` is set.
- Common build/output directories are excluded by default:
  `.git`, `target`, `build`, `out`, `node_modules`, `third_party`, `dist`.
- `dirs_with_files` does **not** include the root directory.

---

## 🧪 Development

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

---

## 📜 License

MIT License. See `LICENSE` for details.

---

## 🤝 Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.

---

## 📞 Support

- GitHub Issues: [Report a bug](https://github.com/chouzz/treestat/issues)
- GitHub README: [View the latest docs](https://github.com/chouzz/treestat#readme)

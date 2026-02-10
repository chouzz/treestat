use std::env;
use std::path::PathBuf;

use crate::lang::Lang;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountMode {
    Direct,
    Tree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderMode {
    Include,
    Exclude,
    Only,
}

#[derive(Debug)]
pub struct Cli {
    pub path: PathBuf,
    pub lang: Option<Lang>,
    pub ext: Vec<String>,
    pub headers: HeaderMode,
    pub count_mode: CountMode,
    pub max_depth: Option<usize>,
    pub min_count: usize,
    pub show_empty: bool,
    pub follow_symlinks: bool,
    pub exclude: Vec<String>,
    pub no_gitignore: bool,
    pub hidden: bool,
    pub format: Format,
    pub json_pretty: bool,
}

impl Cli {
    pub fn parse_env() -> Result<Self, String> {
        Self::parse(env::args().skip(1).collect())
    }

    pub fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut path: Option<PathBuf> = None;
        let mut lang = None;
        let mut ext = vec![];
        let mut headers = HeaderMode::Include;
        let mut count_mode = CountMode::Tree;
        let mut max_depth = None;
        let mut min_count = 0usize;
        let mut show_empty = false;
        let mut follow_symlinks = false;
        let mut exclude = vec![];
        let mut no_gitignore = false;
        let mut hidden = false;
        let mut format = Format::Text;
        let mut json_pretty = false;

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            match arg.as_str() {
                "-h" | "--help" => return Err("--help".to_string()),
                "-V" | "--version" => return Err("--version".to_string()),
                "--lang" => {
                    i += 1;
                    let v = args.get(i).ok_or("--lang requires a value")?;
                    lang = Some(parse_lang(v)?);
                }
                "--ext" => {
                    i += 1;
                    let v = args.get(i).ok_or("--ext requires a value")?;
                    for item in v.split(',') {
                        if let Some(n) = crate::lang::normalize_ext(item) {
                            ext.push(n);
                        }
                    }
                }
                "--headers" => {
                    i += 1;
                    headers = parse_headers(args.get(i).ok_or("--headers requires a value")?)?;
                }
                "--count-mode" => {
                    i += 1;
                    count_mode =
                        parse_count_mode(args.get(i).ok_or("--count-mode requires a value")?)?;
                }
                "--max-depth" => {
                    i += 1;
                    max_depth = Some(parse_usize(
                        args.get(i).ok_or("--max-depth requires a value")?,
                        "max-depth",
                    )?);
                }
                "--min-count" => {
                    i += 1;
                    min_count = parse_usize(
                        args.get(i).ok_or("--min-count requires a value")?,
                        "min-count",
                    )?;
                }
                "--show-empty" => show_empty = true,
                "--follow-symlinks" => follow_symlinks = true,
                "--exclude" => {
                    i += 1;
                    exclude.push(args.get(i).ok_or("--exclude requires a value")?.to_string());
                }
                "--no-gitignore" => no_gitignore = true,
                "--hidden" => hidden = true,
                "--format" => {
                    i += 1;
                    format = parse_format(args.get(i).ok_or("--format requires a value")?)?;
                }
                "--json-pretty" => json_pretty = true,
                s if s.starts_with('-') => return Err(format!("unknown option: {s}")),
                other => {
                    if path.is_some() {
                        return Err(format!("unexpected positional argument: {other}"));
                    }
                    path = Some(PathBuf::from(other));
                }
            }
            i += 1;
        }

        Ok(Self {
            path: path.unwrap_or_else(|| PathBuf::from(".")),
            lang,
            ext,
            headers,
            count_mode,
            max_depth,
            min_count,
            show_empty,
            follow_symlinks,
            exclude,
            no_gitignore,
            hidden,
            format,
            json_pretty,
        })
    }
}

fn parse_usize(v: &str, field: &str) -> Result<usize, String> {
    v.parse::<usize>()
        .map_err(|_| format!("invalid {field}: {v}"))
}

fn parse_lang(v: &str) -> Result<Lang, String> {
    match v.to_ascii_lowercase().as_str() {
        "c" => Ok(Lang::C),
        "cpp" => Ok(Lang::Cpp),
        "python" => Ok(Lang::Python),
        "rust" => Ok(Lang::Rust),
        "go" => Ok(Lang::Go),
        "java" => Ok(Lang::Java),
        "js" => Ok(Lang::Js),
        "ts" => Ok(Lang::Ts),
        "all" => Ok(Lang::All),
        _ => Err(format!("invalid --lang value: {v}")),
    }
}

fn parse_headers(v: &str) -> Result<HeaderMode, String> {
    match v.to_ascii_lowercase().as_str() {
        "include" => Ok(HeaderMode::Include),
        "exclude" => Ok(HeaderMode::Exclude),
        "only" => Ok(HeaderMode::Only),
        _ => Err(format!("invalid --headers value: {v}")),
    }
}

fn parse_count_mode(v: &str) -> Result<CountMode, String> {
    match v.to_ascii_lowercase().as_str() {
        "direct" => Ok(CountMode::Direct),
        "tree" => Ok(CountMode::Tree),
        _ => Err(format!("invalid --count-mode value: {v}")),
    }
}

fn parse_format(v: &str) -> Result<Format, String> {
    match v.to_ascii_lowercase().as_str() {
        "text" => Ok(Format::Text),
        "json" => Ok(Format::Json),
        _ => Err(format!("invalid --format value: {v}")),
    }
}

pub fn print_help() {
    println!(
        "treestat [PATH] [OPTIONS]\n\nOptions:\n  --lang <c|cpp|python|rust|go|java|js|ts|all>\n  --ext <a,b,c>\n  --headers <include|exclude|only>\n  --count-mode <direct|tree>\n  --max-depth <N>\n  --min-count <N>\n  --show-empty\n  --follow-symlinks\n  --exclude <PATTERN> (repeatable)\n  --no-gitignore\n  --hidden\n  --format <text|json>\n  --json-pretty\n  -h, --help\n  -V, --version"
    );
}

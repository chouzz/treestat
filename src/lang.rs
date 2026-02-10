use std::collections::HashSet;

use crate::cli::HeaderMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    C,
    Cpp,
    Python,
    Rust,
    Go,
    Java,
    Js,
    Ts,
    All,
}

pub fn build_extensions(
    lang: Option<Lang>,
    ext: &[String],
    headers: HeaderMode,
) -> HashSet<String> {
    let mut selected = HashSet::new();

    for e in ext {
        if let Some(n) = normalize_ext(e) {
            selected.insert(n);
        }
    }

    let active_lang = lang.unwrap_or(Lang::All);
    for e in lang_extensions(active_lang) {
        selected.insert((*e).to_string());
    }

    if matches!(active_lang, Lang::C | Lang::Cpp | Lang::All) {
        apply_header_mode(&mut selected, headers);
    }

    selected
}

pub fn normalize_ext(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.trim_start_matches('.').to_ascii_lowercase())
}

pub fn apply_header_mode(exts: &mut HashSet<String>, headers: HeaderMode) {
    let header_exts: HashSet<String> = ["h", "hh", "hpp", "hxx", "h++"]
        .into_iter()
        .map(str::to_string)
        .collect();
    match headers {
        HeaderMode::Include => exts.extend(header_exts),
        HeaderMode::Exclude => exts.retain(|e| !header_exts.contains(e)),
        HeaderMode::Only => exts.retain(|e| header_exts.contains(e)),
    }
}

pub fn lang_extensions(lang: Lang) -> &'static [&'static str] {
    match lang {
        Lang::C => &["c"],
        Lang::Cpp => &["c", "cc", "cpp", "cxx"],
        Lang::Python => &["py", "pyi"],
        Lang::Rust => &["rs"],
        Lang::Go => &["go"],
        Lang::Java => &["java"],
        Lang::Js => &["js", "mjs", "cjs", "jsx"],
        Lang::Ts => &["ts", "mts", "cts", "tsx"],
        Lang::All => &[
            "c", "cc", "cpp", "cxx", "h", "hh", "hpp", "hxx", "h++", "py", "pyi", "rs", "go",
            "java", "js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx",
        ],
    }
}

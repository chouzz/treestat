use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::OnceLock;

use crate::cli::HeaderMode;

#[derive(Debug, Default)]
struct LinguistLanguage {
    aliases: Vec<String>,
    extensions: Vec<String>,
}

#[derive(Debug)]
struct LanguageRegistry {
    names: HashMap<String, String>,
    extensions: HashMap<String, HashSet<String>>,
}

static REGISTRY: OnceLock<LanguageRegistry> = OnceLock::new();

fn parse_embedded_linguist_yaml() -> HashMap<String, LinguistLanguage> {
    let mut result = HashMap::new();
    let mut current_lang: Option<String> = None;
    let mut current_field: Option<&str> = None;

    for line in include_str!("../data/linguist_languages.yml").lines() {
        let raw = line.trim();
        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }

        if !line.starts_with(' ') && raw.ends_with(':') {
            let lang = raw.trim_end_matches(':').to_string();
            result
                .entry(lang.clone())
                .or_insert_with(LinguistLanguage::default);
            current_lang = Some(lang);
            current_field = None;
            continue;
        }

        if raw.starts_with("aliases:") {
            current_field = Some("aliases");
            continue;
        }
        if raw.starts_with("extensions:") {
            current_field = Some("extensions");
            continue;
        }

        if raw.starts_with("- ") {
            let Some(lang) = current_lang.as_ref() else {
                continue;
            };
            let value = raw.trim_start_matches("- ").trim().to_string();
            let entry = result
                .entry(lang.clone())
                .or_insert_with(LinguistLanguage::default);
            match current_field {
                Some("aliases") => entry.aliases.push(value),
                Some("extensions") => entry.extensions.push(value),
                _ => {}
            }
        }
    }

    result
}

fn registry() -> &'static LanguageRegistry {
    REGISTRY.get_or_init(|| {
        let raw = parse_embedded_linguist_yaml();

        let mut names = HashMap::new();
        let mut extensions = HashMap::new();

        for (lang, spec) in raw {
            let canonical = lang.to_ascii_lowercase();
            names.insert(canonical.clone(), canonical.clone());
            for alias in spec.aliases {
                names.insert(alias.to_ascii_lowercase(), canonical.clone());
            }

            let ext_set = spec
                .extensions
                .into_iter()
                .filter_map(|e| normalize_ext(&e))
                .collect::<HashSet<_>>();
            extensions.insert(canonical, ext_set);
        }

        LanguageRegistry { names, extensions }
    })
}

pub fn canonical_language_name(raw: &str) -> Option<String> {
    registry().names.get(&raw.to_ascii_lowercase()).cloned()
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

pub fn build_extensions(
    langs: &[String],
    ext: &[String],
    headers: HeaderMode,
) -> Result<HashSet<String>, String> {
    let mut selected = HashSet::new();

    for e in ext {
        if let Some(n) = normalize_ext(e) {
            selected.insert(n);
        }
    }

    let requested_langs = if langs.is_empty() {
        vec!["all".to_string()]
    } else {
        langs.to_vec()
    };

    for raw in &requested_langs {
        if raw.eq_ignore_ascii_case("all") {
            for exts in registry().extensions.values() {
                selected.extend(exts.iter().cloned());
            }
            continue;
        }

        let canonical =
            canonical_language_name(raw).ok_or_else(|| format!("invalid --lang value: {raw}"))?;
        let exts = registry()
            .extensions
            .get(&canonical)
            .ok_or_else(|| format!("language has no extensions: {raw}"))?;
        selected.extend(exts.iter().cloned());
    }

    if requested_langs.iter().any(|l| {
        l.eq_ignore_ascii_case("all")
            || canonical_language_name(l).is_some_and(|name| name == "c" || name == "c++")
    }) {
        apply_header_mode(&mut selected, headers);
    }

    Ok(selected)
}

pub fn display_langs(langs: &[String]) -> String {
    if langs.is_empty() {
        return "all".to_string();
    }

    let mut normalized = BTreeSet::new();
    for lang in langs {
        let canonical = canonical_language_name(lang).unwrap_or_else(|| lang.to_ascii_lowercase());
        normalized.insert(canonical);
    }

    normalized.into_iter().collect::<Vec<_>>().join(",")
}

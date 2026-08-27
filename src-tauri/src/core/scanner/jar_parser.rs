use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

use crate::core::constants::{BOOK_PATH_MARKERS, MD_PATH_MARKERS, RESEARCH_PATH_MARKERS};
use crate::core::scanner::lang_parser::{iter_translatable_strings, load_lenient_json, parse_legacy_lang};
use crate::core::shield::{apply_smart_glue, is_technical_term, looks_like_source_language};

pub enum JarEntryType {
    LangJson,
    LangLegacy,
    BookJson,
    BookMd,
}

pub struct TranslatableJarEntry {
    pub internal_path: String,
    pub entry_type: JarEntryType,
    pub strings: HashMap<String, String>, // key or path -> original text
}

pub fn scan_jar_translatables(
    jar_path: &Path,
    translate_mods: bool,
    translate_books: bool,
    smart_glue: bool,
) -> Result<Vec<TranslatableJarEntry>, String> {
    let file = File::open(jar_path).map_err(|e| e.to_string())?;
    let mut zip = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut entries = Vec::new();

    for i in 0..zip.len() {
        let (name, is_target) = {
            let file = zip.by_index(i).map_err(|e| e.to_string())?;
            let name = file.name().to_string();
            let fl = name.to_lowercase();
            let is_book_json = fl.ends_with(".json")
                && fl.contains("/en_us/")
                && (BOOK_PATH_MARKERS.iter().any(|&m| fl.contains(m))
                    || RESEARCH_PATH_MARKERS.iter().any(|&m| fl.contains(m)));

            let is_book_md = (fl.ends_with(".md") || fl.ends_with(".txt"))
                && fl.contains("/en_us/")
                && MD_PATH_MARKERS.iter().any(|&m| fl.contains(m));

            let is_lang_json = (fl.ends_with("en_us.json") || fl.ends_with("en_us.lang")) && !is_book_json;
            (name, (is_lang_json && translate_mods) || ((is_book_json || is_book_md) && translate_books))
        };

        if !is_target {
            continue;
        }

        let fl = name.to_lowercase();
        let mut buffer = Vec::new();
        {
            let mut file = zip.by_index(i).map_err(|e| e.to_string())?;
            file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
        }

        let is_book_json = fl.ends_with(".json")
            && fl.contains("/en_us/")
            && (BOOK_PATH_MARKERS.iter().any(|&m| fl.contains(m))
                || RESEARCH_PATH_MARKERS.iter().any(|&m| fl.contains(m)));

        let is_book_md = (fl.ends_with(".md") || fl.ends_with(".txt"))
            && fl.contains("/en_us/")
            && MD_PATH_MARKERS.iter().any(|&m| fl.contains(m));

        if is_book_json && translate_books {
            if let Ok(json_val) = load_lenient_json(&buffer) {
                let translatables = iter_translatable_strings(&json_val, Vec::new());
                let mut map = HashMap::new();
                for (p, text) in translatables {
                    let trimmed = text.trim();
                    if !trimmed.is_empty()
                        && looks_like_source_language(trimmed)
                        && !is_technical_term(trimmed)
                    {
                        map.insert(p.join("/"), text);
                    }
                }
                if !map.is_empty() {
                    entries.push(TranslatableJarEntry {
                        internal_path: name,
                        entry_type: JarEntryType::BookJson,
                        strings: map,
                    });
                }
            }
        } else if is_book_md && translate_books {
            let mut text = String::from_utf8_lossy(&buffer).to_string();
            if smart_glue {
                text = apply_smart_glue(&text);
            }
            let mut map = HashMap::new();
            let mut in_yaml = false;

            for (idx, line) in text.lines().enumerate() {
                let stripped = line.trim();
                if stripped == "---" {
                    in_yaml = !in_yaml;
                    continue;
                }
                if in_yaml {
                    if stripped.to_lowercase().starts_with("title:") {
                        let title_part = stripped["title:".len()..].trim().trim_matches(|c| c == '\'' || c == '"');
                        if looks_like_source_language(title_part) {
                            map.insert(idx.to_string(), title_part.to_string());
                        }
                    }
                    continue;
                }
                if stripped.starts_with('<') || stripped.starts_with("![") {
                    continue;
                }
                if !stripped.is_empty()
                    && looks_like_source_language(stripped)
                    && !is_technical_term(stripped)
                {
                    map.insert(idx.to_string(), line.to_string());
                }
            }
            if !map.is_empty() {
                entries.push(TranslatableJarEntry {
                    internal_path: name,
                    entry_type: JarEntryType::BookMd,
                    strings: map,
                });
            }
        } else if translate_mods {
            if fl.ends_with(".json") {
                if let Ok(json_val) = load_lenient_json(&buffer) {
                    if let Some(obj) = json_val.as_object() {
                        let mut map = HashMap::new();
                        for (k, v) in obj {
                            if let Some(str_val) = v.as_str() {
                                let trimmed = str_val.trim();
                                if !trimmed.is_empty()
                                    && looks_like_source_language(trimmed)
                                    && !is_technical_term(trimmed)
                                {
                                    map.insert(k.clone(), str_val.to_string());
                                }
                            }
                        }
                        if !map.is_empty() {
                            entries.push(TranslatableJarEntry {
                                internal_path: name,
                                entry_type: JarEntryType::LangJson,
                                strings: map,
                            });
                        }
                    }
                }
            } else if fl.ends_with(".lang") {
                let text = String::from_utf8_lossy(&buffer);
                let parsed = parse_legacy_lang(&text);
                let mut map = HashMap::new();
                for (k, v) in parsed {
                    let trimmed = v.trim();
                    if !trimmed.is_empty()
                        && looks_like_source_language(trimmed)
                        && !is_technical_term(trimmed)
                    {
                        map.insert(k, v);
                    }
                }
                if !map.is_empty() {
                    entries.push(TranslatableJarEntry {
                        internal_path: name,
                        entry_type: JarEntryType::LangLegacy,
                        strings: map,
                    });
                }
            }
        }
    }

    Ok(entries)
}

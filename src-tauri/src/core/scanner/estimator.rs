use std::fs;
use std::path::Path;

use crate::core::scanner::discovery::{discover_jar_files, discover_loose_lang_files, discover_snbt_files};
use crate::core::scanner::jar_parser::scan_jar_translatables;
use crate::core::scanner::lang_parser::load_lenient_json;
use crate::core::scanner::snbt_parser::extract_snbt_strings;
use crate::core::shield::{already_translated, is_technical_term, looks_like_source_language};

pub fn estimate_total_strings(
    mc_dir: &Path,
    target_regex: &str,
    mode: &str,
    translate_mods: bool,
    translate_books: bool,
    translate_quests: bool,
    smart_glue: bool,
) -> usize {
    let mut total = 0;

    // 1. Estimate JARs
    if translate_mods || translate_books {
        let jars = discover_jar_files(mc_dir);
        for jar_path in jars {
            if let Ok(entries) = scan_jar_translatables(&jar_path, translate_mods, translate_books, smart_glue) {
                for entry in entries {
                    for (_k, text) in entry.strings {
                        if mode == "force" || !already_translated(&text, target_regex) {
                            total += 1;
                        }
                    }
                }
            }
        }
    }

    // 2. Estimate Loose Lang files
    if translate_mods {
        let loose = discover_loose_lang_files(mc_dir);
        for path in loose {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let fl = file_name.to_lowercase();
            if fl.ends_with(".lang") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let map = crate::core::scanner::lang_parser::parse_legacy_lang(&content);
                    for (_k, text) in map {
                        let trimmed = text.trim();
                        if !trimmed.is_empty()
                            && looks_like_source_language(trimmed)
                            && !is_technical_term(trimmed)
                        {
                            if mode == "force" || !already_translated(&text, target_regex) {
                                total += 1;
                            }
                        }
                    }
                }
            } else if fl.ends_with(".json") {
                if let Ok(content) = fs::read(&path) {
                    if let Ok(json_val) = load_lenient_json(&content) {
                        if let Some(obj) = json_val.as_object() {
                            for (_k, v) in obj {
                                if let Some(s) = v.as_str() {
                                    let trimmed = s.trim();
                                    if !trimmed.is_empty()
                                        && looks_like_source_language(trimmed)
                                        && !is_technical_term(trimmed)
                                    {
                                        if mode == "force" || !already_translated(s, target_regex) {
                                            total += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Estimate Quests (.snbt & BetterQuesting)
    if translate_quests {
        let snbt_files = discover_snbt_files(mc_dir);
        for path in snbt_files {
            if let Ok(content) = fs::read_to_string(&path) {
                let skip_regex = if mode == "append" {
                    Some(target_regex)
                } else {
                    None
                };
                let strings = extract_snbt_strings(&content, skip_regex);
                total += strings.len();
            }
        }

        let bq_files = crate::core::scanner::discovery::discover_better_questing_files(mc_dir);
        for bq_path in bq_files {
            let file_name = bq_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let fl = file_name.to_lowercase();
            if fl.ends_with(".lang") {
                if let Ok(content) = fs::read_to_string(&bq_path) {
                    let map = crate::core::scanner::lang_parser::parse_legacy_lang(&content);
                    for (_k, text) in map {
                        if mode == "force" || !already_translated(&text, target_regex) {
                            total += 1;
                        }
                    }
                }
            } else if fl.ends_with(".json") {
                if let Ok(content) = fs::read(&bq_path) {
                    if let Ok(json_val) = load_lenient_json(&content) {
                        let pairs = crate::core::scanner::bq_parser::extract_better_questing_strings(&json_val);
                        for (_k, text) in pairs {
                            if mode == "force" || !already_translated(&text, target_regex) {
                                total += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    total
}

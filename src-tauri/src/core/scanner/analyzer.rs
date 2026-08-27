use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::core::scanner::discovery::{discover_jar_files, discover_loose_lang_files, discover_snbt_files, get_mod_name};
use crate::core::scanner::jar_parser::scan_jar_translatables;
use crate::core::scanner::lang_parser::{load_lenient_json, parse_legacy_lang};
use crate::core::scanner::snbt_parser::extract_snbt_strings;
use crate::core::shield::already_translated;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRow {
    pub icon: String,
    pub name: String,
    pub kind: String,
    pub translated_count: usize,
    pub total_count: usize,
    pub percent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub rows: Vec<AnalysisRow>,
    pub total_translated: usize,
    pub total_strings: usize,
    pub overall_percent: u32,
}

pub fn analyze_modpack(
    mc_dir: &Path,
    target_regex: &str,
    translate_mods: bool,
    translate_books: bool,
    translate_quests: bool,
    smart_glue: bool,
) -> AnalysisResult {
    let mut rows = Vec::new();
    let mut total_translated = 0usize;
    let mut total_strings = 0usize;

    // 1. Scan JARs
    if translate_mods || translate_books {
        let jars = discover_jar_files(mc_dir);
        for jar_path in jars {
            let mod_name = get_mod_name(&jar_path);
            if let Ok(entries) = scan_jar_translatables(&jar_path, translate_mods, translate_books, smart_glue) {
                for entry in entries {
                    let total = entry.strings.len();
                    if total == 0 {
                        continue;
                    }
                    let tr_count = entry
                        .strings
                        .values()
                        .filter(|v| already_translated(v, target_regex))
                        .count();

                    let (icon, kind) = match entry.entry_type {
                        crate::core::scanner::jar_parser::JarEntryType::LangJson
                        | crate::core::scanner::jar_parser::JarEntryType::LangLegacy => {
                            ("📦", "Интерфейс")
                        }
                        crate::core::scanner::jar_parser::JarEntryType::BookJson => {
                            ("📖", "Книга JSON")
                        }
                        crate::core::scanner::jar_parser::JarEntryType::BookMd => {
                            ("📝", "Книга MD")
                        }
                    };

                    let percent = if total > 0 {
                        ((tr_count as f64 / total as f64) * 100.0).round() as u32
                    } else {
                        100
                    };

                    total_translated += tr_count;
                    total_strings += total;

                    rows.push(AnalysisRow {
                        icon: icon.to_string(),
                        name: mod_name.clone(),
                        kind: kind.to_string(),
                        translated_count: tr_count,
                        total_count: total,
                        percent,
                    });
                }
            }
        }
    }

    // 2. Scan Loose Lang files (resources/..., kubejs/assets/..., defaultconfigs/...)
    if translate_mods {
        let loose = discover_loose_lang_files(mc_dir);
        for path in loose {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let fl = file_name.to_lowercase();
            let mut map = std::collections::HashMap::new();

            if fl.ends_with(".lang") {
                if let Ok(content) = fs::read_to_string(&path) {
                    map = parse_legacy_lang(&content);
                }
            } else if fl.ends_with(".json") {
                if let Ok(content) = fs::read(&path) {
                    if let Ok(json_val) = load_lenient_json(&content) {
                        if let Some(obj) = json_val.as_object() {
                            for (k, v) in obj {
                                if let Some(s) = v.as_str() {
                                    map.insert(k.clone(), s.to_string());
                                }
                            }
                        }
                    }
                }
            }

            let total = map.len();
            if total > 0 {
                let tr_count = map.values().filter(|v| already_translated(v, target_regex)).count();
                let percent = if total > 0 {
                    ((tr_count as f64 / total as f64) * 100.0).round() as u32
                } else {
                    100
                };

                let rel_display = path.strip_prefix(mc_dir)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| file_name.to_string());

                total_translated += tr_count;
                total_strings += total;

                rows.push(AnalysisRow {
                    icon: "📄".to_string(),
                    name: rel_display,
                    kind: "Ресурсы сборки".to_string(),
                    translated_count: tr_count,
                    total_count: total,
                    percent,
                });
            }
        }
    }

    // 3. Scan Quests (.snbt and BetterQuesting)
    if translate_quests {
        let snbt_files = discover_snbt_files(mc_dir);
        for snbt_path in snbt_files {
            if let Ok(content) = fs::read_to_string(&snbt_path) {
                let strings = extract_snbt_strings(&content, None);
                let total = strings.len();
                if total == 0 {
                    continue;
                }
                let tr_count = strings
                    .iter()
                    .filter(|s| already_translated(s, target_regex))
                    .count();

                let percent = if total > 0 {
                    ((tr_count as f64 / total as f64) * 100.0).round() as u32
                } else {
                    100
                };

                let file_name = snbt_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("quests.snbt")
                    .to_string();

                total_translated += tr_count;
                total_strings += total;

                rows.push(AnalysisRow {
                    icon: "📜".to_string(),
                    name: file_name,
                    kind: "Квесты FTB".to_string(),
                    translated_count: tr_count,
                    total_count: total,
                    percent,
                });
            }
        }

        let bq_files = crate::core::scanner::discovery::discover_better_questing_files(mc_dir);
        for bq_path in bq_files {
            let file_name = bq_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let fl = file_name.to_lowercase();

            if fl.ends_with(".lang") {
                if let Ok(content) = fs::read_to_string(&bq_path) {
                    let map = parse_legacy_lang(&content);
                    let total = map.len();
                    if total > 0 {
                        let tr_count = map.values().filter(|v| already_translated(v, target_regex)).count();
                        let percent = ((tr_count as f64 / total as f64) * 100.0).round() as u32;

                        let display_name = bq_path.strip_prefix(mc_dir)
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| file_name.to_string());

                        total_translated += tr_count;
                        total_strings += total;

                        rows.push(AnalysisRow {
                            icon: "📜".to_string(),
                            name: format!("BetterQuesting ({})", display_name),
                            kind: "Квесты BQ".to_string(),
                            translated_count: tr_count,
                            total_count: total,
                            percent,
                        });
                    }
                }
            } else if fl.ends_with(".json") {
                if let Ok(content) = fs::read(&bq_path) {
                    if let Ok(json_val) = load_lenient_json(&content) {
                        let pairs = crate::core::scanner::bq_parser::extract_better_questing_strings(&json_val);
                        let total = pairs.len();
                        if total > 0 {
                            let tr_count = pairs.iter().filter(|(_, v)| already_translated(v, target_regex)).count();
                            let percent = ((tr_count as f64 / total as f64) * 100.0).round() as u32;

                            let display_name = bq_path.strip_prefix(mc_dir)
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|_| file_name.to_string());

                            total_translated += tr_count;
                            total_strings += total;

                            rows.push(AnalysisRow {
                                icon: "📜".to_string(),
                                name: format!("BetterQuesting ({})", display_name),
                                kind: "Квесты BQ".to_string(),
                                translated_count: tr_count,
                                total_count: total,
                                percent,
                            });
                        }
                    }
                }
            }
        }
    }

    let overall_percent = if total_strings > 0 {
        ((total_translated as f64 / total_strings as f64) * 100.0).round() as u32
    } else {
        0
    };

    AnalysisResult {
        rows,
        total_translated,
        total_strings,
        overall_percent,
    }
}

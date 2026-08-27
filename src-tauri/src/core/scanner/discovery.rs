use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::core::constants::LOOSE_JSON_SEARCH_DIRS;

pub fn get_mod_name(jar_path: &Path) -> String {
    let file_name = jar_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Mod");
    
    let base = file_name
        .split("-0").next().unwrap_or(file_name)
        .split("-1").next().unwrap_or(file_name);

    base.replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn discover_jar_files(mc_dir: &Path) -> Vec<PathBuf> {
    let mods_dir = mc_dir.join("mods");
    if !mods_dir.is_dir() {
        return Vec::new();
    }

    let mut jars = Vec::new();
    if let Ok(entries) = fs::read_dir(&mods_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case("jar") {
                        jars.push(path);
                    }
                }
            }
        }
    }
    jars.sort();
    jars
}

pub fn discover_loose_lang_files(mc_dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for rel in LOOSE_JSON_SEARCH_DIRS {
        let base = if rel.is_empty() {
            mc_dir.to_path_buf()
        } else {
            mc_dir.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR))
        };
        if !base.is_dir() {
            continue;
        }
        let max_depth = if rel.is_empty() { 1 } else { 10 };
        for entry in WalkDir::new(&base).max_depth(max_depth).into_iter().flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let name_lower = name.to_lowercase();
                    let path_str = path.to_string_lossy().to_lowercase();
                    // Don't include betterquesting files here as they are handled in discover_better_questing_files
                    if path_str.contains("betterquesting") {
                        continue;
                    }
                    if name_lower == "en_us.json" 
                        || name_lower == "en_us.lang" 
                        || (name_lower.ends_with(".lang") && name_lower.contains("en_us"))
                        || (rel.is_empty() && name.starts_with("GregTech_") && name_lower.contains("en_us"))
                    {
                        found.push(path.to_path_buf());
                    }
                }
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

pub fn discover_snbt_files(mc_dir: &Path) -> Vec<PathBuf> {
    let quests_dir = mc_dir.join("config").join("ftbquests").join("quests");
    if !quests_dir.is_dir() {
        return Vec::new();
    }

    let ignore_langs: HashSet<&str> = [
        "ru_ru.snbt", "zh_cn.snbt", "es_es.snbt", "de_de.snbt",
        "fr_fr.snbt", "pt_br.snbt", "ko_kr.snbt", "ja_jp.snbt",
    ].iter().cloned().collect();

    let mut result = Vec::new();
    for entry in WalkDir::new(&quests_dir).into_iter().flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("snbt") {
                    let file_name_lower = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    let is_lang_dir = path
                        .parent()
                        .map(|p| p.to_string_lossy().to_lowercase().contains("lang"))
                        .unwrap_or(false);

                    if is_lang_dir && file_name_lower != "en_us.snbt" {
                        continue;
                    }

                    if ignore_langs.contains(file_name_lower.as_str()) {
                        continue;
                    }

                    result.push(path.to_path_buf());
                }
            }
        }
    }
    result.sort();
    result
}

pub fn discover_better_questing_files(mc_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    // 1. BetterQuesting config directory (both single-file DefaultQuests.json and modular JSONs)
    let bq_dir = mc_dir.join("config").join("betterquesting");
    if bq_dir.is_dir() {
        for entry in WalkDir::new(&bq_dir).into_iter().flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_low = ext.to_lowercase();
                    if ext_low == "json" || ext_low == "lang" {
                        let name_low = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
                        if name_low.ends_with(".lang") && !name_low.contains("en_us") && name_low != "template.lang" {
                            continue;
                        }
                        files.push(path.to_path_buf());
                    }
                }
            }
        }
    }

    // 2. TXLoader BetterQuesting language files (GTNH, 1.7.10, etc.)
    let tx_dirs = [
        mc_dir.join("config").join("txloader").join("load").join("betterquesting"),
        mc_dir.join("config").join("txloader").join("forceload").join("betterquesting"),
    ];
    for tx_dir in &tx_dirs {
        if tx_dir.is_dir() {
            for entry in WalkDir::new(tx_dir).into_iter().flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let name_low = name.to_lowercase();
                        if name_low == "template.lang" || name_low == "en_us.lang" {
                            files.push(path.to_path_buf());
                        }
                    }
                }
            }
        }
    }

    // 3. Triumph
    let triumph_dir = mc_dir.join("config").join("triumph");
    if triumph_dir.is_dir() {
        for entry in WalkDir::new(&triumph_dir).into_iter().flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "json" || e == "lang") {
                files.push(path.to_path_buf());
            }
        }
    }

    // 4. HQM (Hardcore Questing Mode)
    let hqm_dir = mc_dir.join("config").join("hqm");
    if hqm_dir.is_dir() {
        for entry in WalkDir::new(&hqm_dir).into_iter().flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "json" || e == "lang") {
                files.push(path.to_path_buf());
            }
        }
    }

    files.sort();
    files.dedup();
    files
}

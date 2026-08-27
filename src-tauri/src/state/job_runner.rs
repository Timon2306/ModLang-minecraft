use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

use regex::RegexBuilder;
use rayon::prelude::*;

use crate::core::cache::TranslationCache;
use crate::core::config::AppConfig;
use crate::core::constants::get_languages;
use crate::core::packager::{create_backup, PackWriter};
use crate::core::scanner::discovery::{
    discover_better_questing_files, discover_jar_files, discover_loose_lang_files, discover_snbt_files, get_mod_name
};
use crate::core::scanner::jar_parser::{scan_jar_translatables, JarEntryType, TranslatableJarEntry};
use crate::core::scanner::lang_parser::{
    apply_translations_by_path, load_lenient_json, parse_legacy_lang, serialize_legacy_lang
};
use crate::core::scanner::snbt_parser::{apply_snbt_translations, extract_snbt_strings};
use crate::core::shield::already_translated;
use crate::core::translator::service::TranslationService;
use crate::core::translator::traits::EngineCallbacks;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub current: usize,
    pub total: usize,
    pub percent: u32,
    pub eta: String,
    pub rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub message: String,
    pub color: String,
}

fn to_legacy_caps_code(code: &str) -> String {
    if let Some((lang, country)) = code.split_once('_') {
        format!("{}_{}", lang.to_lowercase(), country.to_uppercase())
    } else {
        code.to_string()
    }
}

pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub cache: TranslationCache,
    pub dictionary: Arc<RwLock<HashMap<String, String>>>,
    pub cancel_token: Arc<RwLock<CancellationToken>>,
    pub pause_flag: Arc<RwLock<bool>>,
    pub is_running: Arc<RwLock<bool>>,
}

impl AppState {
    pub fn new(config: AppConfig, cache: TranslationCache, dictionary: HashMap<String, String>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            cache,
            dictionary: Arc::new(RwLock::new(dictionary)),
            cancel_token: Arc::new(RwLock::new(CancellationToken::new())),
            pause_flag: Arc::new(RwLock::new(false)),
            is_running: Arc::new(RwLock::new(false)),
        }
    }
}

pub async fn run_translation_job(
    app_handle: AppHandle,
    state: Arc<AppState>,
) -> Result<(), String> {
    // 1. Prepare configuration and flags
    let config = state.config.read().clone();
    let mc_dir = PathBuf::from(&config.mc_dir);
    if !mc_dir.exists() {
        return Err("Папка Minecraft не найдена".to_string());
    }

    let target_lang = get_languages()
        .into_iter()
        .find(|l| l.label == config.target_language)
        .ok_or("Целевой язык не найден")?;

    let cancel_token = CancellationToken::new();
    {
        *state.cancel_token.write() = cancel_token.clone();
        *state.pause_flag.write() = false;
        *state.is_running.write() = true;
    }

    let emit_log = {
        let handle = app_handle.clone();
        Arc::new(move |msg: String, color: String| {
            let _ = handle.emit("translation-log", LogEvent { message: msg, color });
        })
    };

    let emit_status = {
        let handle = app_handle.clone();
        Arc::new(move |text: String, progress: Option<f64>| {
            let _ = handle.emit("translation-status", serde_json::json!({
                "text": text,
                "progress": progress
            }));
        })
    };

    let callbacks = EngineCallbacks {
        cancel_token: cancel_token.clone(),
        pause_flag: state.pause_flag.clone(),
        on_log: emit_log.clone(),
        on_status: emit_status.clone(),
    };

    emit_log(format!("[СТАРТ] Запуск перевода ({})\n", target_lang.name), "yellow".to_string());

    // 2. Discover files
    let jars = if config.translate_mods || config.translate_books {
        discover_jar_files(&mc_dir)
    } else {
        Vec::new()
    };
    let loose = if config.translate_mods {
        discover_loose_lang_files(&mc_dir)
    } else {
        Vec::new()
    };
    let snbt = if config.translate_quests {
        discover_snbt_files(&mc_dir)
    } else {
        Vec::new()
    };
    let bq = if config.translate_quests {
        discover_better_questing_files(&mc_dir)
    } else {
        Vec::new()
    };

    if jars.is_empty() && loose.is_empty() && snbt.is_empty() && bq.is_empty() {
        emit_log("[ОШИБКА] Не найдено файлов для перевода!".to_string(), "red".to_string());
        *state.is_running.write() = false;
        return Ok(());
    }

    // 3. Backup if Inplace mode
    if config.output_mode == "inplace" {
        emit_log("[БЭКАП] Создание резервной копии...".to_string(), "cyan".to_string());
        match create_backup(&mc_dir) {
            Ok(b_path) => {
                emit_log(format!("   [ФАЙЛ] Резервная копия: {:?}", b_path), "dim".to_string());
            }
            Err(e) => {
                emit_log(format!("   [ВНИМАНИЕ] Ошибка бэкапа: {}", e), "yellow".to_string());
            }
        }
    }

    // 4. Initialize PackWriter if Resourcepack mode
    let mut pack_writer = if config.output_mode == "resourcepack" {
        match PackWriter::new(&mc_dir, &config.pack_name, &config.mc_version, &target_lang.name) {
            Ok(pw) => {
                emit_log(format!("[ПАКЕТ] Ресурспак: {:?}", pw.rp_zip_path), "cyan".to_string());
                emit_log(format!("[ПАКЕТ] Датапак: {:?}", pw.dp_zip_path), "magenta".to_string());
                Some(pw)
            }
            Err(e) => {
                emit_log(format!("[ОШИБКА] Ошибка создания ресурспака: {}", e), "red".to_string());
                None
            }
        }
    } else {
        None
    };

    let dict = state.dictionary.read().clone();
    let start_time = Instant::now();

    // 4.5 Fast parallel pre-scan of all JARs using all CPU cores
    let scan_start = Instant::now();
    let pre_scanned_jars: Vec<(PathBuf, String, Vec<TranslatableJarEntry>)> = jars
        .par_iter()
        .filter_map(|jar_path| {
            let mod_name = get_mod_name(jar_path);
            if let Ok(entries) = scan_jar_translatables(jar_path, config.translate_mods, config.translate_books, config.smart_glue) {
                if !entries.is_empty() {
                    Some((jar_path.clone(), mod_name, entries))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    let scan_duration = scan_start.elapsed().as_millis();
    emit_log(
        format!("[СКАН] Предзагружено {} модов в ОЗУ за {} мс\n", pre_scanned_jars.len(), scan_duration),
        "green".to_string(),
    );

    let total_items = pre_scanned_jars.len() + loose.len() + snbt.len() + bq.len();
    let processed_items = AtomicUsize::new(0);
    let total_strings_translated = AtomicUsize::new(0);

    // 5. Process JARs (Continuous stream from memory)
    for (_jar_path, mod_name, entries) in pre_scanned_jars {
        if !callbacks.should_run() {
            break;
        }
        callbacks.wait_if_paused().await;

        emit_status(format!("⚡ {}", mod_name), None);
        let service = TranslationService::new(&config, state.cache.clone(), dict.clone(), &mod_name);

        for entry in entries {
            if !callbacks.should_run() {
                break;
            }
            callbacks.wait_if_paused().await;

            let mut to_translate: HashMap<String, String> = HashMap::new();
            for (k, text) in &entry.strings {
                if config.process_mode == "force" || !already_translated(text, &target_lang.regex_pattern) {
                    to_translate.insert(k.clone(), text.clone());
                }
            }

                if to_translate.is_empty() {
                    continue;
                }

                let kind_label = match entry.entry_type {
                    JarEntryType::LangJson | JarEntryType::LangLegacy => "Интерфейс",
                    JarEntryType::BookJson => "Книга JSON",
                    JarEntryType::BookMd => "Книга MD",
                };

                emit_log(
                    format!("[ПЕРЕВОД] {} [{}] — {} строк", mod_name, kind_label, to_translate.len()),
                    "cyan".to_string(),
                );

                let translated_map = service.translate_dict(to_translate.clone(), &target_lang, &callbacks).await;
                let count = translated_map.len();
                total_strings_translated.fetch_add(count, Ordering::SeqCst);

                // Write to resourcepack
                if let Some(pw) = pack_writer.as_mut() {
                    let internal_target_path = match entry.entry_type {
                        JarEntryType::LangJson => {
                            let path = entry.internal_path.clone();
                            if path.to_lowercase().contains("en_us.json") {
                                RegexBuilder::new(r"en_us\.json").case_insensitive(true).build().unwrap().replace(&path, &format!("{}.json", target_lang.file_code)).to_string()
                            } else {
                                path
                            }
                        }
                        JarEntryType::LangLegacy => {
                            let path = entry.internal_path.clone();
                            if path.to_lowercase().contains("en_us.lang") {
                                RegexBuilder::new(r"en_us\.lang").case_insensitive(true).build().unwrap().replace(&path, &format!("{}.lang", target_lang.file_code)).to_string()
                            } else {
                                path
                            }
                        }
                        JarEntryType::BookJson | JarEntryType::BookMd => {
                            let path = entry.internal_path.clone();
                            if path.to_lowercase().contains("/en_us/") {
                                RegexBuilder::new(r"/en_us/").case_insensitive(true).build().unwrap().replace(&path, &format!("/{}/", target_lang.file_code)).to_string()
                            } else {
                                path
                            }
                        }
                    };

                    match entry.entry_type {
                        JarEntryType::LangJson => {
                            let json_bytes = serde_json::to_vec_pretty(&translated_map).unwrap_or_default();
                            let _ = pw.write_entry(&internal_target_path, &json_bytes);
                        }
                        JarEntryType::LangLegacy => {
                            let lang_str = serialize_legacy_lang(&translated_map);
                            let _ = pw.write_entry(&internal_target_path, lang_str.as_bytes());
                            let caps_code = to_legacy_caps_code(&target_lang.file_code);
                            if caps_code != target_lang.file_code {
                                let internal_target_caps = internal_target_path.replace(&format!("{}.lang", target_lang.file_code), &format!("{}.lang", caps_code));
                                let _ = pw.write_entry(&internal_target_caps, lang_str.as_bytes());
                            }
                        }
                        JarEntryType::BookJson => {
                            let mut base_obj = serde_json::json!({});
                            apply_translations_by_path(&mut base_obj, &translated_map);
                            let json_bytes = serde_json::to_vec_pretty(&base_obj).unwrap_or_default();
                            let _ = pw.write_entry(&internal_target_path, &json_bytes);
                        }
                        JarEntryType::BookMd => {
                            let lines: Vec<String> = translated_map.values().cloned().collect();
                            let _ = pw.write_entry(&internal_target_path, lines.join("\n").as_bytes());
                        }
                    }
                }
            }

            let done = processed_items.fetch_add(1, Ordering::SeqCst) + 1;
            let elapsed = start_time.elapsed().as_secs_f64();
            let rate = if elapsed > 0.0 { done as f64 / elapsed } else { 0.0 };
            let eta_secs = if rate > 0.0 { (total_items - done) as f64 / rate } else { 0.0 };

            let _ = app_handle.emit(
                "translation-progress",
                ProgressEvent {
                    current: done,
                    total: total_items,
                    percent: ((done as f64 / total_items as f64) * 100.0).round() as u32,
                    eta: format_eta(eta_secs),
                    rate,
                },
            );
        }

    // 5.5 Process Loose Resource Files (resources/skyfactory4/lang/en_us.lang, kubejs/assets/..., etc.)
    for loose_path in &loose {
        if !callbacks.should_run() {
            break;
        }
        callbacks.wait_if_paused().await;

        let file_name = loose_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        emit_status(format!("📄 {}", file_name), None);
        let fl = file_name.to_lowercase();
        let is_lang = fl.ends_with(".lang");
        let is_json = fl.ends_with(".json");

        let mut strings_map = HashMap::new();
        if is_lang {
            if let Ok(content) = fs::read_to_string(loose_path) {
                strings_map = parse_legacy_lang(&content);
            }
        } else if is_json {
            if let Ok(content) = fs::read(loose_path) {
                if let Ok(json_val) = load_lenient_json(&content) {
                    if let Some(obj) = json_val.as_object() {
                        for (k, v) in obj {
                            if let Some(s) = v.as_str() {
                                strings_map.insert(k.clone(), s.to_string());
                            }
                        }
                    }
                }
            }
        }

        let mut to_translate = HashMap::new();
        for (k, text) in &strings_map {
            if config.process_mode == "force" || !already_translated(text, &target_lang.regex_pattern) {
                to_translate.insert(k.clone(), text.clone());
            }
        }

        if !to_translate.is_empty() {
            let display_name = loose_path.strip_prefix(&mc_dir)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| file_name.to_string());

            emit_log(
                format!("⚡ Перевод {} [Ресурсы сборки] — {} строк", display_name, to_translate.len()),
                "cyan".to_string(),
            );

            let service = TranslationService::new(&config, state.cache.clone(), dict.clone(), &display_name);
            let translated_map = service.translate_dict(to_translate, &target_lang, &callbacks).await;
            let count = translated_map.len();
            total_strings_translated.fetch_add(count, Ordering::SeqCst);

            let mut final_map = strings_map.clone();
            for (k, v) in translated_map {
                final_map.insert(k, v);
            }

            // Save In-place
            if config.output_mode == "inplace" {
                if let Some(parent) = loose_path.parent() {
                    if is_lang {
                        let target_path = parent.join(format!("{}.lang", target_lang.file_code));
                        let _ = fs::write(&target_path, serialize_legacy_lang(&final_map));
                        let caps_code = to_legacy_caps_code(&target_lang.file_code);
                        if caps_code != target_lang.file_code {
                            let target_path_caps = parent.join(format!("{}.lang", caps_code));
                            let _ = fs::write(&target_path_caps, serialize_legacy_lang(&final_map));
                        }
                    } else if is_json {
                        let target_path = parent.join(format!("{}.json", target_lang.file_code));
                        let _ = fs::write(&target_path, serde_json::to_string_pretty(&final_map).unwrap_or_default());
                    }
                }
            }

            // Write to Resource Pack
            if let Some(pw) = pack_writer.as_mut() {
                if let Ok(rel) = loose_path.strip_prefix(&mc_dir) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    let internal_target = if rel_str.starts_with("resources/") {
                        let rest = &rel_str["resources/".len()..];
                        let replaced = RegexBuilder::new(r"en_us\.").case_insensitive(true).build().unwrap().replace(rest, &format!("{}.", target_lang.file_code)).to_string();
                        format!("assets/{}", replaced)
                    } else if rel_str.starts_with("kubejs/assets/") {
                        let rest = &rel_str["kubejs/assets/".len()..];
                        let replaced = RegexBuilder::new(r"en_us\.").case_insensitive(true).build().unwrap().replace(rest, &format!("{}.", target_lang.file_code)).to_string();
                        format!("assets/{}", replaced)
                    } else {
                        let replaced = RegexBuilder::new(r"en_us\.").case_insensitive(true).build().unwrap().replace(&rel_str, &format!("{}.", target_lang.file_code)).to_string();
                        format!("assets/{}", replaced)
                    };

                    if is_lang {
                        let _ = pw.write_entry(&internal_target, serialize_legacy_lang(&final_map).as_bytes());
                        let caps_code = to_legacy_caps_code(&target_lang.file_code);
                        if caps_code != target_lang.file_code {
                            let internal_target_caps = internal_target.replace(&format!("{}.lang", target_lang.file_code), &format!("{}.lang", caps_code));
                            let _ = pw.write_entry(&internal_target_caps, serialize_legacy_lang(&final_map).as_bytes());
                        }
                    } else if is_json {
                        let json_bytes = serde_json::to_vec_pretty(&final_map).unwrap_or_default();
                        let _ = pw.write_entry(&internal_target, &json_bytes);
                    }
                }
            }
        }

        let done = processed_items.fetch_add(1, Ordering::SeqCst) + 1;
        let elapsed = start_time.elapsed().as_secs_f64();
        let rate = if elapsed > 0.0 { done as f64 / elapsed } else { 0.0 };
        let eta_secs = if rate > 0.0 { (total_items - done) as f64 / rate } else { 0.0 };

        let _ = app_handle.emit(
            "translation-progress",
            ProgressEvent {
                current: done,
                total: total_items,
                percent: ((done as f64 / total_items as f64) * 100.0).round() as u32,
                eta: format_eta(eta_secs),
                rate,
            },
        );
    }

    // 6. Process Quests (.snbt)
    for snbt_path in &snbt {
        if !callbacks.should_run() {
            break;
        }
        callbacks.wait_if_paused().await;

        let file_name = snbt_path.file_name().and_then(|n| n.to_str()).unwrap_or("quests.snbt");
        emit_status(format!("📜 {}", file_name), None);
        if let Ok(content) = fs::read_to_string(snbt_path) {
            let skip_re = if config.process_mode == "append" {
                Some(target_lang.regex_pattern.as_str())
            } else {
                None
            };
            let strings = extract_snbt_strings(&content, skip_re);
            if !strings.is_empty() {
                let file_name = snbt_path.file_name().and_then(|n| n.to_str()).unwrap_or("quests.snbt");
                emit_log(
                    format!("⚡ Перевод {} [Квесты] — {} строк", file_name, strings.len()),
                    "yellow".to_string(),
                );

                let mut to_translate = HashMap::new();
                for (idx, s) in strings.iter().enumerate() {
                    to_translate.insert(idx.to_string(), s.clone());
                }

                let service = TranslationService::new(&config, state.cache.clone(), dict.clone(), file_name);
                let translated = service.translate_dict(to_translate, &target_lang, &callbacks).await;

                let mut mapping = HashMap::new();
                for (idx, s) in strings.iter().enumerate() {
                    if let Some(trans_val) = translated.get(&idx.to_string()) {
                        mapping.insert(s.clone(), trans_val.clone());
                    }
                }

                let new_snbt = apply_snbt_translations(&content, &mapping);
                let _ = fs::write(snbt_path, new_snbt);
                total_strings_translated.fetch_add(mapping.len(), Ordering::SeqCst);
            }
        }

        let done = processed_items.fetch_add(1, Ordering::SeqCst) + 1;
        let elapsed = start_time.elapsed().as_secs_f64();
        let rate = if elapsed > 0.0 { done as f64 / elapsed } else { 0.0 };
        let eta_secs = if rate > 0.0 { (total_items - done) as f64 / rate } else { 0.0 };

        let _ = app_handle.emit(
            "translation-progress",
            ProgressEvent {
                current: done,
                total: total_items,
                percent: ((done as f64 / total_items as f64) * 100.0).round() as u32,
                eta: format_eta(eta_secs),
                rate,
            },
        );
    }

    // 6b. Process Better Questing Quests (.lang and .json)
    for bq_path in &bq {
        if !callbacks.should_run() {
            break;
        }
        callbacks.wait_if_paused().await;

        let file_name = bq_path.file_name().and_then(|n| n.to_str()).unwrap_or("quests");
        let fl = file_name.to_lowercase();
        let display_name = bq_path.strip_prefix(&mc_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file_name.to_string());

        emit_status(format!("📜 {}", file_name), None);

        if fl.ends_with(".lang") {
            if let Ok(content) = fs::read_to_string(bq_path) {
                let strings_map = parse_legacy_lang(&content);
                let mut to_translate = HashMap::new();
                for (k, text) in &strings_map {
                    if config.process_mode == "force" || !already_translated(text, &target_lang.regex_pattern) {
                        to_translate.insert(k.clone(), text.clone());
                    }
                }

                if !to_translate.is_empty() {
                    emit_log(
                        format!("⚡ Перевод {} [Квесты BQ] — {} строк", display_name, to_translate.len()),
                        "yellow".to_string(),
                    );

                    let service = TranslationService::new(&config, state.cache.clone(), dict.clone(), &display_name);
                    let translated_map = service.translate_dict(to_translate, &target_lang, &callbacks).await;
                    let count = translated_map.len();
                    total_strings_translated.fetch_add(count, Ordering::SeqCst);

                    let mut final_map = strings_map.clone();
                    for (k, v) in translated_map {
                        final_map.insert(k, v);
                    }

                    // Save In-place
                    if let Some(parent) = bq_path.parent() {
                        let target_path = parent.join(format!("{}.lang", target_lang.file_code));
                        let _ = fs::write(&target_path, serialize_legacy_lang(&final_map));
                        let caps_code = to_legacy_caps_code(&target_lang.file_code);
                        if caps_code != target_lang.file_code {
                            let target_path_caps = parent.join(format!("{}.lang", caps_code));
                            let _ = fs::write(&target_path_caps, serialize_legacy_lang(&final_map));
                        }
                    }

                    // Write to Resource Pack
                    if let Some(pw) = pack_writer.as_mut() {
                        let lang_str = serialize_legacy_lang(&final_map);
                        let _ = pw.write_entry(&format!("assets/betterquesting/lang/{}.lang", target_lang.file_code), lang_str.as_bytes());
                        let caps_code = to_legacy_caps_code(&target_lang.file_code);
                        if caps_code != target_lang.file_code {
                            let _ = pw.write_entry(&format!("assets/betterquesting/lang/{}.lang", caps_code), lang_str.as_bytes());
                        }
                    }
                }
            }
        } else if fl.ends_with(".json") {
            if let Ok(content) = fs::read(bq_path) {
                if let Ok(mut json_val) = load_lenient_json(&content) {
                    let pairs = crate::core::scanner::bq_parser::extract_better_questing_strings(&json_val);
                    let mut to_translate = HashMap::new();
                    for (k, text) in &pairs {
                        if config.process_mode == "force" || !already_translated(text, &target_lang.regex_pattern) {
                            to_translate.insert(k.clone(), text.clone());
                        }
                    }

                    if !to_translate.is_empty() {
                        emit_log(
                            format!("⚡ Перевод {} [Квесты BQ] — {} строк", display_name, to_translate.len()),
                            "yellow".to_string(),
                        );

                        let service = TranslationService::new(&config, state.cache.clone(), dict.clone(), &display_name);
                        let translated_map = service.translate_dict(to_translate, &target_lang, &callbacks).await;
                        let count = translated_map.len();
                        total_strings_translated.fetch_add(count, Ordering::SeqCst);

                        crate::core::scanner::bq_parser::apply_better_questing_translations(&mut json_val, &translated_map);
                        let json_bytes = serde_json::to_vec_pretty(&json_val).unwrap_or_default();
                        let _ = fs::write(bq_path, json_bytes);
                    }
                }
            }
        }

        let done = processed_items.fetch_add(1, Ordering::SeqCst) + 1;
        let elapsed = start_time.elapsed().as_secs_f64();
        let rate = if elapsed > 0.0 { done as f64 / elapsed } else { 0.0 };
        let eta_secs = if rate > 0.0 { (total_items - done) as f64 / rate } else { 0.0 };

        let _ = app_handle.emit(
            "translation-progress",
            ProgressEvent {
                current: done,
                total: total_items,
                percent: ((done as f64 / total_items as f64) * 100.0).round() as u32,
                eta: format_eta(eta_secs),
                rate,
            },
        );
    }

    // 7. Finish
    if let Some(mut pw) = pack_writer {
        let _ = pw.close();
    }
    state.cache.save();

    if callbacks.should_run() {
        emit_log("\n[УСПЕХ] Перевод успешно завершен!".to_string(), "green".to_string());
        if config.output_mode == "resourcepack" {
            emit_log("[ИНФО] Включите Resource Pack в игре Minecraft (F3 + T для перезагрузки).".to_string(), "yellow".to_string());
        }
        emit_status("Готово!".to_string(), Some(1.0));
    } else {
        emit_log("\n[СТОП] Остановлено пользователем.".to_string(), "red".to_string());
        emit_status("Остановлено".to_string(), Some(1.0));
    }

    *state.is_running.write() = false;
    Ok(())
}

fn format_eta(seconds: f64) -> String {
    if seconds <= 0.0 || seconds.is_nan() || seconds.is_infinite() {
        return "готово".to_string();
    }
    if seconds < 60.0 {
        format!("≈ {} сек", seconds.round() as u64)
    } else if seconds < 3600.0 {
        format!("≈ {} мин {} сек", (seconds / 60.0).floor() as u64, (seconds % 60.0).round() as u64)
    } else {
        format!("≈ {} ч {} мин", (seconds / 3600.0).floor() as u64, ((seconds % 3600.0) / 60.0).floor() as u64)
    }
}

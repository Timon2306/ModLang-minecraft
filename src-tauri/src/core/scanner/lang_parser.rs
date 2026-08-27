use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use serde_json::Value;
use crate::core::constants::KEYS_TO_TRANSLATE;

lazy_static! {
    static ref BLOCK_COMMENT_RE: Regex = RegexBuilder::new(r"/\*.*?\*/")
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    static ref LINE_COMMENT_RE: Regex = RegexBuilder::new(r"(?m)^\s*//.*$")
        .build()
        .unwrap();

    static ref TRAILING_COMMA_RE: Regex = Regex::new(r",\s*([\]}])").unwrap();
}

pub fn load_lenient_json(raw: &[u8]) -> Result<Value, String> {
    let mut text = String::from_utf8_lossy(raw).to_string();
    if text.starts_with('\u{feff}') {
        text.remove(0);
    }
    let no_block = BLOCK_COMMENT_RE.replace_all(&text, "");
    let no_line = LINE_COMMENT_RE.replace_all(&no_block, "");
    let clean = TRAILING_COMMA_RE.replace_all(&no_line, "$1");
    serde_json::from_str::<Value>(&clean).map_err(|e| e.to_string())
}

pub fn parse_legacy_lang(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            map.insert(k.trim().to_string(), v.to_string());
        }
    }
    map
}

pub fn serialize_legacy_lang(map: &HashMap<String, String>) -> String {
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    let mut lines = Vec::with_capacity(keys.len());
    for k in keys {
        lines.push(format!("{}={}", k, map.get(k).unwrap()));
    }
    lines.join("\n")
}

pub fn iter_translatable_strings(data: &Value, path: Vec<String>) -> Vec<(Vec<String>, String)> {
    let mut result = Vec::new();
    match data {
        Value::Object(map) => {
            for (key, val) in map {
                let mut child_path = path.clone();
                child_path.push(key.clone());

                if KEYS_TO_TRANSLATE.contains(&key.as_str()) {
                    match val {
                        Value::String(s) => {
                            result.push((child_path, s.clone()));
                        }
                        Value::Array(arr) => {
                            for (idx, item) in arr.iter().enumerate() {
                                if let Value::String(s) = item {
                                    let mut item_path = child_path.clone();
                                    item_path.push(idx.to_string());
                                    result.push((item_path, s.clone()));
                                }
                            }
                        }
                        _ => {}
                    }
                } else if val.is_object() || val.is_array() {
                    result.extend(iter_translatable_strings(val, child_path));
                }
            }
        }
        Value::Array(arr) => {
            for (idx, item) in arr.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(idx.to_string());
                result.extend(iter_translatable_strings(item, child_path));
            }
        }
        _ => {}
    }
    result
}

pub fn apply_translations_by_path(data: &mut Value, translations: &HashMap<String, String>) {
    for (path_key, translated_str) in translations {
        let parts: Vec<&str> = path_key.split('/').collect();
        set_at_path(data, &parts, translated_str);
    }
}

fn set_at_path(data: &mut Value, path: &[&str], value: &str) {
    if path.is_empty() {
        return;
    }
    let mut cur = data;
    for &part in &path[..path.len() - 1] {
        if let Ok(idx) = part.parse::<usize>() {
            if let Value::Array(arr) = cur {
                if idx < arr.len() {
                    cur = &mut arr[idx];
                } else {
                    return;
                }
            } else {
                return;
            }
        } else if let Value::Object(map) = cur {
            if let Some(next) = map.get_mut(part) {
                cur = next;
            } else {
                return;
            }
        } else {
            return;
        }
    }

    let last = path[path.len() - 1];
    if let Ok(idx) = last.parse::<usize>() {
        if let Value::Array(arr) = cur {
            if idx < arr.len() {
                arr[idx] = Value::String(value.to_string());
            }
        }
    } else if let Value::Object(map) = cur {
        map.insert(last.to_string(), Value::String(value.to_string()));
    }
}

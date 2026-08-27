use std::collections::HashMap;
use serde_json::Value;
use crate::core::shield::looks_like_source_language;

pub fn extract_better_questing_strings(data: &Value) -> Vec<(String, String)> {
    let mut result = Vec::new();
    
    // 1. Check standard DefaultQuests.json format (questDatabase)
    let quest_db = data.get("questDatabase:9").or_else(|| data.get("questDatabase"));
    if let Some(Value::Object(map)) = quest_db {
        for (q_id, q_val) in map {
            let props_opt = q_val.get("properties:10").or_else(|| q_val.get("properties"));
            if let Some(props_val) = props_opt {
                let bq_opt = props_val.get("betterquesting:10").or_else(|| props_val.get("betterquesting"));
                if let Some(bq_val) = bq_opt {
                    if let Some(name) = bq_val.get("name:8").or_else(|| bq_val.get("name")).and_then(|v| v.as_str()) {
                        if looks_like_source_language(name) && !name.trim().is_empty() {
                            result.push((format!("{}/name", q_id), name.to_string()));
                        }
                    }
                    if let Some(desc) = bq_val.get("desc:8").or_else(|| bq_val.get("desc")).and_then(|v| v.as_str()) {
                        if looks_like_source_language(desc) && !desc.trim().is_empty() {
                            result.push((format!("{}/desc", q_id), desc.to_string()));
                        }
                    }
                }
            }
        }
    }

    // 2. Check questLines in DefaultQuests.json
    let quest_lines = data.get("questLines:9").or_else(|| data.get("questLines"));
    if let Some(Value::Object(map)) = quest_lines {
        for (ql_id, ql_val) in map {
            let props_opt = ql_val.get("properties:10").or_else(|| ql_val.get("properties"));
            if let Some(props_val) = props_opt {
                let bq_opt = props_val.get("betterquesting:10").or_else(|| props_val.get("betterquesting"));
                if let Some(bq_val) = bq_opt {
                    if let Some(name) = bq_val.get("name:8").or_else(|| bq_val.get("name")).and_then(|v| v.as_str()) {
                        if looks_like_source_language(name) && !name.trim().is_empty() {
                            result.push((format!("line_{}/name", ql_id), name.to_string()));
                        }
                    }
                    if let Some(desc) = bq_val.get("desc:8").or_else(|| bq_val.get("desc")).and_then(|v| v.as_str()) {
                        if looks_like_source_language(desc) && !desc.trim().is_empty() {
                            result.push((format!("line_{}/desc", ql_id), desc.to_string()));
                        }
                    }
                }
            }
        }
    }

    // 3. Check modular single quest / questline JSON file (properties at root)
    let root_props = data.get("properties:10").or_else(|| data.get("properties"));
    if let Some(props_val) = root_props {
        let bq_opt = props_val.get("betterquesting:10").or_else(|| props_val.get("betterquesting"));
        if let Some(bq_val) = bq_opt {
            if let Some(name) = bq_val.get("name:8").or_else(|| bq_val.get("name")).and_then(|v| v.as_str()) {
                if looks_like_source_language(name) && !name.trim().is_empty() {
                    result.push(("root/name".to_string(), name.to_string()));
                }
            }
            if let Some(desc) = bq_val.get("desc:8").or_else(|| bq_val.get("desc")).and_then(|v| v.as_str()) {
                if looks_like_source_language(desc) && !desc.trim().is_empty() {
                    result.push(("root/desc".to_string(), desc.to_string()));
                }
            }
        }
    }

    result
}

pub fn apply_better_questing_translations(data: &mut Value, translations: &HashMap<String, String>) {
    // 1. Check root properties (modular file)
    let root_props = if data.get_mut("properties:10").is_some() {
        data.get_mut("properties:10")
    } else {
        data.get_mut("properties")
    };
    if let Some(props_val) = root_props {
        let bq_opt = if props_val.get_mut("betterquesting:10").is_some() {
            props_val.get_mut("betterquesting:10")
        } else {
            props_val.get_mut("betterquesting")
        };
        if let Some(Value::Object(bq_map)) = bq_opt {
            if let Some(name) = translations.get("root/name") {
                if bq_map.contains_key("name:8") {
                    bq_map.insert("name:8".to_string(), Value::String(name.clone()));
                } else {
                    bq_map.insert("name".to_string(), Value::String(name.clone()));
                }
            }
            if let Some(desc) = translations.get("root/desc") {
                if bq_map.contains_key("desc:8") {
                    bq_map.insert("desc:8".to_string(), Value::String(desc.clone()));
                } else {
                    bq_map.insert("desc".to_string(), Value::String(desc.clone()));
                }
            }
        }
    }

    // 2. Check questDatabase
    let quest_db = if data.get_mut("questDatabase:9").is_some() {
        data.get_mut("questDatabase:9")
    } else {
        data.get_mut("questDatabase")
    };

    if let Some(Value::Object(map)) = quest_db {
        for (key, translated_text) in translations {
            if let Some((q_id, field)) = key.split_once('/') {
                if let Some(q_val) = map.get_mut(q_id) {
                    let props_opt = if q_val.get_mut("properties:10").is_some() {
                        q_val.get_mut("properties:10")
                    } else {
                        q_val.get_mut("properties")
                    };

                    if let Some(props_val) = props_opt {
                        let bq_opt = if props_val.get_mut("betterquesting:10").is_some() {
                            props_val.get_mut("betterquesting:10")
                        } else {
                            props_val.get_mut("betterquesting")
                        };

                        if let Some(Value::Object(bq_map)) = bq_opt {
                            if field == "name" {
                                if bq_map.contains_key("name:8") {
                                    bq_map.insert("name:8".to_string(), Value::String(translated_text.clone()));
                                } else {
                                    bq_map.insert("name".to_string(), Value::String(translated_text.clone()));
                                }
                            } else if field == "desc" {
                                if bq_map.contains_key("desc:8") {
                                    bq_map.insert("desc:8".to_string(), Value::String(translated_text.clone()));
                                } else {
                                    bq_map.insert("desc".to_string(), Value::String(translated_text.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Check questLines
    let quest_lines = if data.get_mut("questLines:9").is_some() {
        data.get_mut("questLines:9")
    } else {
        data.get_mut("questLines")
    };

    if let Some(Value::Object(map)) = quest_lines {
        for (key, translated_text) in translations {
            if let Some((ql_id, field)) = key.split_once('/') {
                if let Some(ql_id_clean) = ql_id.strip_prefix("line_") {
                    if let Some(ql_val) = map.get_mut(ql_id_clean) {
                        let props_opt = if ql_val.get_mut("properties:10").is_some() {
                            ql_val.get_mut("properties:10")
                        } else {
                            ql_val.get_mut("properties")
                        };

                        if let Some(props_val) = props_opt {
                            let bq_opt = if props_val.get_mut("betterquesting:10").is_some() {
                                props_val.get_mut("betterquesting:10")
                            } else {
                                props_val.get_mut("betterquesting")
                            };

                            if let Some(Value::Object(bq_map)) = bq_opt {
                                if field == "name" {
                                    if bq_map.contains_key("name:8") {
                                        bq_map.insert("name:8".to_string(), Value::String(translated_text.clone()));
                                    } else {
                                        bq_map.insert("name".to_string(), Value::String(translated_text.clone()));
                                    }
                                } else if field == "desc" {
                                    if bq_map.contains_key("desc:8") {
                                        bq_map.insert("desc:8".to_string(), Value::String(translated_text.clone()));
                                    } else {
                                        bq_map.insert("desc".to_string(), Value::String(translated_text.clone()));
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

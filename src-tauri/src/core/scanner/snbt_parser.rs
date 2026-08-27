use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use crate::core::shield::{is_translation_key, looks_like_source_language};

lazy_static! {
    static ref SNBT_STRING_RE: Regex = Regex::new(r#""((?:[^"\\]|\\.)*)""#).unwrap();

    static ref SINGLE_FIELD_RE: Regex = RegexBuilder::new(
        r#"(?:"|)(title|subtitle|text|desc|description)(?:"|)\s*:\s*"((?:[^"\\]|\\.)*)""#
    )
    .case_insensitive(true)
    .build()
    .unwrap();

    static ref ARRAY_BLOCK_RE: Regex = RegexBuilder::new(
        r#"(?:"|)(description|text|desc)(?:"|)\s*:\s*(\[\s*(?:"(?:[^"\\]|\\.)*"\s*,?\s*)*\])"#
    )
    .case_insensitive(true)
    .build()
    .unwrap();
}

pub fn extract_snbt_strings(content: &str, skip_translated_regex: Option<&str>) -> Vec<String> {
    let mut strings = Vec::new();
    let skip_re = skip_translated_regex.and_then(|r| Regex::new(r).ok());

    // 1. Single fields
    for cap in SINGLE_FIELD_RE.captures_iter(content) {
        if let Some(m) = cap.get(2) {
            let val = m.as_str().to_string();
            let trimmed = val.trim();
            if trimmed.is_empty() || is_translation_key(trimmed) || !looks_like_source_language(trimmed) {
                continue;
            }
            if let Some(re) = &skip_re {
                if re.is_match(&val) {
                    continue;
                }
            }
            strings.push(val);
        }
    }

    // 2. Array fields
    for block in ARRAY_BLOCK_RE.find_iter(content) {
        for cap in SNBT_STRING_RE.captures_iter(block.as_str()) {
            if let Some(m) = cap.get(1) {
                let val = m.as_str().to_string();
                let trimmed = val.trim();
                if trimmed.is_empty() || is_translation_key(trimmed) || !looks_like_source_language(trimmed) {
                    continue;
                }
                if let Some(re) = &skip_re {
                    if re.is_match(&val) {
                        continue;
                    }
                }
                strings.push(val);
            }
        }
    }

    // Deduplicate preserving order
    let mut seen = std::collections::HashSet::new();
    strings.retain(|s| seen.insert(s.clone()));
    strings
}

pub fn apply_snbt_translations(content: &str, mapping: &HashMap<String, String>) -> String {
    // 1. Replace single fields
    let step1 = SINGLE_FIELD_RE.replace_all(content, |caps: &regex::Captures| {
        let field_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let original_val = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        
        let translated = mapping.get(original_val).map(|s| s.as_str()).unwrap_or(original_val);
        let escaped = translated.replace('\\', "\\\\").replace('"', "\\\"");
        format!("{}: \"{}\"", field_name, escaped)
    });

    // 2. Replace array fields
    let step2 = ARRAY_BLOCK_RE.replace_all(&step1, |caps: &regex::Captures| {
        let field_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let array_body = caps.get(2).map(|m| m.as_str()).unwrap_or("");

        let replaced_array = SNBT_STRING_RE.replace_all(array_body, |item_caps: &regex::Captures| {
            let original_val = item_caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let translated = mapping.get(original_val).map(|s| s.as_str()).unwrap_or(original_val);
            let escaped = translated.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{}\"", escaped)
        });

        format!("{}: {}", field_name, replaced_array)
    });

    step2.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snbt_extract_and_replace() {
        let snbt_sample = r#"
        {
            id: "123456789",
            title: "Getting Started with RF",
            subtitle: "Build your first generator",
            description: [
                "Generators produce FE and RF power.",
                "Connect it with cables to your machines."
            ],
            tasks: [
                { id: "task_1", type: "item", item: "minecraft:iron_ingot" }
            ]
        }
        "#;

        let extracted = extract_snbt_strings(snbt_sample, None);
        assert_eq!(extracted.len(), 4);
        assert!(extracted.contains(&"Getting Started with RF".to_string()));
        assert!(extracted.contains(&"Build your first generator".to_string()));

        let mut mapping = HashMap::new();
        mapping.insert("Getting Started with RF".to_string(), "Начало работы с RF".to_string());
        mapping.insert("Build your first generator".to_string(), "Создайте свой первый генератор".to_string());
        mapping.insert("Generators produce FE and RF power.".to_string(), "Генераторы производят энергию FE и RF.".to_string());
        mapping.insert("Connect it with cables to your machines.".to_string(), "Подключите их проводами к вашим машинам.".to_string());

        let translated_snbt = apply_snbt_translations(snbt_sample, &mapping);
        assert!(translated_snbt.contains(r#"title: "Начало работы с RF""#));
        assert!(translated_snbt.contains(r#"subtitle: "Создайте свой первый генератор""#));
        assert!(translated_snbt.contains(r#""Генераторы производят энергию FE и RF.""#));
    }
}

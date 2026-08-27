use std::collections::{HashMap, HashSet};
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use crate::core::constants::IGNORE_TERMS;

lazy_static! {
    pub static ref PLACEHOLDER_RE: Regex = Regex::new(r"\[\s*#\s*(\d+)\s*#\s*\]").unwrap();
    
    pub static ref FORMAT_RE: Regex = RegexBuilder::new(
        r"(\$\([^)]+\)|[&§][0-9a-fk-orlmn]|<[^>]+>|\{[^\}]+\}|\]\([^)]+\)|!\[[^\]]*\]|\[[a-z0-9_.-]+:[a-z0-9_./-]+\]|\([a-z0-9_.-]+:[a-z0-9_./-]+\)|\([A-Za-z0-9_./-]+\.md[#a-zA-Z0-9_-]*\)|\r?\n|%[0-9.,]*\$?[a-zA-Z%])"
    )
    .case_insensitive(true)
    .build()
    .unwrap();

    pub static ref IGNORE_RE: Regex = {
        let mut escaped: Vec<String> = IGNORE_TERMS.iter().map(|t| regex::escape(t)).collect();
        escaped.sort_by(|a, b| b.len().cmp(&a.len()));
        let pattern = format!(r"\b({})\b", escaped.join("|"));
        Regex::new(&pattern).unwrap()
    };

    pub static ref NEWLINE_GLUE_RE: Regex = Regex::new(r"\s*(?:\\n|\r?\n)\s*").unwrap();

    pub static ref LETTER_RE: Regex = Regex::new(r"[a-zA-Z]").unwrap();
    pub static ref KEY_FORMAT_RE: Regex = Regex::new(r"^[a-zA-Z0-9_-]+[.:][a-zA-Z0-9_.-]+$").unwrap();
    pub static ref TECH_IDENTIFIER_RE: Regex = Regex::new(r"^[a-z0-9_.-]+$").unwrap();
    pub static ref MULTI_SPACE_RE: Regex = Regex::new(r"\s+").unwrap();
}

pub fn apply_smart_glue(text: &str) -> String {
    if text.is_empty() {
        return text.to_string();
    }
    
    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;

    for m in NEWLINE_GLUE_RE.find_iter(text) {
        let prefix = &text[..m.start()];
        let suffix = &text[m.end()..];

        // Check if prefix ends with sentence punctuation or list markers
        let should_keep = if let Some(last_char) = prefix.trim_end().chars().last() {
            matches!(last_char, '.' | '!' | '?' | '>' | ']' | ':' | '#')
        } else {
            false
        } || if let Some(first_char) = suffix.trim_start().chars().next() {
            matches!(first_char, '-' | '*' | '#' | '<' | '|')
        } else {
            false
        };

        result.push_str(&text[last_end..m.start()]);
        if should_keep {
            result.push('\n');
        } else {
            result.push(' ');
        }
        last_end = m.end();
    }
    result.push_str(&text[last_end..]);
    result
}

pub fn mask_protected_fragments(text: &str) -> (String, HashMap<String, String>) {
    let mut mapping: HashMap<String, String> = HashMap::new();
    let mut reserved_ids: HashSet<usize> = HashSet::new();

    for cap in PLACEHOLDER_RE.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            if let Ok(id) = m.as_str().parse::<usize>() {
                reserved_ids.insert(id);
            }
        }
    }

    let mut next_id = 0usize;
    let mut intermediate = String::with_capacity(text.len());
    let mut last_end = 0;

    // First pass: replace format patterns
    for m in FORMAT_RE.find_iter(text) {
        intermediate.push_str(&text[last_end..m.start()]);
        while reserved_ids.contains(&next_id) {
            next_id += 1;
        }
        let token = format!("[#{next_id}#]");
        next_id += 1;
        mapping.insert(token.clone(), m.as_str().to_string());
        intermediate.push_str(&token);
        last_end = m.end();
    }
    intermediate.push_str(&text[last_end..]);

    // Second pass: replace ignore terms in intermediate
    let mut final_result = String::with_capacity(intermediate.len());
    last_end = 0;

    for m in IGNORE_RE.find_iter(&intermediate) {
        final_result.push_str(&intermediate[last_end..m.start()]);
        while reserved_ids.contains(&next_id) {
            next_id += 1;
        }
        let token = format!("[#{next_id}#]");
        next_id += 1;
        mapping.insert(token.clone(), m.as_str().to_string());
        final_result.push_str(&token);
        last_end = m.end();
    }
    final_result.push_str(&intermediate[last_end..]);

    let cleaned = MULTI_SPACE_RE.replace_all(final_result.trim(), " ").to_string();
    (cleaned, mapping)
}

pub fn unmask_translation(text: &str, mapping: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (token, original) in mapping {
        let idx = token.trim_matches(|c| c == '#' || c == '[' || c == ']');
        if let Ok(re) = Regex::new(&format!(r"\[\s*#\s*{}\s*#\s*\]", regex::escape(idx))) {
            result = re.replace_all(&result, original.as_str()).to_string();
        }
    }
    result
}

pub fn is_technical_term(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }
    let lower = trimmed.to_lowercase();
    if !LETTER_RE.is_match(&lower) {
        return true;
    }
    if TECH_IDENTIFIER_RE.is_match(&lower) && (lower.contains('.') || lower.contains('_')) {
        return true;
    }
    let prefixes = [
        "glyph_", "ritual_", "familiar_", "source_", "mana_", "spell_",
        "effect_", "rune_", "altar_", "botania_", "create_", "kubejs_",
    ];
    for p in &prefixes {
        if lower.starts_with(p) {
            return true;
        }
    }
    false
}

pub fn is_translation_key(text: &str) -> bool {
    KEY_FORMAT_RE.is_match(text.trim())
}

pub fn looks_like_source_language(text: &str) -> bool {
    LETTER_RE.is_match(text)
}

pub fn already_translated(text: &str, target_regex: &str) -> bool {
    if let Ok(re) = Regex::new(target_regex) {
        re.is_match(text)
    } else {
        false
    }
}

pub fn placeholders_match(translated: &str, expected_masked: &str) -> bool {
    let mut expected_counts: HashMap<String, usize> = HashMap::new();
    let mut actual_counts: HashMap<String, usize> = HashMap::new();

    for cap in PLACEHOLDER_RE.captures_iter(expected_masked) {
        if let Some(m) = cap.get(1) {
            *expected_counts.entry(m.as_str().to_string()).or_insert(0) += 1;
        }
    }

    for cap in PLACEHOLDER_RE.captures_iter(translated) {
        if let Some(m) = cap.get(1) {
            *actual_counts.entry(m.as_str().to_string()).or_insert(0) += 1;
        }
    }

    expected_counts == actual_counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_and_unmask() {
        let source = "Power §a%s§r\n[docs](guide.md) RF";
        let (masked, mapping) = mask_protected_fragments(source);
        assert!(masked.contains("[#0#]"));
        
        let translated = masked.replace("Power", "Мощность");
        let unmasked = unmask_translation(&translated, &mapping);
        assert_eq!(unmasked, "Мощность §a%s§r\n[docs](guide.md) RF");
    }

    #[test]
    fn test_placeholders_match() {
        let original_masked = "Value [#0#] and [#1#]";
        let valid_trans = "Значение [#0#] и [#1#]";
        let invalid_missing = "Значение [#0#]";
        let invalid_dup = "Значение [#0#] [#0#]";
        let invalid_extra = "Значение [#0#] [#1#] [#2#]";

        assert!(placeholders_match(valid_trans, original_masked));
        assert!(!placeholders_match(invalid_missing, original_masked));
        assert!(!placeholders_match(invalid_dup, original_masked));
        assert!(!placeholders_match(invalid_extra, original_masked));
    }

    #[test]
    fn test_technical_terms() {
        assert!(is_technical_term("item.minecraft.diamond"));
        assert!(is_technical_term("glyph_press"));
        assert!(is_technical_term("12345"));
        assert!(!is_technical_term("Diamond Sword"));
        assert!(!is_technical_term("Requires 500 RF to operate"));
    }
}

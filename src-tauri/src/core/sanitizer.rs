use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref FIX_COLOR_INNER_SPACE_RE: Regex = RegexBuilder::new(r"([&§])\s+([0-9a-fk-or])")
        .case_insensitive(true).build().unwrap();

    static ref FIX_COLOR_TRAILING_SPACE_RE: Regex = RegexBuilder::new(r"([&§][0-9a-fk-o])\s+")
        .case_insensitive(true).build().unwrap();

    static ref FIX_RESET_LEADING_SPACE_RE: Regex = RegexBuilder::new(r"\s+([&§]r)\s+")
        .case_insensitive(true).build().unwrap();

    static ref FIX_RESET_MERGED_RE: Regex = RegexBuilder::new(r"([^\s&§])([&§]r)([^\s&§])")
        .case_insensitive(true).build().unwrap();

    static ref FIX_VAR_BRACKETS_SQUARE_RE: Regex = Regex::new(r"\[\s+(%\d*\$?[sd])\s+\]").unwrap();
    static ref FIX_VAR_BRACKETS_PAREN_RE: Regex = Regex::new(r"\(\s+(%\d*\$?[sd])\s+\)").unwrap();
    static ref FIX_VAR_BRACKETS_QUOTE_RE: Regex = Regex::new(r#""\s+(%\d*\$?[sd])\s+""#).unwrap();

    static ref FIX_PERCENT_SIMPLE_RE: Regex = Regex::new(r"%\s+([sd])").unwrap();
    static ref FIX_PERCENT_INDEXED_RE: Regex = Regex::new(r"%\s+(\d+)\s*\$\s*([sd])").unwrap();
    static ref FIX_PERCENT_FLOAT_RE: Regex = Regex::new(r"%\s*\.\s*(\d+)\s*([fd])").unwrap();

    static ref FIX_MD_LINK_RE: Regex = Regex::new(r"\]\s+\(").unwrap();
    static ref FIX_MD_IMG_RE: Regex = Regex::new(r"!\s+\[").unwrap();
    static ref FIX_BRACKET_OPEN_RE: Regex = Regex::new(r"\[\s+").unwrap();
    static ref FIX_BRACKET_CLOSE_RE: Regex = Regex::new(r"\s+\]").unwrap();
    static ref FIX_MULTI_SPACE_RE: Regex = Regex::new(r" {2,}").unwrap();

    static ref FIX_BROKEN_NEWLINE_RE: Regex = Regex::new(r"(?:\\+\s*n|/\s*n|\\+n)").unwrap();
}

pub fn polish_translation(text: &str, dictionary: &HashMap<String, String>) -> String {
    if text.is_empty() {
        return text.to_string();
    }

    let mut out = text.to_string();

    // Fix broken newline artifacts
    out = FIX_BROKEN_NEWLINE_RE.replace_all(&out, "\n").to_string();

    // Color code fixes
    out = FIX_COLOR_INNER_SPACE_RE.replace_all(&out, "$1$2").to_string();
    out = FIX_COLOR_TRAILING_SPACE_RE.replace_all(&out, "$1").to_string();
    out = FIX_RESET_LEADING_SPACE_RE.replace_all(&out, "$1 ").to_string();
    out = FIX_RESET_MERGED_RE.replace_all(&out, "$1$2 $3").to_string();

    // Variable fixes
    out = FIX_VAR_BRACKETS_SQUARE_RE.replace_all(&out, "[$1]").to_string();
    out = FIX_VAR_BRACKETS_PAREN_RE.replace_all(&out, "($1)").to_string();
    out = FIX_VAR_BRACKETS_QUOTE_RE.replace_all(&out, "\"$1\"").to_string();
    out = FIX_PERCENT_SIMPLE_RE.replace_all(&out, "%$1").to_string();
    out = FIX_PERCENT_INDEXED_RE.replace_all(&out, "%$1$$2").to_string();
    out = FIX_PERCENT_FLOAT_RE.replace_all(&out, "%.$1$2").to_string();

    // Markdown and brackets fixes
    out = FIX_MD_LINK_RE.replace_all(&out, "](").to_string();
    out = FIX_MD_IMG_RE.replace_all(&out, "![").to_string();
    out = FIX_BRACKET_OPEN_RE.replace_all(&out, "[").to_string();
    out = FIX_BRACKET_CLOSE_RE.replace_all(&out, "]").to_string();
    out = FIX_MULTI_SPACE_RE.replace_all(&out, " ").to_string();

    // Fast dictionary replacements: only regex-replace if substring is present!
    let lower_out = out.to_lowercase();
    for (wrong, right) in dictionary {
        let wrong_lower = wrong.to_lowercase();
        if lower_out.contains(&wrong_lower) {
            if let Ok(word_re) = RegexBuilder::new(&format!(r"\b{}\b", regex::escape(wrong)))
                .case_insensitive(true)
                .build()
            {
                out = word_re.replace_all(&out, |caps: &regex::Captures| {
                    let matched = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                    match_case(matched, right)
                }).to_string();
            }
        }
    }

    out
}

fn match_case(original: &str, replacement: &str) -> String {
    if original.is_empty() || replacement.is_empty() {
        return replacement.to_string();
    }
    
    // Check if ALL UPPERCASE
    if original.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) {
        return replacement.to_uppercase();
    }
    
    // Check if Titlecase (First char uppercase)
    let first_orig = original.chars().next().unwrap();
    if first_orig.is_uppercase() {
        let mut chars = replacement.chars();
        let first = chars.next().unwrap().to_uppercase().to_string();
        return format!("{}{}", first, chars.as_str());
    }

    replacement.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_and_reset_cleaning() {
        let dict = HashMap::new();
        
        let text = "§ aТест § 6золото §r и продолжение";
        let polished = polish_translation(text, &dict);
        assert_eq!(polished, "§aТест §6золото§r и продолжение");

        let text2 = "Уровень&rи опыт";
        let polished2 = polish_translation(text2, &dict);
        assert_eq!(polished2, "Уровень&r и опыт");
    }

    #[test]
    fn test_variable_bracket_cleaning() {
        let dict = HashMap::new();
        let text = "Количество: [ %s ] или ( %1$s ) или % .2f шт";
        let polished = polish_translation(text, &dict);
        assert_eq!(polished, "Количество: [%s] или (%1$s) или %.2f шт");
    }

    #[test]
    fn test_dictionary_case_replacement() {
        let mut dict = HashMap::new();
        dict.insert("полуслой".to_string(), "плита".to_string());

        assert_eq!(polish_translation("Деревянный Полуслой", &dict), "Деревянный Плита");
        assert_eq!(polish_translation("ПОЛУСЛОЙ ИЗ КАМНЯ", &dict), "ПЛИТА ИЗ КАМНЯ");
        assert_eq!(polish_translation("гладкий полуслой", &dict), "гладкий плита");
    }
}

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub const SETTINGS_FILE: &str = "settings.json";
pub const CACHE_FILE_STD: &str = "cache.json";
pub const CACHE_FILE_AI: &str = "ai_cache.json";
pub const DICT_FILE: &str = "dictionary.json";

pub const OPENROUTER_API: &str = "https://openrouter.ai/api/v1/chat/completions";
pub const DEFAULT_OPENROUTER_MODEL: &str = "google/gemma-2-9b-it:free";
pub const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";
pub const DEFAULT_GEMINI_MODEL: &str = "gemini-3.5-flash-lite";
pub const OPENCODE_GO_API: &str = "https://opencode.ai/zen/go/v1/chat/completions";
pub const DEFAULT_OPENCODE_MODEL: &str = "deepseek-v4-flash";

pub const KEYS_TO_TRANSLATE: &[&str] = &[
    "name", "title", "text", "description", "subtitle", "label", "hover_text", "link_text", "desc"
];

pub const BOOK_PATH_MARKERS: &[&str] = &["patchouli", "lexicon", "guide"];
pub const MD_PATH_MARKERS: &[&str] = &["/en_us/", "/ae2guide/", "/guide/", "/manual/", "/lexicon/"];
pub const RESEARCH_PATH_MARKERS: &[&str] = &["/research/", "/researches/", "/quests/"];

pub const LOOSE_JSON_SEARCH_DIRS: &[&str] = &[
    "kubejs/assets",
    "defaultconfigs",
    "config/ftbquests/lang",
    "minecraft/resources",
    "minecraft/openloader/resources",
    "resources",
    "openloader/resources",
    "config/txloader/load",
    "config/txloader/forceload",
    "config/txloader",
    "",
];

pub const IGNORE_TERMS: &[&str] = &[
    "mB/tick", "RF/t", "FE/t", "mB/t", "XP/t", "kRF", "ticks", "NBT", "FPS", "TPS",
    "JEI", "REI", "EMI", "API", "JSON", "XYZ", "RGB", "GUI", "HUD", "VIII", "VII",
    "III", "XII", "mB", "RF", "FE", "EU", "AE", "kW", "HP", "XP", "MP", "ID",
    "II", "IV", "VI", "IX", "XI", "UI", "J"
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub label: String,
    pub file_code: String,
    pub api_code: String,
    pub deepl_code: String,
    pub name: String,
    pub regex_pattern: String,
}

pub fn get_languages() -> Vec<LanguageInfo> {
    vec![
        LanguageInfo {
            label: "Русский".to_string(),
            file_code: "ru_ru".to_string(),
            api_code: "ru".to_string(),
            deepl_code: "RU".to_string(),
            name: "Russian".to_string(),
            regex_pattern: r"[А-Яа-яЁё]".to_string(),
        },
        LanguageInfo {
            label: "Українська".to_string(),
            file_code: "uk_ua".to_string(),
            api_code: "uk".to_string(),
            deepl_code: "UK".to_string(),
            name: "Ukrainian".to_string(),
            regex_pattern: r"[а-щьюяєіїґА-ЩЬЮЯЄІЇҐ]".to_string(),
        },
        LanguageInfo {
            label: "English (UK)".to_string(),
            file_code: "en_gb".to_string(),
            api_code: "en".to_string(),
            deepl_code: "EN-GB".to_string(),
            name: "English".to_string(),
            regex_pattern: r"[a-zA-Z]".to_string(),
        },
        LanguageInfo {
            label: "Deutsch".to_string(),
            file_code: "de_de".to_string(),
            api_code: "de".to_string(),
            deepl_code: "DE".to_string(),
            name: "German".to_string(),
            regex_pattern: r"[äöüßÄÖÜẞ]".to_string(),
        },
        LanguageInfo {
            label: "Español".to_string(),
            file_code: "es_es".to_string(),
            api_code: "es".to_string(),
            deepl_code: "ES".to_string(),
            name: "Spanish".to_string(),
            regex_pattern: r"[áéíóúüñÁÉÍÓÚÜÑ]".to_string(),
        },
        LanguageInfo {
            label: "Français".to_string(),
            file_code: "fr_fr".to_string(),
            api_code: "fr".to_string(),
            deepl_code: "FR".to_string(),
            name: "French".to_string(),
            regex_pattern: r"[àâæçéèêëîïôœùûüÿÀÂÆÇÉÈÊËÎÏÔŒÙÛÜŸ]".to_string(),
        },
        LanguageInfo {
            label: "Italiano".to_string(),
            file_code: "it_it".to_string(),
            api_code: "it".to_string(),
            deepl_code: "IT".to_string(),
            name: "Italian".to_string(),
            regex_pattern: r"[àèéìíîòóùúÀÈÉÌÍÎÒÓÙÚ]".to_string(),
        },
        LanguageInfo {
            label: "Polski".to_string(),
            file_code: "pl_pl".to_string(),
            api_code: "pl".to_string(),
            deepl_code: "PL".to_string(),
            name: "Polish".to_string(),
            regex_pattern: r"[ąćęłńóśźżĄĆĘŁŃÓŚŹŻ]".to_string(),
        },
        LanguageInfo {
            label: "Čeština".to_string(),
            file_code: "cs_cz".to_string(),
            api_code: "cs".to_string(),
            deepl_code: "CS".to_string(),
            name: "Czech".to_string(),
            regex_pattern: r"[áčďéěíňóřšťúůýžÁČĎÉĚÍŇÓŘŠŤÚŮÝŽ]".to_string(),
        },
        LanguageInfo {
            label: "Türkçe".to_string(),
            file_code: "tr_tr".to_string(),
            api_code: "tr".to_string(),
            deepl_code: "TR".to_string(),
            name: "Turkish".to_string(),
            regex_pattern: r"[çğıöşüÇĞİÖŞÜ]".to_string(),
        },
        LanguageInfo {
            label: "Português".to_string(),
            file_code: "pt_br".to_string(),
            api_code: "pt".to_string(),
            deepl_code: "PT-BR".to_string(),
            name: "Portuguese".to_string(),
            regex_pattern: r"[ãõáéíóúâêôÃÕÁÉÍÓÚÂÊÔ]".to_string(),
        },
        LanguageInfo {
            label: "中文 (简体)".to_string(),
            file_code: "zh_cn".to_string(),
            api_code: "zh-CN".to_string(),
            deepl_code: "ZH".to_string(),
            name: "Simplified Chinese".to_string(),
            regex_pattern: r"[\u4e00-\u9fff]".to_string(),
        },
        LanguageInfo {
            label: "日本語".to_string(),
            file_code: "ja_jp".to_string(),
            api_code: "ja".to_string(),
            deepl_code: "JA".to_string(),
            name: "Japanese".to_string(),
            regex_pattern: r"[\u3040-\u30ff\u4e00-\u9fff]".to_string(),
        },
        LanguageInfo {
            label: "한국어".to_string(),
            file_code: "ko_kr".to_string(),
            api_code: "ko".to_string(),
            deepl_code: "KO".to_string(),
            name: "Korean".to_string(),
            regex_pattern: r"[\uac00-\ud7af]".to_string(),
        },
        // --- СНГ ---
        LanguageInfo {
            label: "Беларуская".to_string(),
            file_code: "be_by".to_string(),
            api_code: "be".to_string(),
            deepl_code: "".to_string(),
            name: "Belarusian".to_string(),
            regex_pattern: r"[А-Яа-яЁёЎўІі]".to_string(),
        },
        LanguageInfo {
            label: "Қазақша".to_string(),
            file_code: "kk_kz".to_string(),
            api_code: "kk".to_string(),
            deepl_code: "".to_string(),
            name: "Kazakh".to_string(),
            regex_pattern: r"[А-Яа-яЁёҒғҚқҢңӨөҰұҮүҺһІі]".to_string(),
        },
        LanguageInfo {
            label: "Ўзбекча".to_string(),
            file_code: "uz_uz".to_string(),
            api_code: "uz".to_string(),
            deepl_code: "".to_string(),
            name: "Uzbek".to_string(),
            regex_pattern: r"[a-zA-ZʻʼGgOoUu]".to_string(),
        },
        LanguageInfo {
            label: "Azərbaycan".to_string(),
            file_code: "az_az".to_string(),
            api_code: "az".to_string(),
            deepl_code: "".to_string(),
            name: "Azerbaijani".to_string(),
            regex_pattern: r"[ÇçƏəĞğİıÖöŞşÜü]".to_string(),
        },
        LanguageInfo {
            label: "Српски".to_string(),
            file_code: "sr_rs".to_string(),
            api_code: "sr".to_string(),
            deepl_code: "".to_string(),
            name: "Serbian".to_string(),
            regex_pattern: r"[А-Яа-яЂђЉљЊњЋћЏџ]".to_string(),
        },
        // --- Европа ---
        LanguageInfo {
            label: "Română".to_string(),
            file_code: "ro_ro".to_string(),
            api_code: "ro".to_string(),
            deepl_code: "RO".to_string(),
            name: "Romanian".to_string(),
            regex_pattern: r"[ăâîșțĂÂÎȘȚ]".to_string(),
        },
        LanguageInfo {
            label: "Български".to_string(),
            file_code: "bg_bg".to_string(),
            api_code: "bg".to_string(),
            deepl_code: "BG".to_string(),
            name: "Bulgarian".to_string(),
            regex_pattern: r"[А-Яа-яЁё]".to_string(),
        },
        LanguageInfo {
            label: "Magyar".to_string(),
            file_code: "hu_hu".to_string(),
            api_code: "hu".to_string(),
            deepl_code: "HU".to_string(),
            name: "Hungarian".to_string(),
            regex_pattern: r"[áéíóöőúüűÁÉÍÓÖŐÚÜŰ]".to_string(),
        },
        LanguageInfo {
            label: "Slovenčina".to_string(),
            file_code: "sk_sk".to_string(),
            api_code: "sk".to_string(),
            deepl_code: "SK".to_string(),
            name: "Slovak".to_string(),
            regex_pattern: r"[áäčďéíĺľňóôŕšťúýžÁÄČĎÉÍĹĽŇÓÔŔŠŤÚÝŽ]".to_string(),
        },
        LanguageInfo {
            label: "Nederlands".to_string(),
            file_code: "nl_nl".to_string(),
            api_code: "nl".to_string(),
            deepl_code: "NL".to_string(),
            name: "Dutch".to_string(),
            regex_pattern: r"[a-zA-Z]".to_string(),
        },
        LanguageInfo {
            label: "Svenska".to_string(),
            file_code: "sv_se".to_string(),
            api_code: "sv".to_string(),
            deepl_code: "SV".to_string(),
            name: "Swedish".to_string(),
            regex_pattern: r"[åäöÅÄÖ]".to_string(),
        },
        LanguageInfo {
            label: "Norsk".to_string(),
            file_code: "no_no".to_string(),
            api_code: "no".to_string(),
            deepl_code: "NB".to_string(),
            name: "Norwegian".to_string(),
            regex_pattern: r"[æøåÆØÅ]".to_string(),
        },
        LanguageInfo {
            label: "Dansk".to_string(),
            file_code: "da_dk".to_string(),
            api_code: "da".to_string(),
            deepl_code: "DA".to_string(),
            name: "Danish".to_string(),
            regex_pattern: r"[æøåÆØÅ]".to_string(),
        },
    ]
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PackFormat {
    pub rp: u32,
    pub dp: u32,
}

pub fn get_pack_formats() -> HashMap<String, PackFormat> {
    let mut map = HashMap::new();
    map.insert("1.21.1".to_string(), PackFormat { rp: 34, dp: 48 });
    map.insert("1.20.4".to_string(), PackFormat { rp: 22, dp: 26 });
    map.insert("1.20.1".to_string(), PackFormat { rp: 15, dp: 15 });
    map.insert("1.19.4".to_string(), PackFormat { rp: 13, dp: 12 });
    map.insert("1.19.2".to_string(), PackFormat { rp: 9, dp: 10 });
    map.insert("1.18.2".to_string(), PackFormat { rp: 8, dp: 9 });
    map.insert("1.16.5".to_string(), PackFormat { rp: 6, dp: 6 });
    map.insert("1.12.2".to_string(), PackFormat { rp: 3, dp: 1 });
    map.insert("1.7.10".to_string(), PackFormat { rp: 1, dp: 1 });
    map
}

pub fn get_default_dictionary() -> HashMap<String, String> {
    let mut dict = HashMap::new();
    dict.insert("полуслой".to_string(), "плита".to_string());
    dict.insert("слиток меди".to_string(), "медный слиток".to_string());
    dict.insert("слиток олова".to_string(), "оловянный слиток".to_string());
    dict.insert("слиток бронзы".to_string(), "бронзовый слиток".to_string());
    dict.insert("слиток стали".to_string(), "стальной слиток".to_string());
    dict.insert("слиток серебра".to_string(), "серебряный слиток".to_string());
    dict.insert("слиток свинца".to_string(), "свинцовый слиток".to_string());
    dict.insert("слиток никеля".to_string(), "никелевый слиток".to_string());
    dict.insert("слиток платины".to_string(), "платиновый слиток".to_string());
    dict.insert("слиток иридия".to_string(), "иридиевый слиток".to_string());
    dict.insert("слиток титана".to_string(), "титановый слиток".to_string());
    dict.insert("слиток вольфрама".to_string(), "вольфрамовый слиток".to_string());
    dict.insert("слиток хрома".to_string(), "хромовый слиток".to_string());
    dict.insert("слиток алюминия".to_string(), "алюминиевый слиток".to_string());
    dict.insert("слиток цинка".to_string(), "цинковый слиток".to_string());
    dict.insert("слиток латуни".to_string(), "латунный слиток".to_string());
    dict.insert("слиток электрума".to_string(), "электрумовый слиток".to_string());
    dict.insert("слиток инвара".to_string(), "инваровый слиток".to_string());
    dict.insert("пыль меди".to_string(), "медная пыль".to_string());
    dict.insert("пыль олова".to_string(), "оловянная пыль".to_string());
    dict.insert("пыль бронзы".to_string(), "бронзовая пыль".to_string());
    dict.insert("пыль железа".to_string(), "железная пыль".to_string());
    dict.insert("пыль золота".to_string(), "золотая пыль".to_string());
    dict.insert("пыль алмаза".to_string(), "алмазная пыль".to_string());
    dict.insert("шестерня меди".to_string(), "медная шестерня".to_string());
    dict.insert("шестерня железа".to_string(), "железная шестерня".to_string());
    dict.insert("шестерня золота".to_string(), "золотая шестерня".to_string());
    dict.insert("пластина меди".to_string(), "медная пластина".to_string());
    dict.insert("пластина железа".to_string(), "железная пластина".to_string());
    dict.insert("пластина золота".to_string(), "золотая пластина".to_string());
    dict
}

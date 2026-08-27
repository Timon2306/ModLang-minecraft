export interface LanguageInfo {
  label: string;
  file_code: string;
  api_code: string;
  deepl_code: string;
  name: string;
  regex_pattern: string;
}

export interface PackFormat {
  rp: number;
  dp: number;
}

export interface AppConfig {
  mc_dir: string;
  theme: string;
  smart_glue: boolean;
  google_workers: number;
  google_delay_ms: number;
  
  // Gemini Settings
  gemini_api_key: string;
  gemini_model: string;
  gemini_delay_ms: number;

  // OpenAI / OpenRouter Settings
  openrouter_api_url: string;
  openrouter_api_key: string;
  openrouter_model: string;
  openrouter_site_url: string;
  openrouter_app_name: string;

  // DeepL Settings
  deepl_key: string;

  // OpenCode Go Settings
  opencode_api_key: string;
  opencode_model: string;

  // Translation Preferences
  target_language: string;
  mc_version: string;
  output_mode: string;
  pack_name: string;
  engine: string;
  google_mode: string;
  ai_mode: string;
  process_mode: string;
  translate_mods: boolean;
  translate_books: boolean;
  translate_quests: boolean;
}

export interface AnalysisRow {
  icon: string;
  name: string;
  kind: string;
  translated_count: number;
  total_count: number;
  percent: number;
}

export interface AnalysisResult {
  rows: AnalysisRow[];
  total_translated: number;
  total_strings: number;
  overall_percent: number;
}

export interface ProgressEvent {
  current: number;
  total: number;
  percent: number;
  eta: string;
  rate: number;
}

export interface LogEvent {
  message: string;
  color: string;
}

export interface StatusEvent {
  text: string;
  progress: number | null;
}

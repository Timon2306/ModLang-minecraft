# ModLang — Minecraft Modpack Translator

High-performance desktop tool for localizing Minecraft mods, quests, and guidebooks into any language.

Built with **Tauri v2**, **Rust**, and **React**.

---

## Features

- **Broad format support:**
  - Mods (`.jar`): reads `assets/<mod_id>/lang/en_us.json` and legacy `en_us.lang` (MC 1.12.2 and below)
  - Quests: FTB Quests (`.snbt`), Better Questing (`DefaultQuests.json`)
  - Books: Patchouli (JSON + Markdown), Guide-API
  - Scripts: KubeJS, OpenLoader, Resource Loader configs
- **Sanitization Engine** — protects variables (`%s`, `{0}`), color codes (`§a`, `&c`), formatting tags (`$(br)`), and unit strings (`RF/t`, `FE`, `mB`) from being corrupted during translation
- **Multiple translation backends:**
  - Google Gemini (`gemini-2.5-flash`) with Minecraft-aware context prompt
  - OpenAI / OpenRouter — compatible with LM Studio, Ollama, KoboldCPP
  - Google Translate (web) — multithreaded with exponential backoff
  - DeepL — Free and Pro tiers
- **Output:** generates valid **Resource Pack** and **Datapack** `.zip` files with correct `pack.mcmeta` for Minecraft 1.7.10 through 1.21.1+. Source `.jar` files are never modified.
- **Translation cache** — already-translated strings are stored and reused, dramatically speeding up repeated runs across modpacks that share mods.
- **Minecraft-styled UI** — dark inventory-slot inputs, 3D-bevel buttons, XP bar progress indicator, in-game chat console log.

---

## Installation

Download the latest release from the [Releases](https://github.com/Timon2306/modlang/releases) page:

| File | Description |
| :--- | :--- |
| `ModLang_x.x.x_x64-setup.exe` | NSIS installer (recommended) |
| `ModLang_x.x.x_x64_en-US.msi` | MSI installer |
| `modlang_cache_export.7z` | Pre-built translation cache (see below) |

**Requirements:** Windows 10/11 x64. No additional runtime needed.

---

## Pre-built Translation Cache

`modlang_cache_export.7z` contains a community-built cache of pre-translated strings from hundreds of popular Minecraft mods. Strings like item names, tooltips, and UI labels repeat across modpacks — with this cache, ModLang can skip re-translating them and just look up the result instantly.

**To use:**
1. Extract `modlang_cache_export.7z`
2. Place `cache.json` next to `settings.json` in the app directory (or wherever your working directory is)
3. Run ModLang — it will automatically load and extend the cache

---

## Configuration

Copy `settings.example.json` to `settings.json` and fill in your values:

```json
{
  "mc_dir": "C:\\path\\to\\your\\minecraft\\instance",
  "gemini_api_key": "YOUR_GEMINI_API_KEY",
  "deepl_key": "YOUR_DEEPL_KEY",
  "engine": "google",
  "target_language": "Русский",
  "mc_version": "1.7.10"
}
```

> `settings.json` is listed in `.gitignore` and will never be committed — it stays local only.

**Getting API keys:**
- Gemini: [aistudio.google.com](https://aistudio.google.com) — free tier available
- DeepL: [deepl.com/pro-api](https://www.deepl.com/pro-api) — free tier: 500K chars/month
- OpenRouter: [openrouter.ai](https://openrouter.ai) — many free models available

---

## Building from Source

**Requirements:**
- [Node.js](https://nodejs.org/) v18+
- [Rust & Cargo](https://rustup.rs/) 1.75+

```bash
# Install frontend dependencies
npm install

# Development mode
npm run tauri dev

# Production build
npm run tauri build
```

---

## Tech Stack

| Layer | Technology |
| :--- | :--- |
| Desktop framework | Tauri v2 |
| Backend | Rust 2021 — Tokio, Rayon, Reqwest, Serde, Zip |
| Frontend | React 18, TypeScript, Tailwind CSS, Vite |
| Cache | Thread-safe JSON |

---

## License

MIT

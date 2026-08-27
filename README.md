# ModLang — Переводчик модпаков Minecraft

Высокопроизводительный десктопный инструмент для локализации модов, квестов и справочников Minecraft на любой язык.

Написан на **Tauri v2**, **Rust** и **React**.

---

## Возможности

- **Поддержка форматов:**
  - Моды (`.jar`): читает `assets/<mod_id>/lang/en_us.json` и legacy `en_us.lang` (MC 1.12.2 и ниже)
  - Квесты: FTB Quests (`.snbt`), Better Questing (`DefaultQuests.json`)
  - Книги и мануалы: Patchouli (JSON + Markdown), Guide-API
  - Скрипты и конфиги: KubeJS, OpenLoader, Resource Loader
- **Sanitization Engine** — защищает переменные (`%s`, `{0}`), цветовые коды (`§a`, `&c`), теги форматирования (`$(br)`) и единицы измерения (`RF/t`, `FE`, `mB`) от искажения при переводе
- **Несколько движков перевода:**
  - Google Gemini (`gemini-2.5-flash`) с Minecraft-специфичным контекстным промптом
  - OpenAI / OpenRouter — совместим с LM Studio, Ollama, KoboldCPP
  - Google Translate (веб) — многопоточный режим с защитой от блокировок
  - DeepL — Free и Pro тарифы
- **Результат:** генерирует валидные **Resource Pack** и **Datapack** `.zip` с корректным `pack.mcmeta` для Minecraft 1.7.10 — 1.21.1+. Исходные `.jar` файлы не изменяются.
- **Кэш переводов** — уже переведённые строки сохраняются и переиспользуются, что значительно ускоряет повторный перевод модпаков с пересекающимися модами.
- **Интерфейс в стиле Minecraft** — тёмные слоты инвентаря, 3D-кнопки, шкала опыта (XP bar) для отображения прогресса, консоль в виде игрового чата.

---

## Установка

Скачай последний релиз со страницы [Releases](https://github.com/Timon2306/modlang-modpack-translator/releases):

| Файл | Описание |
| :--- | :--- |
| `ModLang_x.x.x_x64-setup.exe` | NSIS установщик (рекомендуется) |
| `ModLang_x.x.x_x64_en-US.msi` | MSI установщик |
| `modlang_cache_export.7z` | Готовый кэш переводов (см. ниже) |

**Требования:** Windows 10/11 x64. Дополнительные зависимости не нужны.

---

## Готовый кэш переводов

`modlang_cache_export.7z` содержит кэш уже переведённых строк из сотен популярных Minecraft-модов. Названия предметов, подсказки, интерфейс — они повторяются от модпака к модпаку, и с этим кэшем ModLang просто берёт готовый результат вместо повторного перевода.

**Как использовать:**
1. Распакуй `modlang_cache_export.7z`
2. Открой вкладку **«Глоссарий и Кэш»** в приложении
3. Нажми **«Импорт кэша из JSON»** и выбери распакованный `cache.json`

---

## Настройка

Скопируй `settings.example.json` из репозитория в `settings.json` и заполни нужные поля:

```json
{
  "mc_dir": "C:\\путь\\до\\твоего\\minecraft\\инстанса",
  "gemini_api_key": "ВАШ_КЛЮЧ_GEMINI",
  "deepl_key": "ВАШ_КЛЮЧ_DEEPL",
  "engine": "google",
  "target_language": "Русский",
  "mc_version": "1.7.10"
}
```

> `settings.json` добавлен в `.gitignore` и никогда не попадёт в репозиторий — он остаётся только локально.

**Где получить API ключи:**
- Gemini: [aistudio.google.com](https://aistudio.google.com) — есть бесплатный лимит
- DeepL: [deepl.com/pro-api](https://www.deepl.com/pro-api) — Free: 500 000 символов в месяц
- OpenRouter: [openrouter.ai](https://openrouter.ai) — много бесплатных моделей

---

## Сборка из исходников

**Требования:**
- [Node.js](https://nodejs.org/) v18+
- [Rust и Cargo](https://rustup.rs/) v1.75+

```bash
# Установить зависимости фронтенда
npm install

# Запуск в режиме разработки
npm run tauri dev

# Сборка релизной версии
npm run tauri build
```

---

## Стек технологий

| Слой | Технологии |
| :--- | :--- |
| Десктопный фреймворк | Tauri v2 |
| Бэкенд | Rust 2021 — Tokio, Rayon, Reqwest, Serde, Zip |
| Фронтенд | React 18, TypeScript, Tailwind CSS, Vite |
| Кэш | Потокобезопасный JSON |

---

## Лицензия

MIT — подробнее в файле [LICENSE](LICENSE).

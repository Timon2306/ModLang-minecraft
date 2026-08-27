import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { FolderOpen, Play, Pause, Square, BarChart3 } from 'lucide-react';
import { McButton } from '../common/McButton';
import { McInput } from '../common/McInput';
import { McSelect } from '../common/McSelect';
import { McCheckbox } from '../common/McCheckbox';
import { McXpBar } from '../common/McXpBar';
import { McChatConsole } from '../console/McChatConsole';
import { AppConfig, LanguageInfo, LogEvent, ProgressEvent } from '../../types';

interface TranslateScreenProps {
  config: AppConfig;
  languages: LanguageInfo[];
  isRunning: boolean;
  isPaused: boolean;
  progress: ProgressEvent;
  logs: LogEvent[];
  onConfigChange: (config: AppConfig) => void;
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onStop: () => void;
  onClearLogs: () => void;
  onGoToAnalysis: () => void;
}

export const TranslateScreen: React.FC<TranslateScreenProps> = ({
  config,
  languages,
  isRunning,
  isPaused,
  progress,
  logs,
  onConfigChange,
  onStart,
  onPause,
  onResume,
  onStop,
  onClearLogs,
  onGoToAnalysis,
}) => {
  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Выберите папку Minecraft (инстанс со сборкой)',
      });
      if (selected && typeof selected === 'string') {
        const updated = { ...config, mc_dir: selected };
        onConfigChange(updated);
        await invoke('save_settings_cmd', { newConfig: updated });
      }
    } catch (e) {
      console.error(e);
    }
  };

  const updateField = async <K extends keyof AppConfig>(key: K, value: AppConfig[K]) => {
    const updated = { ...config, [key]: value };
    onConfigChange(updated);
    try {
      await invoke('save_settings_cmd', { newConfig: updated });
    } catch (e) {
      console.error(e);
    }
  };

  const mcVersions = [
    { value: '1.21.1', label: '1.21.1 / 1.21.x (Format 34)' },
    { value: '1.20.6', label: '1.20.6 / 1.20.5 (Format 32)' },
    { value: '1.20.4', label: '1.20.4 / 1.20.3 (Format 22)' },
    { value: '1.20.1', label: '1.20.1 / 1.20.x (Format 15)' },
    { value: '1.19.4', label: '1.19.4 (Format 13)' },
    { value: '1.19.2', label: '1.19.2 (Format 9)' },
    { value: '1.18.2', label: '1.18.2 (Format 8)' },
    { value: '1.16.5', label: '1.16.5 (Format 6)' },
    { value: '1.12.2', label: '1.12.2 (Legacy .lang / Format 3)' },
    { value: '1.7.10', label: '1.7.10 (Legacy .lang / Format 1)' },
  ];

  const engineOptions = [
    { value: 'google', label: 'Google Translate (Web API)' },
    { value: 'gemini', label: `Google Gemini API (${config.gemini_model || 'gemini-3.5-flash-lite'})` },
    { value: 'opencode', label: `OpenCode Go (${config.opencode_model || 'deepseek-v4-flash'})` },
    { value: 'openai', label: 'OpenAI / OpenRouter / Local LLM' },
    { value: 'deepl', label: 'DeepL API' },
  ];

  return (
    <div className="grid grid-cols-12 gap-4 h-[calc(100vh-120px)] overflow-hidden">
      {/* Left Settings Sidebar */}
      <div className="col-span-5 flex flex-col gap-3 overflow-y-auto pr-1">
        {/* Instance Path Card */}
        <div className="mc-panel p-3 flex flex-col gap-2">
          <div className="flex justify-between items-center">
            <span className="text-xs font-pixel text-mc-gold uppercase font-bold">Папка Minecraft сборки</span>
          </div>
          <div className="flex gap-2 items-center">
            <div className="mc-slot px-3 py-2 text-xs font-mono text-zinc-300 truncate flex-1" title={config.mc_dir}>
              {config.mc_dir || 'Папка не выбрана'}
            </div>
            <McButton size="sm" onClick={handleSelectFolder} disabled={isRunning} icon={<FolderOpen className="w-4 h-4" />}>
              Выбрать
            </McButton>
          </div>
        </div>

        {/* Translation Target & Version */}
        <div className="mc-panel p-3 grid grid-cols-2 gap-2">
          <McSelect
            label="Целевой язык"
            options={languages.map((l) => ({ value: l.label, label: l.label }))}
            value={config.target_language}
            onChange={(e) => updateField('target_language', e.target.value)}
            disabled={isRunning}
          />
          <McSelect
            label="Версия игры"
            options={mcVersions}
            value={config.mc_version}
            onChange={(e) => updateField('mc_version', e.target.value)}
            disabled={isRunning}
          />
        </div>

        {/* Output Mode & Pack Name */}
        <div className="mc-panel p-3 flex flex-col gap-2">
          <div className="flex justify-between items-center">
            <span className="text-xs font-pixel text-mc-textMuted uppercase">Метод сохранения</span>
            {config.output_mode === 'resourcepack' && (
              <button
                onClick={() => invoke('open_resourcepack_folder_cmd')}
                className="text-[10px] font-mono text-mc-cyan hover:underline flex items-center gap-1 cursor-pointer"
                title="Открыть папку ресурспаков в проводнике"
              >
                <FolderOpen className="w-3 h-3" /> Папка паков
              </button>
            )}
          </div>
          <div className="grid grid-cols-2 gap-2">
            <label className="flex items-center gap-2 cursor-pointer text-xs font-mono">
              <input
                type="radio"
                name="output_mode"
                value="resourcepack"
                checked={config.output_mode === 'resourcepack'}
                onChange={() => updateField('output_mode', 'resourcepack')}
                disabled={isRunning}
                className="accent-mc-emerald"
              />
              <span>Resource Pack</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer text-xs font-mono">
              <input
                type="radio"
                name="output_mode"
                value="inplace"
                checked={config.output_mode === 'inplace'}
                onChange={() => updateField('output_mode', 'inplace')}
                disabled={isRunning}
                className="accent-mc-emerald"
              />
              <span>В файлы (.jar/.snbt)</span>
            </label>
          </div>
          {config.output_mode === 'resourcepack' && (
            <McInput
              label="Имя ресурспака"
              value={config.pack_name}
              onChange={(e) => updateField('pack_name', e.target.value)}
              placeholder="ModLang_Pack"
              disabled={isRunning}
            />
          )}
        </div>

        {/* Categories to Translate */}
        <div className="mc-panel p-3 flex flex-col gap-1.5">
          <span className="text-xs font-pixel text-mc-textMuted uppercase">Объекты перевода</span>
          <div className="grid grid-cols-1 gap-1">
            <McCheckbox
              label="Интерфейс модов (предметы, блоки, GUI)"
              checked={config.translate_mods}
              onChange={(v) => updateField('translate_mods', v)}
              disabled={isRunning}
            />
            <McCheckbox
              label="Справочники и книги (Patchouli, Guide-API, MD)"
              checked={config.translate_books}
              onChange={(v) => updateField('translate_books', v)}
              disabled={isRunning}
            />
            <McCheckbox
              label="Квесты (FTB Quests .snbt, Better Questing, Triumph)"
              checked={config.translate_quests}
              onChange={(v) => updateField('translate_quests', v)}
              disabled={isRunning}
            />
          </div>
        </div>

        {/* Engine and Processing Mode */}
        <div className="mc-panel p-3 flex flex-col gap-2">
          <McSelect
            label="Движок перевода"
            options={engineOptions}
            value={config.engine}
            onChange={(e) => updateField('engine', e.target.value)}
            disabled={isRunning}
          />
          {config.engine === 'google' && (
            <div className="flex gap-4 text-xs font-mono pt-1">
              <label className="flex items-center gap-1.5 cursor-pointer">
                <input
                  type="radio"
                  name="google_mode"
                  value="single"
                  checked={config.google_mode === 'single'}
                  onChange={() => updateField('google_mode', 'single')}
                  disabled={isRunning}
                  className="accent-mc-emerald"
                />
                <span>Построчно (точно)</span>
              </label>
              <label className="flex items-center gap-1.5 cursor-pointer">
                <input
                  type="radio"
                  name="google_mode"
                  value="batch"
                  checked={config.google_mode === 'batch'}
                  onChange={() => updateField('google_mode', 'batch')}
                  disabled={isRunning}
                  className="accent-mc-emerald"
                />
                <span>Пачками (быстро)</span>
              </label>
            </div>
          )}
          {config.engine === 'gemini' || config.engine === 'openai' || config.engine === 'opencode' ? (
            <div className="flex gap-4 text-xs font-mono pt-1">
              <label className="flex items-center gap-1.5 cursor-pointer">
                <input
                  type="radio"
                  name="ai_mode"
                  value="safe"
                  checked={config.ai_mode === 'safe'}
                  onChange={() => updateField('ai_mode', 'safe')}
                  disabled={isRunning}
                  className="accent-mc-emerald"
                />
                <span>Безопасный (20 строк)</span>
              </label>
              <label className="flex items-center gap-1.5 cursor-pointer">
                <input
                  type="radio"
                  name="ai_mode"
                  value="context"
                  checked={config.ai_mode === 'context'}
                  onChange={() => updateField('ai_mode', 'context')}
                  disabled={isRunning}
                  className="accent-mc-emerald"
                />
                <span>Лор + Контекст (40 строк)</span>
              </label>
            </div>
          ) : null}
          
          <div className="pt-2 border-t border-mc-borderDark">
            <span className="text-xs font-pixel text-mc-textMuted uppercase block mb-1">Режим обработки</span>
            <div className="flex justify-between text-xs font-mono">
              <label className="flex items-center gap-1 cursor-pointer">
                <input
                  type="radio"
                  name="process_mode"
                  value="append"
                  checked={config.process_mode === 'append'}
                  onChange={() => updateField('process_mode', 'append')}
                  disabled={isRunning}
                  className="accent-mc-emerald"
                />
                <span>Доперевод</span>
              </label>
              <label className="flex items-center gap-1 cursor-pointer">
                <input
                  type="radio"
                  name="process_mode"
                  value="skip"
                  checked={config.process_mode === 'skip'}
                  onChange={() => updateField('process_mode', 'skip')}
                  disabled={isRunning}
                  className="accent-mc-emerald"
                />
                <span>Пропуск &gt;90%</span>
              </label>
              <label className="flex items-center gap-1 cursor-pointer">
                <input
                  type="radio"
                  name="process_mode"
                  value="force"
                  checked={config.process_mode === 'force'}
                  onChange={() => updateField('process_mode', 'force')}
                  disabled={isRunning}
                  className="accent-mc-emerald"
                />
                <span>С нуля</span>
              </label>
            </div>
          </div>
        </div>

        {/* Action Controls */}
        <div className="flex flex-col gap-2 pt-1">
          <div className="grid grid-cols-2 gap-2">
            <McButton
              variant="gray"
              onClick={onGoToAnalysis}
              disabled={isRunning}
              icon={<BarChart3 className="w-4 h-4" />}
            >
              Анализ сборки
            </McButton>
            {!isRunning ? (
              <McButton
                variant="green"
                onClick={onStart}
                icon={<Play className="w-4 h-4" />}
              >
                НАЧАТЬ ПЕРЕВОД
              </McButton>
            ) : (
              <McButton
                variant="red"
                onClick={onStop}
                icon={<Square className="w-4 h-4" />}
              >
                ОСТАНОВИТЬ
              </McButton>
            )}
          </div>
          {isRunning && (
            <McButton
              variant={isPaused ? 'green' : 'gold'}
              onClick={isPaused ? onResume : onPause}
              icon={isPaused ? <Play className="w-4 h-4" /> : <Pause className="w-4 h-4" />}
            >
              {isPaused ? 'ВОЗОБНОВИТЬ' : 'ПАУЗА'}
            </McButton>
          )}
        </div>
      </div>

      {/* Right Chat Console & Progress */}
      <div className="col-span-7 flex flex-col gap-3 h-full overflow-hidden">
        {/* XP Progress Bar */}
        <div className="mc-panel p-3 flex-shrink-0">
          <McXpBar
            current={progress.current}
            total={progress.total}
            percent={progress.percent}
            eta={progress.eta}
            rate={progress.rate}
          />
        </div>

        {/* In-Game Chat Console */}
        <div className="flex-1 overflow-hidden">
          <McChatConsole logs={logs} onClear={onClearLogs} />
        </div>
      </div>
    </div>
  );
};

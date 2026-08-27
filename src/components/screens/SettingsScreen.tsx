import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Save, Check, Sparkles, Cpu, Zap, Settings } from 'lucide-react';
import { McButton } from '../common/McButton';
import { McInput } from '../common/McInput';
import { McCheckbox } from '../common/McCheckbox';
import { AppConfig } from '../../types';

interface SettingsScreenProps {
  config: AppConfig;
  onConfigChange: (config: AppConfig) => void;
}

export const SettingsScreen: React.FC<SettingsScreenProps> = ({
  config,
  onConfigChange,
}) => {
  const [localConfig, setLocalConfig] = useState<AppConfig>({ ...config });
  const [saved, setSaved] = useState(false);

  const handleSave = async () => {
    try {
      await invoke('save_settings_cmd', { newConfig: localConfig });
      onConfigChange(localConfig);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error(e);
    }
  };

  const update = <K extends keyof AppConfig>(key: K, value: AppConfig[K]) => {
    setLocalConfig((prev) => ({ ...prev, [key]: value }));
  };

  return (
    <div className="flex flex-col gap-4 h-[calc(100vh-120px)] overflow-y-auto pr-1">
      {/* Top Header with Save Button */}
      <div className="flex justify-between items-center mc-panel p-3">
        <div>
          <h2 className="text-sm font-pixel text-mc-gold uppercase font-bold">Настройки API и провайдеров перевода</h2>
          <p className="text-xs text-mc-textMuted font-mono">Конфигурация ключей доступа, моделей нейросетей и многопоточности</p>
        </div>
        <McButton
          variant="green"
          size="sm"
          onClick={handleSave}
          icon={saved ? <Check className="w-4 h-4 text-white" /> : <Save className="w-4 h-4" />}
        >
          {saved ? 'СОХРАНЕНО' : 'СОХРАНИТЬ НАСТРОЙКИ'}
        </McButton>
      </div>

      <div className="grid grid-cols-2 gap-4">
        {/* Gemini API Settings */}
        <div className="mc-panel p-4 flex flex-col gap-3">
          <div className="flex items-center justify-between border-b border-mc-borderDark pb-2">
            <div className="flex items-center gap-2">
              <Sparkles className="w-4 h-4 text-mc-emerald" />
              <span className="text-xs font-pixel text-mc-emerald uppercase font-bold">Google Gemini API</span>
            </div>
            <span className="text-xs text-zinc-500 font-mono">gemini.google.com</span>
          </div>
          <McInput
            label="Gemini API Key"
            type="password"
            placeholder="AIzaSy..."
            value={localConfig.gemini_api_key}
            onChange={(e) => update('gemini_api_key', e.target.value)}
          />
          <McInput
            label="ID модели Gemini"
            placeholder="gemini-3.5-flash-lite"
            value={localConfig.gemini_model}
            onChange={(e) => update('gemini_model', e.target.value)}
          />
          <McInput
            label="Задержка между запросами Gemini (мс)"
            type="number"
            placeholder="4100"
            value={localConfig.gemini_delay_ms || 4100}
            onChange={(e) => update('gemini_delay_ms', parseInt(e.target.value) || 4100)}
          />
          <p className="text-xs text-zinc-500 font-mono leading-relaxed">
            Задержка <strong>4100 мс</strong> (~14 запросов в минуту) идеально соблюдает лимит Free-тарифа (15 RPM) и предотвращает ошибки 429.
          </p>
        </div>

        {/* OpenAI / OpenRouter Settings */}
        <div className="mc-panel p-4 flex flex-col gap-3">
          <div className="flex items-center justify-between border-b border-mc-borderDark pb-2">
            <div className="flex items-center gap-2">
              <Cpu className="w-4 h-4 text-mc-diamond" />
              <span className="text-xs font-pixel text-mc-diamond uppercase font-bold">OpenAI / OpenRouter / Local LLM</span>
            </div>
            <span className="text-xs text-zinc-500 font-mono">openrouter.ai / localhost</span>
          </div>
          <McInput
            label="API URL (эндпоинт Chat Completions)"
            placeholder="https://openrouter.ai/api/v1/chat/completions"
            value={localConfig.openrouter_api_url}
            onChange={(e) => update('openrouter_api_url', e.target.value)}
          />
          <McInput
            label="API Key (для OpenRouter или OpenAI)"
            type="password"
            placeholder="sk-or-v1-..."
            value={localConfig.openrouter_api_key}
            onChange={(e) => update('openrouter_api_key', e.target.value)}
          />
          <McInput
            label="ID Модели (например google/gemma-2-9b-it:free или mistralai/mistral-7b-instruct)"
            placeholder="google/gemma-2-9b-it:free"
            value={localConfig.openrouter_model}
            onChange={(e) => update('openrouter_model', e.target.value)}
          />
        </div>

        {/* DeepL API Settings */}
        <div className="mc-panel p-4 flex flex-col gap-3">
          <div className="flex items-center justify-between border-b border-mc-borderDark pb-2">
            <div className="flex items-center gap-2">
              <Zap className="w-4 h-4 text-mc-purple" />
              <span className="text-xs font-pixel text-mc-purple uppercase font-bold">DeepL API</span>
            </div>
            <span className="text-xs text-zinc-500 font-mono">deepl.com</span>
          </div>
          <McInput
            label="DeepL API Key (Free или Pro)"
            type="password"
            placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx:fx"
            value={localConfig.deepl_key}
            onChange={(e) => update('deepl_key', e.target.value)}
          />
          <p className="text-xs text-zinc-500 font-mono">
            Автоматически определяет Free (с суффиксом <code>:fx</code>) или Pro эндпоинт.
          </p>
        </div>

        {/* OpenCode Go Settings */}
        <div className="mc-panel p-4 flex flex-col gap-3">
          <div className="flex items-center justify-between border-b border-mc-borderDark pb-2">
            <div className="flex items-center gap-2">
              <Sparkles className="w-4 h-4 text-mc-diamond" />
              <span className="text-xs font-pixel text-mc-diamond uppercase font-bold">OpenCode Go</span>
            </div>
            <span className="text-xs text-zinc-500 font-mono">$10/мес · opencode.ai/go</span>
          </div>
          <McInput
            label="OpenCode Go API Key"
            type="password"
            placeholder="sk-..."
            value={localConfig.opencode_api_key}
            onChange={(e) => update('opencode_api_key', e.target.value)}
          />
          <div className="flex flex-col gap-1">
            <span className="text-xs text-zinc-400 font-mono">Модель</span>
            <div className="grid grid-cols-3 gap-1.5">
              {[
                { id: 'deepseek-v4-flash', label: 'DeepSeek V4 Flash', hint: '7.6K/5ч' },
                { id: 'mimo-v2.5', label: 'MiMo-V2.5', hint: '30K/5ч' },
                { id: 'longcat-2.0', label: 'LongCat-2.0', hint: '11K/5ч' },
                { id: 'qwen3.7-plus', label: 'Qwen3.7 Plus', hint: '4.3K/5ч' },
                { id: 'gpt-5.6-luna', label: 'GPT 5.6 Luna', hint: '2K/5ч' },
                { id: 'glm-5.3-flash', label: 'GLM-5.3-Flash', hint: '1.6K/5ч' },
              ].map((m) => (
                <button
                  key={m.id}
                  onClick={() => update('opencode_model', m.id)}
                  className={`text-left px-2 py-1.5 rounded text-[10px] font-mono border transition-colors cursor-pointer ${
                    localConfig.opencode_model === m.id
                      ? 'border-mc-diamond bg-mc-diamond/10 text-mc-diamond'
                      : 'border-mc-borderDark text-zinc-400 hover:border-zinc-500'
                  }`}
                >
                  <div className="font-medium truncate">{m.label}</div>
                  <div className="text-zinc-500">{m.hint}</div>
                </button>
              ))}
            </div>
          </div>
          <McInput
            label="Или введите ID модели вручную"
            placeholder="deepseek-v4-flash"
            value={localConfig.opencode_model}
            onChange={(e) => update('opencode_model', e.target.value)}
          />
          <p className="text-xs text-zinc-500 font-mono leading-relaxed">
            Огромные лимиты запросов за <strong>$10/мес</strong>. DeepSeek V4 Flash — лучший баланс скорости и качества для перевода.
          </p>
        </div>

        {/* Google & General Settings */}
        <div className="mc-panel p-4 flex flex-col gap-3">
          <div className="flex items-center justify-between border-b border-mc-borderDark pb-2">
            <div className="flex items-center gap-2">
              <Settings className="w-4 h-4 text-mc-gold" />
              <span className="text-xs font-pixel text-mc-gold uppercase font-bold">Общие настройки и Google Web</span>
            </div>
            <span className="text-xs text-zinc-500 font-mono">Многопоточность</span>
          </div>
          <div className="grid grid-cols-2 gap-3">
            <McInput
              label="Потоков Google (1-10)"
              type="number"
              value={localConfig.google_workers}
              onChange={(e) => update('google_workers', parseInt(e.target.value) || 3)}
            />
            <McInput
              label="Задержка Google (мс)"
              type="number"
              value={localConfig.google_delay_ms}
              onChange={(e) => update('google_delay_ms', parseInt(e.target.value) || 0)}
            />
          </div>
          <div className="pt-2 border-t border-mc-borderDark flex flex-col gap-2">
            <McCheckbox
              label="Умная склейка переносов (Smart Glue) для книг"
              checked={localConfig.smart_glue}
              onChange={(v) => update('smart_glue', v)}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { Plus, Trash2, Download, Upload, Database, Save, Check } from 'lucide-react';
import { McButton } from '../common/McButton';
import { McInput } from '../common/McInput';

export const GlossaryScreen: React.FC = () => {
  const [dictionary, setDictionary] = useState<Record<string, string>>({});
  const [newWrong, setNewWrong] = useState('');
  const [newRight, setNewRight] = useState('');
  const [cacheCount, setCacheCount] = useState<number>(0);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    loadGlossary();
    loadCacheStats();
    const interval = setInterval(loadCacheStats, 2000);
    return () => clearInterval(interval);
  }, []);

  const loadGlossary = async () => {
    try {
      const dict = await invoke<Record<string, string>>('get_glossary_cmd');
      setDictionary(dict);
    } catch (e) {
      console.error(e);
    }
  };

  const loadCacheStats = async () => {
    try {
      const count = await invoke<number>('get_cache_stats_cmd');
      setCacheCount(count);
    } catch (e) {
      console.error(e);
    }
  };

  const handleAddTerm = () => {
    if (!newWrong.trim() || !newRight.trim()) return;
    setDictionary((prev) => ({
      ...prev,
      [newWrong.trim().toLowerCase()]: newRight.trim().toLowerCase(),
    }));
    setNewWrong('');
    setNewRight('');
  };

  const handleRemoveTerm = (key: string) => {
    setDictionary((prev) => {
      const updated = { ...prev };
      delete updated[key];
      return updated;
    });
  };

  const handleSaveGlossary = async () => {
    try {
      await invoke('save_glossary_cmd', { newDict: dictionary });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error(e);
    }
  };

  const handleClearCache = async () => {
    if (window.confirm('Вы уверены, что хотите очистить всю базу кэша переводов?')) {
      try {
        await invoke('clear_cache_cmd');
        loadCacheStats();
      } catch (e) {
        console.error(e);
      }
    }
  };

  const handleExportCache = async () => {
    try {
      const path = await save({
        filters: [{ name: 'JSON', extensions: ['json'] }],
        defaultPath: 'modlang_cache_export.json',
      });
      if (path && typeof path === 'string') {
        await invoke('export_cache_cmd', { exportPath: path });
        alert('Кэш успешно экспортирован!');
      }
    } catch (e) {
      console.error(e);
    }
  };

  const handleImportCache = async () => {
    try {
      const path = await open({
        filters: [{ name: 'JSON', extensions: ['json'] }],
        multiple: false,
      });
      if (path && typeof path === 'string') {
        const count = await invoke<number>('import_cache_cmd', { importPath: path });
        alert(`Импортировано записей кэша: ${count}`);
        loadCacheStats();
      }
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="grid grid-cols-12 gap-4 h-[calc(100vh-120px)] overflow-hidden">
      {/* Left: Terminology Dictionary */}
      <div className="col-span-8 flex flex-col gap-3 h-full overflow-hidden">
        <div className="mc-panel p-3 flex justify-between items-center flex-shrink-0">
          <div>
            <h2 className="text-xs font-pixel text-mc-gold uppercase font-bold">Словарь терминологии Minecraft</h2>
            <p className="text-xs text-mc-textMuted font-mono">Автоматическая замена машинных ошибок на каноничные термины</p>
          </div>
          <McButton
            variant="green"
            size="sm"
            onClick={handleSaveGlossary}
            icon={saved ? <Check className="w-3.5 h-3.5" /> : <Save className="w-3.5 h-3.5" />}
          >
            {saved ? 'СОХРАНЕНО' : 'СОХРАНИТЬ СЛОВАРЬ'}
          </McButton>
        </div>

        {/* Add New Term Input Bar */}
        <div className="mc-panel p-3 flex gap-2 items-end flex-shrink-0">
          <div className="flex-1">
            <McInput
              label="Ошибочное слово / термин"
              placeholder="полуслой"
              value={newWrong}
              onChange={(e) => setNewWrong(e.target.value)}
            />
          </div>
          <div className="text-mc-textMuted font-pixel text-lg pb-1.5">➔</div>
          <div className="flex-1">
            <McInput
              label="Правильный перевод"
              placeholder="плита"
              value={newRight}
              onChange={(e) => setNewRight(e.target.value)}
            />
          </div>
          <McButton
            variant="gold"
            size="md"
            onClick={handleAddTerm}
            icon={<Plus className="w-4 h-4" />}
          >
            Добавить
          </McButton>
        </div>

        {/* Terminology Table */}
        <div className="mc-panel flex-1 overflow-hidden flex flex-col">
          <div className="grid grid-cols-12 px-4 py-2 bg-[#141418] border-b border-mc-borderDark text-xs font-pixel text-mc-gold">
            <div className="col-span-5">Оригинальное / ошибочное слово</div>
            <div className="col-span-1 text-center">➔</div>
            <div className="col-span-5">Каноничная замена</div>
            <div className="col-span-1 text-right">Удалить</div>
          </div>

          <div className="overflow-y-auto flex-1 p-2 space-y-1">
            {Object.entries(dictionary).length === 0 ? (
              <div className="text-center py-12 text-zinc-600 font-pixel">
                Словарь пуст. Добавьте термины выше.
              </div>
            ) : (
              Object.entries(dictionary).map(([wrong, right]) => (
                <div
                  key={wrong}
                  className="grid grid-cols-12 items-center px-3 py-1.5 mc-slot text-xs font-mono"
                >
                  <div className="col-span-5 text-mc-redstone font-medium">{wrong}</div>
                  <div className="col-span-1 text-center text-zinc-600">➔</div>
                  <div className="col-span-5 text-mc-emerald font-medium">{right}</div>
                  <div className="col-span-1 text-right">
                    <button
                      onClick={() => handleRemoveTerm(wrong)}
                      className="text-zinc-500 hover:text-mc-redstone p-1 transition-colors"
                      title="Удалить"
                    >
                      <Trash2 className="w-3.5 h-3.5 ml-auto" />
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {/* Right: Translation Cache Manager */}
      <div className="col-span-4 flex flex-col gap-3 h-full overflow-hidden">
        <div className="mc-panel p-4 flex flex-col gap-4">
          <div className="flex items-center gap-2 border-b border-mc-borderDark pb-2">
            <Database className="w-4 h-4 text-mc-emerald" />
            <h2 className="text-xs font-pixel text-mc-emerald uppercase font-bold">База кэша переводов</h2>
          </div>

          <div className="mc-slot p-4 flex flex-col items-center justify-center gap-1 text-center">
            <span className="text-2xl font-bold font-mono text-mc-emerald">{cacheCount}</span>
            <span className="text-xs text-mc-textMuted font-pixel">ЗАКЭШИРОВАНО СТРОК</span>
          </div>

          <p className="text-xs text-zinc-400 font-mono leading-relaxed">
            Кэш хранит все однажды переведенные фразы и предметы, исключая повторные затраты токенов нейросетей и ускоряя перевод больших сборок в разы.
          </p>

          <div className="flex flex-col gap-2 pt-2">
            <McButton
              variant="gray"
              size="sm"
              onClick={handleExportCache}
              icon={<Download className="w-3.5 h-3.5" />}
            >
              Экспорт кэша в JSON
            </McButton>
            <McButton
              variant="gray"
              size="sm"
              onClick={handleImportCache}
              icon={<Upload className="w-3.5 h-3.5" />}
            >
              Импорт кэша из JSON
            </McButton>
            <div className="pt-2 border-t border-mc-borderDark">
              <McButton
                variant="red"
                size="sm"
                className="w-full"
                onClick={handleClearCache}
                icon={<Trash2 className="w-3.5 h-3.5" />}
              >
                Очистить базу кэша
              </McButton>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

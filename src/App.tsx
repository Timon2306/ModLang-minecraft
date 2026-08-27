import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Languages, BarChart3, Settings, BookOpen } from 'lucide-react';
import { McTabs, TabItem } from './components/common/McTabs';
import { TranslateScreen } from './components/screens/TranslateScreen';
import { AnalysisScreen } from './components/screens/AnalysisScreen';
import { SettingsScreen } from './components/screens/SettingsScreen';
import { GlossaryScreen } from './components/screens/GlossaryScreen';
import {
  AnalysisResult,
  AppConfig,
  LanguageInfo,
  LogEvent,
  ProgressEvent,
  StatusEvent,
} from './types';

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState('translate');
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [languages, setLanguages] = useState<LanguageInfo[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [analysisResult, setAnalysisResult] = useState<AnalysisResult | null>(null);
  const [logs, setLogs] = useState<LogEvent[]>([]);
  const [statusText, setStatusText] = useState('Ожидание...');
  const [progress, setProgress] = useState<ProgressEvent>({
    current: 0,
    total: 0,
    percent: 0,
    eta: 'готово',
    rate: 0,
  });

  useEffect(() => {
    // Initial Load
    loadInitialData();

    // Setup Tauri Event Listeners
    let unlistenLog: () => void;
    let unlistenProgress: () => void;
    let unlistenStatus: () => void;

    listen<LogEvent>('translation-log', (event) => {
      setLogs((prev) => [...prev, event.payload]);
    }).then((un) => (unlistenLog = un));

    listen<ProgressEvent>('translation-progress', (event) => {
      setProgress(event.payload);
    }).then((un) => (unlistenProgress = un));

    listen<StatusEvent>('translation-status', (event) => {
      setStatusText(event.payload.text);
      if (event.payload.text.includes('Готово') || event.payload.text.includes('Остановлено')) {
        setIsRunning(false);
        setIsPaused(false);
      }
    }).then((un) => (unlistenStatus = un));

    return () => {
      if (unlistenLog) unlistenLog();
      if (unlistenProgress) unlistenProgress();
      if (unlistenStatus) unlistenStatus();
    };
  }, []);

  const loadInitialData = async () => {
    try {
      const cfg = await invoke<AppConfig>('get_settings_cmd');
      setConfig(cfg);
      const langs = await invoke<LanguageInfo[]>('get_languages_cmd');
      setLanguages(langs);
    } catch (e) {
      console.error('Failed to load initial settings:', e);
    }
  };

  const handleStart = async () => {
    if (!config?.mc_dir) {
      alert('Пожалуйста, выберите папку Minecraft!');
      return;
    }
    setIsRunning(true);
    setIsPaused(false);
    setStatusText('В процессе перевода...');
    setLogs([]); // Автоматически очищаем лог от старых записей и ошибок прошлого запуска
    setProgress({ current: 0, total: 0, percent: 0, eta: 'расчёт...', rate: 0 });
    try {
      await invoke('start_translation_cmd', { config });
    } catch (e) {
      setIsRunning(false);
      setStatusText('Ошибка запуска');
      console.error(e);
    }
  };

  const handlePause = async () => {
    try {
      await invoke('pause_translation_cmd');
      setIsPaused(true);
    } catch (e) {
      console.error(e);
    }
  };

  const handleResume = async () => {
    try {
      await invoke('resume_translation_cmd');
      setIsPaused(false);
    } catch (e) {
      console.error(e);
    }
  };

  const handleStop = async () => {
    try {
      await invoke('stop_translation_cmd');
      setIsRunning(false);
      setIsPaused(false);
    } catch (e) {
      console.error(e);
    }
  };

  const handleRunAnalysis = async () => {
    if (!config?.mc_dir) {
      alert('Пожалуйста, выберите папку Minecraft!');
      return;
    }
    setIsAnalyzing(true);
    try {
      const result = await invoke<AnalysisResult>('analyze_modpack_cmd');
      setAnalysisResult(result);
    } catch (e) {
      console.error(e);
    } finally {
      setIsAnalyzing(false);
    }
  };

  const tabs: TabItem[] = [
    { id: 'translate', label: 'ПЕРЕВОД', icon: <Languages className="w-4 h-4 text-mc-emerald" /> },
    { id: 'analysis', label: 'АНАЛИЗ СБОРКИ', icon: <BarChart3 className="w-4 h-4 text-mc-gold" /> },
    { id: 'settings', label: 'НАСТРОЙКИ API', icon: <Settings className="w-4 h-4 text-mc-diamond" /> },
    { id: 'glossary', label: 'ГЛОССАРИЙ И КЭШ', icon: <BookOpen className="w-4 h-4 text-mc-purple" /> },
  ];

  if (!config) {
    return (
      <div className="flex items-center justify-center h-screen bg-mc-bg text-mc-gold font-pixel text-lg">
        Загрузка ModLang...
      </div>
    );
  }

  return (
    <div className="flex flex-col h-screen p-4 bg-mc-bg text-mc-text overflow-hidden select-none">
      {/* Top Header Bar */}
      <header className="grid grid-cols-12 items-center pb-3 mb-1 border-b border-mc-borderDark">
        <div className="col-span-2"></div>

        {/* Navigation Tabs (Centered and Rock-Solid) */}
        <div className="col-span-7 flex justify-center">
          <McTabs tabs={tabs} activeTab={activeTab} onChange={setActiveTab} />
        </div>

        {/* Live Status Pill (Fixed Width on the right) */}
        <div className="col-span-3 flex justify-end">
          <div className="flex items-center gap-2 font-mono text-xs mc-slot px-3 py-1.5 w-60 justify-start overflow-hidden">
            <div
              className={`w-2.5 h-2.5 rounded-full flex-shrink-0 ${
                isRunning ? (isPaused ? 'bg-mc-gold animate-pulse' : 'bg-mc-emerald animate-ping') : 'bg-zinc-600'
              }`}
            />
            <span className="text-zinc-300 font-medium truncate" title={statusText}>{statusText}</span>
          </div>
        </div>
      </header>

      {/* Main Content Body */}
      <main className="flex-1 overflow-hidden pt-2">
        {activeTab === 'translate' && (
          <TranslateScreen
            config={config}
            languages={languages}
            isRunning={isRunning}
            isPaused={isPaused}
            progress={progress}
            logs={logs}
            onConfigChange={setConfig}
            onStart={handleStart}
            onPause={handlePause}
            onResume={handleResume}
            onStop={handleStop}
            onClearLogs={() => setLogs([])}
            onGoToAnalysis={() => {
              setActiveTab('analysis');
              handleRunAnalysis();
            }}
          />
        )}

        {activeTab === 'analysis' && (
          <AnalysisScreen
            analysis={analysisResult}
            isAnalyzing={isAnalyzing}
            onRefresh={handleRunAnalysis}
          />
        )}

        {activeTab === 'settings' && (
          <SettingsScreen config={config} onConfigChange={setConfig} />
        )}

        {activeTab === 'glossary' && <GlossaryScreen />}
      </main>
    </div>
  );
};
export default App;

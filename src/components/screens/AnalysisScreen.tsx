import React, { useState } from 'react';
import { RefreshCw, Search, Package, BookOpen, FileText, ScrollText, Layers } from 'lucide-react';
import { McButton } from '../common/McButton';
import { McInput } from '../common/McInput';
import { AnalysisResult } from '../../types';

interface AnalysisScreenProps {
  analysis: AnalysisResult | null;
  isAnalyzing: boolean;
  onRefresh: () => void;
}

export const AnalysisScreen: React.FC<AnalysisScreenProps> = ({
  analysis,
  isAnalyzing,
  onRefresh,
}) => {
  const [search, setSearch] = useState('');
  const [kindFilter, setKindFilter] = useState('all');

  const rows = analysis?.rows || [];
  const filtered = rows.filter((r) => {
    const matchesSearch = r.name.toLowerCase().includes(search.toLowerCase());
    const matchesKind = kindFilter === 'all' || r.kind === kindFilter;
    return matchesSearch && matchesKind;
  });

  const getKindIcon = (kind: string) => {
    switch (kind) {
      case 'Интерфейс':
        return <Package className="w-4 h-4 text-mc-cyan" />;
      case 'Книга JSON':
        return <BookOpen className="w-4 h-4 text-mc-gold" />;
      case 'Книга MD':
        return <FileText className="w-4 h-4 text-mc-emerald" />;
      case 'Квесты':
        return <ScrollText className="w-4 h-4 text-mc-purple" />;
      default:
        return <Layers className="w-4 h-4 text-zinc-400" />;
    }
  };

  return (
    <div className="flex flex-col gap-4 h-[calc(100vh-120px)] overflow-hidden">
      {/* Top Controls Bar */}
      <div className="mc-panel p-3 flex justify-between items-center gap-4 flex-shrink-0">
        <div className="flex items-center gap-3 flex-1 max-w-md">
          <div className="relative w-full">
            <McInput
              placeholder="Поиск по названию мода / файла..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-8"
            />
            <Search className="w-4 h-4 text-zinc-500 absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
          </div>
          <select
            value={kindFilter}
            onChange={(e) => setKindFilter(e.target.value)}
            className="mc-slot px-3 py-2 text-xs font-mono text-mc-text focus:outline-none"
          >
            <option value="all">Все типы</option>
            <option value="Интерфейс">Интерфейс (Языковые файлы)</option>
            <option value="Книга JSON">Книга JSON (Patchouli)</option>
            <option value="Книга MD">Книга MD (Guide-API)</option>
            <option value="Квесты">Квесты (FTB / BQ)</option>
          </select>
        </div>

        {/* Global Stats and Refresh */}
        <div className="flex items-center gap-4">
          {analysis && (
            <div className="flex items-center gap-3 font-mono text-xs">
              <span className="text-zinc-400">
                Всего строк: <strong className="text-white">{analysis.total_strings}</strong>
              </span>
              <span className="text-zinc-400">
                Переведено: <strong className="text-mc-emerald">{analysis.total_translated}</strong>
              </span>
              <span className="mc-slot px-2 py-1 text-mc-emerald font-pixel font-bold">
                {analysis.overall_percent}%
              </span>
            </div>
          )}
          <McButton
            variant="gray"
            size="sm"
            onClick={onRefresh}
            disabled={isAnalyzing}
            icon={<RefreshCw className={`w-3.5 h-3.5 ${isAnalyzing ? 'animate-spin' : ''}`} />}
          >
            {isAnalyzing ? 'Сканирование...' : 'Обновить анализ'}
          </McButton>
        </div>
      </div>

      {/* Analysis Table */}
      <div className="mc-panel flex-1 overflow-hidden flex flex-col">
        <div className="grid grid-cols-12 px-4 py-2 bg-[#141418] border-b border-mc-borderDark text-xs font-pixel text-mc-gold">
          <div className="col-span-1">Тип</div>
          <div className="col-span-6">Файл / Мод</div>
          <div className="col-span-2 text-right">Переведено</div>
          <div className="col-span-3 text-right pr-2">Прогресс</div>
        </div>

        <div className="overflow-y-auto flex-1 p-2 space-y-1">
          {filtered.length === 0 ? (
            <div className="text-center py-16 text-zinc-600 font-pixel">
              {isAnalyzing ? 'Выполняется сканирование файлов...' : 'Нет данных анализа. Нажмите «Обновить анализ»'}
            </div>
          ) : (
            filtered.map((row, idx) => {
              const color =
                row.percent >= 90
                  ? 'text-mc-emerald'
                  : row.percent >= 50
                  ? 'text-mc-gold'
                  : row.percent > 0
                  ? 'text-mc-cyan'
                  : 'text-zinc-500';

              return (
                <div
                  key={idx}
                  className="grid grid-cols-12 items-center px-4 py-2 hover:bg-[#202028] transition-colors border-b border-mc-borderDark/40 font-mono text-xs"
                >
                  <div className="col-span-1 flex items-center gap-1.5" title={row.kind}>
                    {getKindIcon(row.kind)}
                  </div>
                  <div className="col-span-6 truncate text-zinc-200 font-medium" title={row.name}>
                    {row.name}
                  </div>
                  <div className="col-span-2 text-right text-zinc-400">
                    <span className="text-mc-text">{row.translated_count}</span>
                    <span className="text-zinc-600"> / {row.total_count}</span>
                  </div>
                  <div className="col-span-3 flex items-center justify-end gap-3 pr-2">
                    <div className="w-24 h-2 bg-[#101014] border border-mc-borderDark relative overflow-hidden flex-shrink-0">
                      <div
                        className="h-full bg-mc-emerald transition-all duration-300"
                        style={{ width: `${row.percent}%` }}
                      />
                    </div>
                    <span className={`w-10 text-right font-pixel text-xs font-bold ${color}`}>
                      {row.percent}%
                    </span>
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};

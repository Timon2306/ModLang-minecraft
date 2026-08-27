import React, { useEffect, useRef, useState } from 'react';
import { Trash2, Copy, Check } from 'lucide-react';
import { LogEvent } from '../../types';

interface McChatConsoleProps {
  logs: LogEvent[];
  onClear: () => void;
}

export const McChatConsole: React.FC<McChatConsoleProps> = ({ logs, onClear }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (autoScroll && containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [logs, autoScroll]);

  const handleScroll = () => {
    if (!containerRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } = containerRef.current;
    const atBottom = scrollHeight - scrollTop - clientHeight < 20;
    setAutoScroll(atBottom);
  };

  const handleCopy = () => {
    const text = logs.map((l) => l.message).join('\n');
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const getColorClass = (color: string) => {
    switch (color) {
      case 'green':
        return 'text-[#55FF55]';
      case 'yellow':
        return 'text-[#FFAA00]';
      case 'red':
        return 'text-[#FF5555]';
      case 'cyan':
        return 'text-[#55FFFF]';
      case 'magenta':
        return 'text-[#FF55FF]';
      case 'dim':
        return 'text-zinc-500';
      case 'white':
      default:
        return 'text-zinc-200';
    }
  };

  return (
    <div className="mc-panel flex flex-col h-full rounded-none overflow-hidden relative">
      {/* Console Header Toolbar */}
      <div className="flex items-center justify-between px-3 py-1.5 bg-[#141418] border-b border-mc-borderDark text-xs font-pixel text-mc-textMuted">
        <div className="flex items-center gap-2">
          <span className="text-mc-gold font-bold">ИГРОВОЙ ЧАТ ЛОГОВ</span>
          <span className="text-zinc-600 font-mono">[{logs.length} строк]</span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleCopy}
            title="Скопировать логи"
            className="mc-btn px-2 py-0.5 text-xs flex items-center gap-1 text-zinc-300 hover:text-white"
          >
            {copied ? <Check className="w-3 h-3 text-mc-emerald" /> : <Copy className="w-3 h-3" />}
            <span>{copied ? 'Скопировано' : 'Копия'}</span>
          </button>
          <button
            onClick={onClear}
            title="Очистить консоль"
            className="mc-btn px-2 py-0.5 text-xs flex items-center gap-1 text-zinc-300 hover:text-white"
          >
            <Trash2 className="w-3 h-3" />
            <span>Очистить</span>
          </button>
        </div>
      </div>

      {/* Console Body */}
      <div
        ref={containerRef}
        onScroll={handleScroll}
        className="mc-slot p-3 flex-1 overflow-y-auto font-mono text-xs leading-relaxed space-y-0.5 select-text bg-[#0c0c10]"
      >
        {logs.length === 0 ? (
          <div className="text-zinc-600 italic font-pixel">
            [Чат пуст. Выберите сборку и нажмите «Начать перевод» или «Анализ»]
          </div>
        ) : (
          logs.map((log, i) => (
            <div key={i} className={`whitespace-pre-wrap break-all ${getColorClass(log.color)}`}>
              {log.message}
            </div>
          ))
        )}
      </div>
    </div>
  );
};

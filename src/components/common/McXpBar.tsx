import React from 'react';

interface McXpBarProps {
  current: number;
  total: number;
  percent: number;
  eta: string;
  rate?: number;
}

export const McXpBar: React.FC<McXpBarProps> = ({
  current,
  total,
  percent,
  eta,
  rate = 0,
}) => {
  const clampedPercent = Math.min(100, Math.max(0, percent));

  return (
    <div className="flex flex-col gap-1.5 w-full">
      <div className="flex justify-between items-center text-xs font-pixel px-1">
        <div className="flex items-center gap-2">
          <span className="text-mc-emerald text-sm font-bold">УРОВЕНЬ: {clampedPercent}%</span>
          <span className="text-mc-textMuted font-mono">({current} / {total} модов / файлов)</span>
        </div>
        <div className="flex items-center gap-3 font-mono text-xs">
          {rate > 0 && <span className="text-zinc-400">⚡ {rate.toFixed(1)} мод/сек</span>}
          <span className="text-mc-gold font-pixel">⏱ ETA: {eta}</span>
        </div>
      </div>
      
      {/* Textured Minecraft XP Track */}
      <div className="mc-xp-track w-full">
        <div
          className="mc-xp-fill"
          style={{ width: `${clampedPercent}%` }}
        />
      </div>
    </div>
  );
};

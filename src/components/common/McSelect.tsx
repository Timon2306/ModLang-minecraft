import React from 'react';
import { clsx } from 'clsx';

interface Option {
  value: string;
  label: string;
}

interface McSelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  options: Option[];
}

export const McSelect: React.FC<McSelectProps> = ({
  label,
  options,
  className,
  ...props
}) => {
  return (
    <div className="flex flex-col gap-1 w-full">
      {label && (
        <label className="text-xs font-pixel text-mc-textMuted uppercase tracking-wider">
          {label}
        </label>
      )}
      <div className="relative">
        <select
          className={clsx(
            'mc-slot px-3 py-2 text-sm text-mc-text font-mono focus:outline-none appearance-none w-full cursor-pointer pr-8',
            className
          )}
          {...props}
        >
          {options.map((opt) => (
            <option key={opt.value} value={opt.value} className="bg-mc-surface text-mc-text">
              {opt.label}
            </option>
          ))}
        </select>
        <div className="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-mc-textMuted text-xs font-pixel">
          ▼
        </div>
      </div>
    </div>
  );
};

import React from 'react';
import { clsx } from 'clsx';

interface McInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
}

export const McInput: React.FC<McInputProps> = ({
  label,
  error,
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
      <input
        className={clsx(
          'mc-slot px-3 py-2 text-sm text-mc-text font-mono placeholder:text-zinc-600 focus:outline-none transition-colors w-full',
          error && 'border-red-500',
          className
        )}
        {...props}
      />
      {error && <span className="text-xs text-mc-redstone font-mono">{error}</span>}
    </div>
  );
};

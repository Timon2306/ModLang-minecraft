import React from 'react';
import { clsx } from 'clsx';
import { Check } from 'lucide-react';

interface McCheckboxProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  description?: string;
  disabled?: boolean;
}

export const McCheckbox: React.FC<McCheckboxProps> = ({
  label,
  checked,
  onChange,
  description,
  disabled,
}) => {
  return (
    <label
      className={clsx(
        'flex items-start gap-2.5 cursor-pointer select-none py-1 group',
        disabled && 'opacity-50 cursor-not-allowed'
      )}
      onClick={() => !disabled && onChange(!checked)}
    >
      <div
        className={clsx(
          'w-5 h-5 mc-slot flex items-center justify-center flex-shrink-0 transition-colors mt-0.5',
          checked && 'border-mc-emerald bg-[#132213]'
        )}
      >
        {checked && <Check className="w-3.5 h-3.5 text-mc-emerald stroke-[3]" />}
      </div>
      <div className="flex flex-col">
        <span className="text-sm font-sans text-mc-text group-hover:text-white transition-colors">
          {label}
        </span>
        {description && (
          <span className="text-xs text-mc-textMuted font-mono leading-tight">
            {description}
          </span>
        )}
      </div>
    </label>
  );
};

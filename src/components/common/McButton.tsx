import React from 'react';
import { clsx } from 'clsx';

interface McButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'gray' | 'green' | 'red' | 'gold';
  size?: 'sm' | 'md' | 'lg';
  icon?: React.ReactNode;
}

export const McButton: React.FC<McButtonProps> = ({
  children,
  variant = 'gray',
  size = 'md',
  icon,
  className,
  disabled,
  ...props
}) => {
  const sizeClasses = {
    sm: 'px-2.5 py-1 text-xs',
    md: 'px-4 py-2 text-sm',
    lg: 'px-6 py-3 text-base',
  }[size];

  const variantClass = {
    gray: '',
    green: 'mc-btn-green text-white',
    red: 'mc-btn-red text-white',
    gold: 'mc-btn-gold text-yellow-100',
  }[variant];

  return (
    <button
      className={clsx(
        'mc-btn flex items-center justify-center gap-2 rounded-none transition-transform active:translate-y-0.5',
        sizeClasses,
        variantClass,
        disabled && 'opacity-50 cursor-not-allowed',
        className
      )}
      disabled={disabled}
      {...props}
    >
      {icon && <span className="flex-shrink-0">{icon}</span>}
      <span>{children}</span>
    </button>
  );
};

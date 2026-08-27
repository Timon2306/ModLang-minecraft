import React from 'react';
import { clsx } from 'clsx';

export interface TabItem {
  id: string;
  label: string;
  icon?: React.ReactNode;
}

interface McTabsProps {
  tabs: TabItem[];
  activeTab: string;
  onChange: (id: string) => void;
}

export const McTabs: React.FC<McTabsProps> = ({ tabs, activeTab, onChange }) => {
  return (
    <div className="flex gap-2 border-b-2 border-mc-borderDark pb-2">
      {tabs.map((tab) => {
        const isActive = tab.id === activeTab;
        return (
          <button
            key={tab.id}
            onClick={() => onChange(tab.id)}
            className={clsx(
              'mc-btn px-4 py-2 text-sm flex items-center gap-2 transition-all',
              isActive
                ? 'mc-btn-gold text-yellow-100 border-b-0'
                : 'text-zinc-400 hover:text-white'
            )}
          >
            {tab.icon}
            <span>{tab.label}</span>
          </button>
        );
      })}
    </div>
  );
};

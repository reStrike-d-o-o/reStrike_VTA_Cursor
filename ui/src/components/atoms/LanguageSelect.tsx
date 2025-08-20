import React, { useEffect, useRef, useState } from 'react';

export interface LanguageOption {
  code: string; // 'en', 'sr', 'hr', 'de', 'fr'
  label: string; // Localized name
  flag: string; // IOC-like code mapped to our svg assets, e.g. 'GBR'
}

interface LanguageSelectProps {
  value: string;
  onChange: (code: string) => void;
  options?: LanguageOption[];
  className?: string;
}

const defaultOptions: LanguageOption[] = [
  { code: 'en', label: 'English', flag: 'GBR' },
  { code: 'sr', label: 'Srpski', flag: 'SRB' },
  { code: 'hr', label: 'Hrvatski', flag: 'CRO' },
  { code: 'de', label: 'Deutsch', flag: 'GER' },
  { code: 'fr', label: 'Français', flag: 'FRA' },
];

const LanguageSelect: React.FC<LanguageSelectProps> = ({ value, onChange, options = defaultOptions, className = '' }) => {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onDocClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        console.log('[LanguageSelect] outside click -> closing');
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', onDocClick);
    return () => document.removeEventListener('mousedown', onDocClick);
  }, []);

  const current = options.find(o => o.code === value) || options[0];
  useEffect(() => {
    console.log('[LanguageSelect] mount', { value, current, options: options.map(o => o.code) });
  }, []);
  useEffect(() => {
    console.log('[LanguageSelect] value changed', { value, current });
  }, [value]);
  useEffect(() => {
    console.log('[LanguageSelect] open state changed', { open });
  }, [open]);

  return (
    <div ref={ref} className={`relative inline-block ${className}`}>
      <button
        type="button"
        className="flex items-center gap-2 bg-gray-700 text-gray-200 text-sm px-2 py-1 rounded border border-gray-600 hover:bg-gray-650"
        onClick={() => { console.log('[LanguageSelect] toggle clicked', { prevOpen: open, nextOpen: !open }); setOpen(o => !o); }}
        aria-haspopup="listbox"
        aria-label="Language"
        title="Language selector"
      >
        <img
          src={`/assets/flags/svg/${current.flag}.svg`}
          alt=""
          width={18}
          height={12}
          className="inline-block"
        />
        <span>{current.label}</span>
        <svg width="12" height="12" viewBox="0 0 20 20" fill="currentColor" className="text-gray-300">
          <path fillRule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 10.94l3.71-3.71a.75.75 0 111.06 1.06l-4.24 4.24a.75.75 0 01-1.06 0L5.21 8.29a.75.75 0 01.02-1.08z" clipRule="evenodd" />
        </svg>
      </button>

      {open && (
        <ul
          role="listbox"
          aria-label="Available languages"
          className="absolute z-20 mt-1 w-44 max-h-64 overflow-auto rounded border border-gray-600 bg-gray-800 shadow-lg"
        >
          {options.map(opt => (
            <li
              key={opt.code}
              role="option"
              className={`px-2 py-1 flex items-center gap-2 cursor-pointer hover:bg-gray-700 ${opt.code === value ? 'bg-gray-700' : ''}`}
              onClick={() => { console.log('[LanguageSelect] select clicked', { requested: opt.code }); onChange(opt.code); setOpen(false); }}
            >
              <img src={`/assets/flags/svg/${opt.flag}.svg`} alt="" width={18} height={12} />
              <span className="text-sm text-gray-200">{opt.label}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};

export default LanguageSelect;



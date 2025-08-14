import React from 'react';
import { useMessageCenter } from '../../stores/messageCenter';

const severityClasses: Record<string, string> = {
  info: 'border-blue-500/40 bg-blue-900/30 text-blue-200',
  success: 'border-green-500/40 bg-green-900/30 text-green-200',
  warning: 'border-yellow-500/40 bg-yellow-900/30 text-yellow-100',
  error: 'border-red-500/40 bg-red-900/30 text-red-200',
};

const GlobalModals: React.FC = () => {
  const { current, close } = useMessageCenter();

  React.useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (!current) return;
      if (e.key === 'Escape') close(false);
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [current, close]);

  if (!current) return null;

  const className = severityClasses[current.severity] || severityClasses.info;

  return (
    <div className="fixed inset-0 z-[1000] flex items-center justify-center">
      <div className="absolute inset-0 bg-black/60" onClick={() => close(false)} />
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="global-modal-title"
        className={`relative z-10 w-[520px] max-w-[95vw] rounded border ${className} shadow-xl`}
      >
        <div className="p-4 border-b border-white/10">
          <div id="global-modal-title" className="text-lg font-semibold">
            {current.title}
          </div>
        </div>
        <div className="p-4 text-sm whitespace-pre-wrap">{current.body}</div>
        <div className="p-3 border-t border-white/10 flex justify-end gap-2">
          {current.kind === 'confirm' ? (
            <>
              <button
                className="px-3 py-1 bg-gray-700 hover:bg-gray-600 text-gray-100 border border-gray-500"
                onClick={() => close(false)}
              >
                {current.cancelText || 'Cancel'}
              </button>
              <button
                className="px-3 py-1 bg-blue-700 hover:bg-blue-600 text-white border border-blue-500"
                onClick={() => close(true)}
              >
                {current.confirmText || 'Confirm'}
              </button>
            </>
          ) : current.kind === 'choices' ? (
            <>
              {(current.choices || []).map((c) => (
                <button
                  key={c.value}
                  className="px-3 py-1 bg-blue-700 hover:bg-blue-600 text-white border border-blue-500"
                  onClick={() => close(c.value)}
                >
                  {c.text}
                </button>
              ))}
              <button
                className="px-3 py-1 bg-gray-700 hover:bg-gray-600 text-gray-100 border border-gray-500"
                onClick={() => close('cancel')}
              >
                Cancel
              </button>
            </>
          ) : (
            <button
              className="px-3 py-1 bg-blue-700 hover:bg-blue-600 text-white border border-blue-500"
              onClick={() => close(true)}
            >
              OK
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default GlobalModals;



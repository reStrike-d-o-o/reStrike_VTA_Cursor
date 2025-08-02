import React from 'react';
import { useConfirmStore } from '../../stores/confirmStore';
import Button from '../atoms/Button';

const DestructiveConfirmModal: React.FC = () => {
  const { isOpen, title, message, remaining, delayMs, cancel, confirm } = useConfirmStore();
  if (!isOpen) return null;

  const pct = Math.min(100, ((delayMs - remaining * 1000) / delayMs) * 100);

  return (
    <div className="fixed inset-0 z-[10001] flex items-center justify-center bg-black/70">
      <div className="bg-gray-800 border border-gray-600 rounded-lg p-6 w-96 text-center shadow-xl">
        <h2 className="text-xl font-semibold mb-4">{title}</h2>
        <p className="mb-4 text-sm text-gray-300">{message}</p>

        <div className="mb-4">
          <div className="h-2 bg-gray-700 rounded overflow-hidden">
            <div className="h-full bg-red-500 transition-all duration-100" style={{ width: `${pct}%` }} />
          </div>
          {remaining > 0 ? (
            <p className="mt-2 text-yellow-400 text-sm">Confirm enabled in {remaining}s…</p>
          ) : (
            <p className="mt-2 text-green-400 text-sm">You can confirm now.</p>
          )}
        </div>

        <div className="flex justify-center gap-4">
          <Button variant="secondary" onClick={cancel}>Cancel</Button>
          <Button variant="danger" onClick={confirm}>Confirm</Button>
        </div>
      </div>
    </div>
  );
};

export default DestructiveConfirmModal;
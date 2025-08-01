import React from 'react';
import Button from '../atoms/Button';

interface LoadTemplate {
  id: number;
  name: string;
  created_at: string;
}

interface LoadModalProps {
  templates: LoadTemplate[];
  onSelect: (template: LoadTemplate) => void;
  onClose: () => void;
}

export const LoadModal: React.FC<LoadModalProps> = ({ templates, onSelect, onClose }) => {
  return (
    <div className="fixed inset-0 flex items-center justify-center z-50 bg-black/60">
      <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 rounded-lg border border-gray-600/30 shadow-lg w-[480px] max-h-[80vh] flex flex-col">
        <div className="p-4 border-b border-gray-700 flex justify-between items-center">
          <h3 className="text-lg font-semibold text-blue-300">Load Trigger Preset</h3>
          <Button variant="ghost" onClick={onClose}>✕</Button>
        </div>
        <div className="flex-1 overflow-auto">
          <table className="min-w-full text-left text-sm text-gray-200">
            <thead className="bg-[#101820] sticky top-0 z-10">
              <tr>
                <th className="px-3 py-2 w-[300px]">Name</th>
                <th className="px-3 py-2 w-[180px]">Created</th>
              </tr>
            </thead>
            <tbody>
              {templates.map(t => (
                <tr key={t.id} className="hover:bg-blue-900 cursor-pointer" onClick={() => onSelect(t)}>
                  <td className="px-3 py-2 whitespace-nowrap">{t.name}</td>
                  <td className="px-3 py-2 whitespace-nowrap text-gray-400">{t.created_at}</td>
                </tr>
              ))}
              {templates.length === 0 && (
                <tr>
                  <td className="px-3 py-6 text-center text-gray-500" colSpan={2}>No presets found</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};

interface SaveModalProps {
  onSave: (name: string) => void;
  onClose: () => void;
}

export const SaveModal: React.FC<SaveModalProps> = ({ onSave, onClose }) => {
  const [name, setName] = React.useState('');
  return (
    <div className="fixed inset-0 flex items-center justify-center z-50 bg-black/60">
      <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 rounded-lg border border-gray-600/30 shadow-lg w-[400px] p-6 space-y-4">
        <h3 className="text-lg font-semibold text-blue-300">Save Trigger Preset</h3>
        <input
          className="w-full px-3 py-2 bg-[#101820] border border-gray-700 rounded text-gray-200 focus:outline-none"
          placeholder="Preset name"
          value={name}
          onChange={e => setName(e.target.value)}
        />
        <div className="flex justify-end gap-2">
          <Button variant="secondary" onClick={onClose}>Cancel</Button>
          <Button variant="primary" onClick={() => name.trim() && onSave(name.trim())}>Save</Button>
        </div>
      </div>
    </div>
  );
};
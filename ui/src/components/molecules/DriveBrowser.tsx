import React, { useEffect, useMemo, useState } from 'react';
import Modal from '../atoms/Modal';
import Button from '../atoms/Button';
import Input from '../atoms/Input';

interface DriveItem { id: string; name: string; mimeType?: string; size?: string; createdTime?: string; modifiedTime?: string; }

type DriveBrowserMode = 'pick-zip' | 'pick-folder';

interface DriveBrowserProps {
  isOpen: boolean;
  mode: DriveBrowserMode;
  onClose: () => void;
  onPick: (item: DriveItem) => void;
}

const DriveBrowser: React.FC<DriveBrowserProps> = ({ isOpen, mode, onClose, onPick }) => {
  const [items, setItems] = useState<DriveItem[]>([]);
  const [filtered, setFiltered] = useState<DriveItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [query, setQuery] = useState('');
  const [sort, setSort] = useState<'name' | 'modified'>('modified');

  const filterByMode = (list: DriveItem[]) => {
    if (mode === 'pick-zip') {
      return list.filter(i => i.name?.toLowerCase().endsWith('.zip'));
    }
    // pick-folder: Google folders have mimeType value indicating folder; when missing, allow name-only fallback
    return list.filter(i => (i.mimeType?.includes('folder')) || (!i.mimeType && !i.name?.includes('.')));
  };

  const sortList = (list: DriveItem[]) => {
    if (sort === 'name') return [...list].sort((a, b) => (a.name||'').localeCompare(b.name||''));
    return [...list].sort((a, b) => (b.modifiedTime||'').localeCompare(a.modifiedTime||''));
  };

  const load = async () => {
    try {
      setLoading(true); setError(null);
      const { invoke } = await import('@tauri-apps/api/core');
      // Prefer full listing; backend can page later if needed
      const res: any = await invoke('drive_list_all_files');
      const list: DriveItem[] = Array.isArray(res?.files) ? res.files : (Array.isArray(res) ? res : (res?.data ?? []));
      setItems(list);
      const filtered = sortList(filterByMode(list));
      setFiltered(filtered);
    } catch (e: any) {
      setError(typeof e==='string'?e:(e?.message||'Failed to load Drive files'));
      setItems([]); setFiltered([]);
    } finally { setLoading(false); }
  };

  useEffect(() => { if (isOpen) { load(); } }, [isOpen]);

  useEffect(() => {
    const list = filterByMode(items).filter(i => (i.name||'').toLowerCase().includes(query.toLowerCase()));
    setFiltered(sortList(list));
  }, [query, sort, items, mode]);

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={mode==='pick-zip' ? 'Choose ZIP from Drive' : 'Choose Drive folder'}>
      <div className="flex items-center gap-3 mb-3">
        <Input placeholder="Search…" value={query} onChange={(e) => setQuery(e.target.value)} className="flex-1" />
        <select aria-label="Sort" className="bg-gray-800 text-gray-200 px-3 py-2 rounded" value={sort} onChange={(e)=>setSort(e.target.value as any)}>
          <option value="modified">Modified</option>
          <option value="name">Name</option>
        </select>
      </div>
      {loading && <div className="text-xs text-gray-400">Loading…</div>}
      {error && <div className="text-xs text-red-400 mb-2">{error}</div>}
      <div className="max-h-[50vh] overflow-auto border border-gray-700 rounded">
        <table className="min-w-full text-left text-sm text-gray-200">
          <thead className="theme-surface-2 sticky top-0 z-10">
            <tr>
              <th className="px-3 py-2 font-semibold">Name</th>
              <th className="px-3 py-2 font-semibold">Type</th>
              <th className="px-3 py-2 font-semibold">Modified</th>
              <th className="px-3 py-2 font-semibold">Size</th>
              <th className="px-3 py-2 font-semibold">Action</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((i) => (
              <tr key={i.id} className="hover:bg-blue-900">
                <td className="px-3 py-2 whitespace-nowrap">{i.name}</td>
                <td className="px-3 py-2 whitespace-nowrap">{i.mimeType || ''}</td>
                <td className="px-3 py-2 whitespace-nowrap">{i.modifiedTime || ''}</td>
                <td className="px-3 py-2 whitespace-nowrap">{i.size || ''}</td>
                <td className="px-3 py-2 whitespace-nowrap">
                  <Button variant="ghost" size="sm" onClick={()=>onPick(i)}>Select</Button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="flex justify-end gap-2 mt-3">
        <Button variant="secondary" size="sm" onClick={onClose}>Close</Button>
      </div>
    </Modal>
  );
};

export default DriveBrowser;

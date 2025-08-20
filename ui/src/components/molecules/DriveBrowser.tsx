import React, { useEffect, useMemo, useState } from 'react';
import Modal from '../atoms/Modal';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import { useI18n } from '../../i18n/index';

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
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [query, setQuery] = useState('');
  const [sort, setSort] = useState<'name' | 'modified'>('modified');
  const [breadcrumbs, setBreadcrumbs] = useState<{ id: string | null; name: string }[]>([{ id: null, name: 'My Drive' }]);
  const { t } = useI18n();

  const currentFolderId = breadcrumbs[breadcrumbs.length - 1]?.id;

  const loadChildren = async () => {
    try {
      setLoading(true); setError(null);
      const { invoke } = await import('@tauri-apps/api/core');
      const res: any = await invoke('drive_list_children', { parentId: currentFolderId || null });
      const list: DriveItem[] = Array.isArray(res?.files) ? res.files : (Array.isArray(res) ? res : (res?.data ?? []));
      setItems(list);
    } catch (e: any) {
      setError(typeof e==='string'?e:(e?.message||'Failed to load Drive files'));
      setItems([]);
    } finally { setLoading(false); }
  };

  useEffect(() => { if (isOpen) { loadChildren(); } }, [isOpen, currentFolderId]);

  const filtered = useMemo(() => {
    const base = items.filter(i => (i.name||'').toLowerCase().includes(query.toLowerCase()));
    const modeFiltered = mode === 'pick-zip' ? base.filter(i => i.name?.toLowerCase().endsWith('.zip')) : base;
    const sorted = sort === 'name' ? [...modeFiltered].sort((a,b) => (a.name||'').localeCompare(b.name||'')) : [...modeFiltered].sort((a,b) => (b.modifiedTime||'').localeCompare(a.modifiedTime||''));
    return sorted;
  }, [items, query, sort, mode]);

  const enterFolder = (item: DriveItem) => {
    if (item.mimeType?.includes('folder')) {
      setBreadcrumbs(prev => [...prev, { id: item.id, name: item.name }]);
    } else if (mode === 'pick-zip') {
      onPick(item);
    }
  };

  const goTo = (index: number) => {
    setBreadcrumbs(prev => prev.slice(0, index + 1));
  };

  const createFolder = async () => {
    const name = window.prompt(t('drive.folder_name', 'Folder name'));
    if (!name) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('drive_create_folder', { name, parentId: currentFolderId || null });
      await loadChildren();
    } catch (e: any) { alert(typeof e==='string'?e:(e?.message||t('drive.err_create_folder', 'Failed to create folder'))); }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={mode==='pick-zip' ? t('drive.choose_zip', 'Choose ZIP from Drive') : t('drive.choose_folder', 'Choose Drive folder')}>
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2 text-sm text-gray-300">
          {breadcrumbs.map((b, idx) => (
            <span key={idx}>
              <button className={`hover:underline ${idx===breadcrumbs.length-1?'text-white font-semibold':''}`} onClick={()=>goTo(idx)}>{b.name}</button>
              {idx<breadcrumbs.length-1 && <span className="mx-1">/</span>}
            </span>
          ))}
        </div>
        <div className="flex items-center gap-2">
          <Input placeholder={t('common.search', 'Search…')} value={query} onChange={(e) => setQuery(e.target.value)} className="w-56" />
          <select aria-label={t('common.sort', 'Sort')} className="bg-gray-800 text-gray-200 px-3 py-2 rounded" value={sort} onChange={(e)=>setSort(e.target.value as any)}>
            <option value="modified">{t('drive.modified', 'Modified')}</option>
            <option value="name">{t('drive.name', 'Name')}</option>
          </select>
          {mode==='pick-folder' && <Button variant="secondary" size="sm" onClick={createFolder}>{t('drive.new_folder', 'New Folder')}</Button>}
          {mode==='pick-folder' && <Button variant="primary" size="sm" onClick={()=>onPick({ id: currentFolderId || 'root', name: breadcrumbs[breadcrumbs.length-1].name })}>{t('drive.choose_here', 'Choose here')}</Button>}
        </div>
      </div>
      {loading && <div className="text-xs text-gray-400">{t('common.loading', 'Loading…')}</div>}
      {error && <div className="text-xs text-red-400 mb-2">{error}</div>}
      <div className="max-h-[50vh] overflow-auto border border-gray-700 rounded">
        <table className="min-w-full text-left text-sm text-gray-200">
          <thead className="theme-surface-2 sticky top-0 z-10">
            <tr>
              <th className="px-3 py-2 font-semibold">{t('drive.col.name', 'Name')}</th>
              <th className="px-3 py-2 font-semibold">{t('drive.col.type', 'Type')}</th>
              <th className="px-3 py-2 font-semibold">{t('drive.col.modified', 'Modified')}</th>
              <th className="px-3 py-2 font-semibold">{t('drive.col.size', 'Size')}</th>
              <th className="px-3 py-2 font-semibold">{t('drive.col.action', 'Action')}</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((i) => (
              <tr key={i.id} className="hover:bg-blue-900">
                <td className="px-3 py-2 whitespace-nowrap">{i.name}</td>
                <td className="px-3 py-2 whitespace-nowrap">{i.mimeType || ''}</td>
                <td className="px-3 py-2 whitespace-nowrap">{i.modifiedTime || ''}</td>
                <td className="px-3 py-2 whitespace-nowrap">{i.size || ''}</td>
                <td className="px-3 py-2 whitespace-nowrap flex gap-2">
                  {i.mimeType?.includes('folder') ? (
                    <Button variant="ghost" size="sm" onClick={()=>enterFolder(i)}>{t('common.open', 'Open')}</Button>
                  ) : (
                    <Button variant="ghost" size="sm" onClick={()=>onPick(i)}>{t('common.select', 'Select')}</Button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="flex justify-end gap-2 mt-3">
        <Button variant="secondary" size="sm" onClick={onClose}>{t('common.close', 'Close')}</Button>
      </div>
    </Modal>
  );
};

export default DriveBrowser;

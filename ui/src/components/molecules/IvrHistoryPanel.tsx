import React, { useEffect, useState } from 'react';
import VideoEventPicker from './VideoEventPicker';
import Button from '../atoms/Button';
import DriveBrowser from './DriveBrowser';
import { useI18n } from '../../i18n/index';

const ProgressToast: React.FC<{ message: string; onCancel?: () => void }> = ({ message, onCancel }) => (
	<div className="fixed right-6 bottom-6 z-50 theme-card p-4 shadow-lg flex items-center gap-3">
		<div className="text-sm text-gray-200">{message}</div>
		{onCancel && <Button variant="ghost" size="sm" onClick={onCancel}>Cancel</Button>}
	</div>
);

const IvrHistoryPanel: React.FC = () => {
	const { t } = useI18n();
	const [days, setDays] = useState<Array<any>>([]);
	const [selectedDayId, setSelectedDayId] = useState<number | null>(null);
	const [matches, setMatches] = useState<Array<any>>([]);
	const [selectedMatchId, setSelectedMatchId] = useState<number | null>(null);
	const [videos, setVideos] = useState<Array<any>>([]);
	const [selectedVideoIds, setSelectedVideoIds] = useState<Set<number>>(new Set());
	const [events, setEvents] = useState<Array<any>>([]);
	const [pickerVideoId, setPickerVideoId] = useState<number | null>(null);
	const [pickerMatchId, setPickerMatchId] = useState<number | null>(null);
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const [driveOpen, setDriveOpen] = useState<null | 'zip' | 'folder'>(null);
	const [progressMsg, setProgressMsg] = useState<string | null>(null);
	const [jobId, setJobId] = useState<string | null>(null);

	useEffect(() => {
		(async () => {
			try {
				setLoading(true); setError(null);
				const { invoke } = await import('@tauri-apps/api/core');
				const resp: any = await invoke('ivr_list_tournament_days');
				setDays(Array.isArray(resp) ? resp : (resp?.data ?? []));
			} catch (e: any) {
				setError(typeof e === 'string' ? e : (e?.message || t('ivr.history.err_load_days', 'Failed to load days')));
			} finally { setLoading(false); }
		})();
	}, []);

	useEffect(() => {
		(async () => {
			if (selectedDayId == null) { setMatches([]); setVideos([]); setSelectedVideoIds(new Set()); return; }
			try {
				setLoading(true); setError(null);
				const { invoke } = await import('@tauri-apps/api/core');
				const m: any = await invoke('ivr_list_matches_for_day', { day_id: selectedDayId, dayId: selectedDayId });
				setMatches(Array.isArray(m) ? m : (m?.data ?? []));
				const v: any = await invoke('ivr_list_recorded_videos', { tournament_day_id: selectedDayId, tournamentDayId: selectedDayId, match_id: selectedMatchId ?? undefined, matchId: selectedMatchId ?? undefined });
				setVideos(Array.isArray(v) ? v : (v?.data ?? []));
				setSelectedVideoIds(new Set());
			} catch (e: any) {
				setError(typeof e === 'string' ? e : (e?.message || t('ivr.history.err_load_matches', 'Failed to load matches/videos')));
			} finally { setLoading(false); }
		})();
	}, [selectedDayId, selectedMatchId]);

	useEffect(() => {
		(async () => {
			if (selectedMatchId == null) { setEvents([]); return; }
			try {
				const { invoke } = await import('@tauri-apps/api/core');
				const ev: any = await invoke('pss_get_events_for_match', { match_id: String(selectedMatchId), matchId: String(selectedMatchId) });
				const list = Array.isArray(ev) ? ev : (ev?.data ?? []);
				setEvents(list);
			} catch (e) {
				setEvents([]);
			}
		})();
	}, [selectedMatchId]);

	useEffect(() => {
		if (!(window as any).__TAURI__?.event) return;
		const unsubs: Array<Promise<() => void>> = [];
		unsubs.push((window as any).__TAURI__.event.listen('ivr_zip_progress', (e: any) => setProgressMsg(t('ivr.history.zip', 'Zipping {done}/{total}: {file}', { done: e?.payload?.items_done ?? 0, total: e?.payload?.items_total ?? 0, file: e?.payload?.file || '' }))));
		unsubs.push((window as any).__TAURI__.event.listen('ivr_upload_progress', (e: any) => setProgressMsg(e?.payload?.phase==='complete'? t('ivr.history.upload_complete', 'Upload complete') : t('ivr.history.upload_starting', 'Starting upload…'))));
		unsubs.push((window as any).__TAURI__.event.listen('ivr_download_progress', (_: any) => setProgressMsg(t('ivr.history.downloading', 'Downloading…'))));
		unsubs.push((window as any).__TAURI__.event.listen('ivr_extract_progress', (e: any) => setProgressMsg(t('ivr.history.extract', 'Extracting {done}/{total}: {file}', { done: e?.payload?.done ?? 0, total: e?.payload?.total ?? 0, file: e?.payload?.file || '' }))));
		unsubs.push((window as any).__TAURI__.event.listen('ivr_index_progress', (_: any) => setProgressMsg(t('ivr.history.index_complete', 'Index complete'))));
		return () => { unsubs.forEach(p => p.then(unsub => unsub()).catch(()=>{})); };
	}, []);

	const cancelJob = async () => {
		if (!jobId) return;
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			await invoke('ivr_cancel_job', { jobId: jobId });
			setProgressMsg(t('ivr.history.cancelled', 'Cancelled'));
		} catch {}
	};

	const toggleVideoSelection = (id: number) => {
		setSelectedVideoIds(prev => {
			const next = new Set(prev);
			if (next.has(id)) next.delete(id); else next.add(id);
			return next;
		});
	};

	const handleDeleteSelected = async () => {
		if (selectedVideoIds.size === 0) return;
		if (!window.confirm(t('ivr.history.confirm_delete', 'Delete selected videos and their records? This cannot be undone.'))) return;
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			const ids = Array.from(selectedVideoIds);
			const res: any = await invoke('ivr_delete_recorded_videos', { ids });
			if (res?.success !== false) {
				if (selectedDayId != null) {
					const v: any = await invoke('ivr_list_recorded_videos', { tournament_day_id: selectedDayId, tournamentDayId: selectedDayId, match_id: selectedMatchId ?? undefined, matchId: selectedMatchId ?? undefined });
					setVideos(Array.isArray(v) ? v : (v?.data ?? []));
					setSelectedVideoIds(new Set());
				}
			} else {
				alert(res?.error || t('ivr.history.err_delete', 'Failed to delete'));
			}
		} catch (e: any) {
			alert(typeof e === 'string' ? e : (e?.message || t('ivr.history.err_delete', 'Failed to delete')));
		}
	};

	return (
		<div className="space-y-6">
			{/* Header */}
			<div className="flex items-center justify-between">
				<div>
					<h2 className="text-xl font-semibold text-white">{t('ivr.history.title', 'Match history')}</h2>
					<p className="text-sm text-gray-400">{t('ivr.history.subtitle', 'Review recorded sessions by day, match and event')}</p>
				</div>
			</div>

			{/* Status */}
			{loading && <div className="text-xs text-gray-400">{t('common.loading', 'Loading…')}</div>}
			{error && (
				<div className="bg-red-900/20 border border-red-500/50 rounded-lg p-3">
					<span className="text-red-400 font-medium">{t('common.error', 'Error')}</span>
					<p className="text-red-300 mt-1 text-sm">{error}</p>
				</div>
			)}

			{/* Main Columns */}
			<div className="grid grid-cols-3 gap-4">
				{/* Days */}
				<div className="theme-card p-6 shadow-lg">
					<h3 className="text-lg font-semibold text-blue-300 mb-3">{t('ivr.history.days.title', 'Tournament / Day')}</h3>
					<div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
						<table className="min-w-full text-left text-sm text-gray-200">
							<thead className="theme-surface-2 sticky top-0 z-10">
								<tr>
									<th className="px-3 py-2 font-semibold">{t('ivr.history.days.col.tournament', 'Tournament')}</th>
									<th className="px-3 py-2 font-semibold">{t('ivr.history.days.col.day', 'Day')}</th>
									<th className="px-3 py-2 font-semibold">{t('ivr.history.days.col.date', 'Date')}</th>
								</tr>
							</thead>
							<tbody>
								{days.length === 0 ? (
									<tr>
										<td colSpan={3} className="px-3 py-2 text-gray-400 text-center">{t('ivr.history.days.empty', 'No days found')}</td>
									</tr>
								) : (
									days.map((d) => (
										<tr key={`${d.tournament_id}-${d.day_id}`} className={`cursor-pointer hover:bg-blue-900 ${selectedDayId===d.day_id ? 'bg-blue-900/60' : ''}`} onClick={() => { setSelectedDayId(d.day_id); setSelectedMatchId(null); }}>
											<td className="px-3 py-2 whitespace-nowrap">{d.tournament_name}</td>
											<td className="px-3 py-2 whitespace-nowrap">{t('ivr.history.day_n', 'Day {n}', { n: d.day_number })}</td>
											<td className="px-3 py-2 whitespace-nowrap">{new Date(d.date).toLocaleDateString()}</td>
										</tr>
									))
								)}
							</tbody>
						</table>
					</div>
				</div>

				{/* Matches */}
				<div className="theme-card p-6 shadow-lg">
					<h3 className="text-lg font-semibold text-blue-300 mb-3">{t('ivr.history.matches.title', 'Matches')}</h3>
					<div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
						<table className="min-w-full text-left text-sm text-gray-200">
							<thead className="theme-surface-2 sticky top-0 z-10">
								<tr>
									<th className="px-3 py-2 font-semibold">#</th>
									<th className="px-3 py-2 font-semibold">{t('ivr.history.matches.col.category', 'Category')}</th>
								</tr>
							</thead>
							<tbody>
								{matches.length === 0 ? (
									<tr>
										<td colSpan={2} className="px-3 py-2 text-gray-400 text-center">{t('ivr.history.matches.empty', 'No matches')}</td>
									</tr>
								) : (
									matches.map((m) => (
										<tr key={m.id} className={`cursor-pointer hover:bg-blue-900 ${selectedMatchId===m.id ? 'bg-blue-900/60' : ''}`} onClick={() => setSelectedMatchId(m.id)}>
											<td className="px-3 py-2 whitespace-nowrap">#{m.match_number ?? ''}</td>
											<td className="px-3 py-2 whitespace-nowrap">{m.category ?? ''}</td>
										</tr>
									))
								)}
							</tbody>
						</table>
					</div>
				</div>

				{/* Events */}
				<div className="theme-card p-6 shadow-lg">
					<h3 className="text-lg font-semibold text-blue-300 mb-3">{t('ivr.history.events.title', 'Events')}</h3>
					<div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
						<table className="min-w-full text-left text-sm text-gray-200">
							<thead className="theme-surface-2 sticky top-0 z-10">
								<tr>
									<th className="px-3 py-2 font-semibold">{t('ivr.history.events.col.rnd', 'RND')}</th>
									<th className="px-3 py-2 font-semibold">{t('ivr.history.events.col.time', 'Time')}</th>
									<th className="px-3 py-2 font-semibold">{t('ivr.history.events.col.code', 'Code')}</th>
								</tr>
							</thead>
							<tbody>
								{events.length === 0 ? (
									<tr>
										<td colSpan={3} className="px-3 py-2 text-gray-400 text-center">{t('ivr.history.events.empty', 'No events')}</td>
									</tr>
								) : (
									events.map((e) => (
										<tr key={e.id} className="cursor-pointer hover:bg-blue-900" onDoubleClick={async () => {
											try {
												const { invoke } = await import('@tauri-apps/api/core');
												const num = String(e.id).replace(/[^0-9]/g, '');
												if (num) { await invoke('ivr_open_event_video', { event_id: Number(num), eventId: Number(num) }); }
											} catch (err) { console.warn('Failed to open event video', err); }
										}}>
											<td className="px-3 py-2 whitespace-nowrap">{t('ivr.history.events.round_prefix', 'R')}{e.round ?? 1}</td>
											<td className="px-3 py-2 whitespace-nowrap">{e.time ?? ''}</td>
											<td className="px-3 py-2 whitespace-nowrap">{e.event_code ?? e.eventCode ?? ''}</td>
										</tr>
									))
								)}
							</tbody>
						</table>
					</div>
				</div>
			</div>

			{/* Recorded Videos Section */}
			<div className="theme-card p-6 shadow-lg">
				<div className="flex items-center justify-between mb-3">
					<h3 className="text-lg font-semibold text-blue-300">{t('ivr.history.videos.title', 'Recorded Videos')}</h3>
					<div className="flex gap-2">
						<Button variant="secondary" size="sm" disabled={selectedVideoIds.size===0} onClick={handleDeleteSelected}>{t('common.delete', 'Delete')}</Button>
						<Button variant="secondary" size="sm" disabled={selectedVideoIds.size===0} onClick={() => setDriveOpen('folder')}>{t('ivr.history.videos.upload_drive', 'Upload to Drive')}</Button>
						<Button variant="primary" size="sm" disabled={!selectedDayId || !selectedMatchId} onClick={() => setDriveOpen('zip')}>{t('common.import', 'Import')}</Button>
					</div>
				</div>

				<div className="max-h-56 overflow-y-auto border border-gray-700 rounded">
					<table className="min-w-full text-left text-sm text-gray-200">
						<thead className="theme-surface-2 sticky top-0 z-10">
							<tr>
								<th className="px-3 py-2 font-semibold">{t('ivr.history.videos.col.path', 'Path')}</th>
								<th className="px-3 py-2 font-semibold">{t('ivr.history.videos.col.start', 'Start')}</th>
								<th className="px-3 py-2 font-semibold">{t('common.actions', 'Actions')}</th>
							</tr>
						</thead>
						<tbody>
							{videos.length === 0 ? (
								<tr>
									<td colSpan={3} className="px-3 py-2 text-gray-400 text-center">{t('ivr.history.videos.empty', 'No videos')}</td>
								</tr>
							) : (
								videos.map((v) => (
									<tr key={v.id} className={`cursor-pointer hover:bg-blue-900 ${selectedVideoIds.has(v.id)?'bg-blue-900/40':''}`} onClick={() => toggleVideoSelection(v.id)} onDoubleClick={async () => {
										try {
											const { invoke } = await import('@tauri-apps/api/core');
											if (v.id) {
												await invoke('ivr_open_recorded_video', { recorded_video_id: v.id, recordedVideoId: v.id });
											} else if (v.file_path) {
												await invoke('ivr_open_video_path', { file_path: v.file_path, filePath: v.file_path, offset_seconds: 0, offsetSeconds: 0 });
											}
										} catch (e) { console.warn('Failed to open video', e); }} }>
										<td className="px-3 py-2 whitespace-nowrap truncate max-w-[28rem]">{v.file_path ?? v.record_directory ?? ''}</td>
										<td className="px-3 py-2 whitespace-nowrap">{new Date(v.start_time).toLocaleString()}</td>
										<td className="px-3 py-2 whitespace-nowrap">
											<Button variant="ghost" size="sm" className="text-blue-400 hover:text-blue-300 mr-2" onClick={async (e) => { e.stopPropagation(); if (v.id) { const { invoke } = await import('@tauri-apps/api/core'); await invoke('ivr_open_recorded_video', { recorded_video_id: v.id, recordedVideoId: v.id }); } }}>{t('common.open', 'Open')}</Button>
											<Button variant="ghost" size="sm" className="text-gray-300 hover:text-white" onClick={(e) => { e.stopPropagation(); if (selectedMatchId) { setPickerVideoId(v.id); setPickerMatchId(selectedMatchId); } }}>{t('ivr.history.pick_event', 'Pick Event')}</Button>
											{pickerVideoId===v.id && pickerMatchId && (
												<VideoEventPicker recordedVideoId={pickerVideoId!} matchId={pickerMatchId!} onClose={() => setPickerVideoId(null)} />
											)}
										</td>
									</tr>
								))
							)}
						</tbody>
					</table>
				</div>
			</div>

			{/* Drive Browser Modals */}
			<DriveBrowser
				isOpen={driveOpen==='zip'}
				mode='pick-zip'
				onClose={()=>setDriveOpen(null)}
				onPick={async (file) => {
					setDriveOpen(null);
					if (!selectedDayId || !selectedMatchId) return;
					try {
						const { invoke } = await import('@tauri-apps/api/core');
						const newJob = `job_${Date.now()}`; setJobId(newJob); setProgressMsg(t('ivr.history.import_starting', 'Starting import…'));
						const res: any = await invoke('ivr_import_recorded_videos', { source: 'drive', path_or_id: file.id, tournament_day_id: selectedDayId, match_id: selectedMatchId, job_id: newJob });
						if (res?.success === false) alert(res?.error || t('ivr.history.err_import', 'Import failed'));
						else {
							const v: any = await invoke('ivr_list_recorded_videos', { tournament_day_id: selectedDayId, match_id: selectedMatchId });
							setVideos(Array.isArray(v) ? v : (v?.data ?? []));
						}
					} catch (e: any) { alert(typeof e==='string'?e:(e?.message||t('ivr.history.err_import', 'Import failed'))); }
				}}
			/>

			<DriveBrowser
				isOpen={driveOpen==='folder'}
				mode='pick-folder'
				onClose={()=>setDriveOpen(null)}
				onPick={async (folder) => {
					setDriveOpen(null);
					try {
						const { invoke } = await import('@tauri-apps/api/core');
						const ids = Array.from(selectedVideoIds);
						const newJob = `job_${Date.now()}`; setJobId(newJob); setProgressMsg(t('ivr.history.upload_starting', 'Starting upload…'));
						const res: any = await invoke('ivr_upload_recorded_videos', { ids, folder_id: folder.id, job_id: newJob });
						if (res?.success === false) alert(res?.error || t('ivr.history.err_upload', 'Upload failed'));
						else setProgressMsg(t('ivr.history.upload_complete', 'Upload complete'));
					} catch (e: any) { alert(typeof e==='string'?e:(e?.message||'Upload failed')); }
				}}
			/>

			{progressMsg && <ProgressToast message={progressMsg} onCancel={jobId ? cancelJob : undefined} />}
		</div>
	);
};

export default IvrHistoryPanel;

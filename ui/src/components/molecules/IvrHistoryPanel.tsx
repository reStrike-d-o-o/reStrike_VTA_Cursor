import React, { useEffect, useState } from 'react';
import VideoEventPicker from './VideoEventPicker';
import Button from '../atoms/Button';

const IvrHistoryPanel: React.FC = () => {
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

	useEffect(() => {
		(async () => {
			try {
				setLoading(true); setError(null);
				const { invoke } = await import('@tauri-apps/api/core');
				const resp: any = await invoke('ivr_list_tournament_days');
				setDays(Array.isArray(resp) ? resp : (resp?.data ?? []));
			} catch (e: any) {
				setError(typeof e === 'string' ? e : (e?.message || 'Failed to load days'));
			} finally { setLoading(false); }
		})();
	}, []);

	useEffect(() => {
		(async () => {
			if (selectedDayId == null) { setMatches([]); setVideos([]); setSelectedVideoIds(new Set()); return; }
			try {
				setLoading(true); setError(null);
				const { invoke } = await import('@tauri-apps/api/core');
				const m: any = await invoke('ivr_list_matches_for_day', { dayId: selectedDayId });
				setMatches(Array.isArray(m) ? m : (m?.data ?? []));
				const v: any = await invoke('ivr_list_recorded_videos', { tournamentDayId: selectedDayId, matchId: selectedMatchId });
				setVideos(Array.isArray(v) ? v : (v?.data ?? []));
				setSelectedVideoIds(new Set());
			} catch (e: any) {
				setError(typeof e === 'string' ? e : (e?.message || 'Failed to load matches/videos'));
			} finally { setLoading(false); }
		})();
	}, [selectedDayId, selectedMatchId]);

	useEffect(() => {
		(async () => {
			if (selectedMatchId == null) { setEvents([]); return; }
			try {
				const { invoke } = await import('@tauri-apps/api/core');
				const ev: any = await invoke('pss_get_events_for_match', { matchId: String(selectedMatchId) });
				const list = Array.isArray(ev) ? ev : (ev?.data ?? []);
				setEvents(list);
			} catch (e) {
				setEvents([]);
			}
		})();
	}, [selectedMatchId]);

	const toggleVideoSelection = (id: number) => {
		setSelectedVideoIds(prev => {
			const next = new Set(prev);
			if (next.has(id)) next.delete(id); else next.add(id);
			return next;
		});
	};

	const handleDeleteSelected = async () => {
		if (selectedVideoIds.size === 0) return;
		if (!window.confirm('Delete selected videos and their records? This cannot be undone.')) return;
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			const ids = Array.from(selectedVideoIds);
			const res: any = await invoke('ivr_delete_recorded_videos', { ids });
			if (res?.success !== false) {
				// Refresh list
				if (selectedDayId != null) {
					const v: any = await invoke('ivr_list_recorded_videos', { tournamentDayId: selectedDayId, matchId: selectedMatchId });
					setVideos(Array.isArray(v) ? v : (v?.data ?? []));
					setSelectedVideoIds(new Set());
				}
			} else {
				alert(res?.error || 'Failed to delete');
			}
		} catch (e: any) {
			alert(typeof e === 'string' ? e : (e?.message || 'Failed to delete'));
		}
	};

	return (
		<div className="space-y-6">
			{/* Header */}
			<div className="flex items-center justify-between">
				<div>
					<h2 className="text-xl font-semibold text-white">Match history</h2>
					<p className="text-sm text-gray-400">Review recorded sessions by day, match and event</p>
				</div>
			</div>

			{/* Status */}
			{loading && <div className="text-xs text-gray-400">Loading…</div>}
			{error && (
				<div className="bg-red-900/20 border border-red-500/50 rounded-lg p-3">
					<span className="text-red-400 font-medium">Error</span>
					<p className="text-red-300 mt-1 text-sm">{error}</p>
				</div>
			)}

			{/* Main Columns */}
			<div className="grid grid-cols-3 gap-4">
				{/* Days */}
				<div className="theme-card p-6 shadow-lg">
					<h3 className="text-lg font-semibold text-blue-300 mb-3">Tournament / Day</h3>
					<div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
						<table className="min-w-full text-left text-sm text-gray-200">
							<thead className="theme-surface-2 sticky top-0 z-10">
								<tr>
									<th className="px-3 py-2 font-semibold">Tournament</th>
									<th className="px-3 py-2 font-semibold">Day</th>
									<th className="px-3 py-2 font-semibold">Date</th>
								</tr>
							</thead>
							<tbody>
								{days.length === 0 ? (
									<tr>
										<td colSpan={3} className="px-3 py-2 text-gray-400 text-center">No days found</td>
									</tr>
								) : (
									days.map((d) => (
										<tr key={`${d.tournament_id}-${d.day_id}`} className={`cursor-pointer hover:bg-blue-900 ${selectedDayId===d.day_id ? 'bg-blue-900/60' : ''}`} onClick={() => { setSelectedDayId(d.day_id); setSelectedMatchId(null); }}>
											<td className="px-3 py-2 whitespace-nowrap">{d.tournament_name}</td>
											<td className="px-3 py-2 whitespace-nowrap">Day {d.day_number}</td>
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
					<h3 className="text-lg font-semibold text-blue-300 mb-3">Matches</h3>
					<div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
						<table className="min-w-full text-left text-sm text-gray-200">
							<thead className="theme-surface-2 sticky top-0 z-10">
								<tr>
									<th className="px-3 py-2 font-semibold">#</th>
									<th className="px-3 py-2 font-semibold">Category</th>
								</tr>
							</thead>
							<tbody>
								{matches.length === 0 ? (
									<tr>
										<td colSpan={2} className="px-3 py-2 text-gray-400 text-center">No matches</td>
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
					<h3 className="text-lg font-semibold text-blue-300 mb-3">Events</h3>
					<div className="max-h-64 overflow-y-auto border border-gray-700 rounded">
						<table className="min-w-full text-left text-sm text-gray-200">
							<thead className="theme-surface-2 sticky top-0 z-10">
								<tr>
									<th className="px-3 py-2 font-semibold">RND</th>
									<th className="px-3 py-2 font-semibold">Time</th>
									<th className="px-3 py-2 font-semibold">Code</th>
								</tr>
							</thead>
							<tbody>
								{events.length === 0 ? (
									<tr>
										<td colSpan={3} className="px-3 py-2 text-gray-400 text-center">No events</td>
									</tr>
								) : (
									events.map((e) => (
										<tr key={e.id} className="cursor-pointer hover:bg-blue-900" onDoubleClick={async () => {
											try {
												const { invoke } = await import('@tauri-apps/api/core');
												const num = String(e.id).replace(/[^0-9]/g, '');
												if (num) { await invoke('ivr_open_event_video', { eventId: Number(num) }); }
											} catch (err) { console.warn('Failed to open event video', err); }
										}}>
											<td className="px-3 py-2 whitespace-nowrap">R{e.round ?? 1}</td>
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
					<h3 className="text-lg font-semibold text-blue-300">Recorded Videos</h3>
					<div className="flex gap-2">
						<Button variant="secondary" size="sm" disabled={selectedVideoIds.size===0} onClick={handleDeleteSelected}>Delete</Button>
						<Button variant="secondary" size="sm" disabled={selectedVideoIds.size===0} onClick={async () => {
							if (selectedVideoIds.size===0) return;
							try {
								const { invoke } = await import('@tauri-apps/api/core');
								const ids = Array.from(selectedVideoIds);
								const res: any = await invoke('ivr_upload_recorded_videos', { ids });
								if (res?.success === false) alert(res?.error || 'Upload failed');
								else alert(`Upload started. Drive file id: ${res?.data?.file_id ?? 'unknown'}`);
							} catch (e: any) { alert(typeof e==='string'?e:(e?.message||'Upload failed')); }
						}}>Upload to Drive</Button>
						<Button variant="primary" size="sm" disabled={!selectedDayId || !selectedMatchId} onClick={async () => {
							if (!selectedDayId || !selectedMatchId) return;
							const mode = window.prompt('Import from: local or drive? (type local/drive)', 'local');
							if (!mode) return;
							const key = mode.toLowerCase()==='drive' ? 'Enter Drive file id to import:' : 'Enter local zip file path to import:';
							const path = window.prompt(key);
							if (!path) return;
							try {
								const { invoke } = await import('@tauri-apps/api/core');
								const res: any = await invoke('ivr_import_recorded_videos', { source: mode.toLowerCase(), pathOrId: path, tournamentDayId: selectedDayId, matchId: selectedMatchId });
								if (res?.success === false) alert(res?.error || 'Import failed');
								else {
									alert('Import completed');
									const v: any = await invoke('ivr_list_recorded_videos', { tournamentDayId: selectedDayId, matchId: selectedMatchId });
									setVideos(Array.isArray(v) ? v : (v?.data ?? []));
								}
							} catch (e: any) { alert(typeof e==='string'?e:(e?.message||'Import failed')); }
						}}>Import</Button>
					</div>
				</div>

				<div className="max-h-56 overflow-y-auto border border-gray-700 rounded">
					<table className="min-w-full text-left text-sm text-gray-200">
						<thead className="theme-surface-2 sticky top-0 z-10">
							<tr>
								<th className="px-3 py-2 font-semibold">Path</th>
								<th className="px-3 py-2 font-semibold">Start</th>
								<th className="px-3 py-2 font-semibold">Actions</th>
							</tr>
						</thead>
						<tbody>
							{videos.length === 0 ? (
								<tr>
									<td colSpan={3} className="px-3 py-2 text-gray-400 text-center">No videos</td>
								</tr>
							) : (
								videos.map((v) => (
									<tr key={v.id} className={`cursor-pointer hover:bg-blue-900 ${selectedVideoIds.has(v.id)?'bg-blue-900/40':''}`} onClick={() => toggleVideoSelection(v.id)} onDoubleClick={async () => {
										try {
											const { invoke } = await import('@tauri-apps/api/core');
											if (v.id) {
												await invoke('ivr_open_recorded_video', { recordedVideoId: v.id });
											} else if (v.file_path) {
												await invoke('ivr_open_video_path', { filePath: v.file_path, offsetSeconds: 0 });
											}
										} catch (e) { console.warn('Failed to open video', e); }} }>
										<td className="px-3 py-2 whitespace-nowrap truncate max-w-[28rem]">{v.file_path ?? v.record_directory ?? ''}</td>
										<td className="px-3 py-2 whitespace-nowrap">{new Date(v.start_time).toLocaleString()}</td>
										<td className="px-3 py-2 whitespace-nowrap">
											<Button variant="ghost" size="sm" className="text-blue-400 hover:text-blue-300 mr-2" onClick={async (e) => { e.stopPropagation(); if (v.id) { const { invoke } = await import('@tauri-apps/api/core'); await invoke('ivr_open_recorded_video', { recordedVideoId: v.id }); } }}>Open</Button>
											<Button variant="ghost" size="sm" className="text-gray-300 hover:text-white" onClick={(e) => { e.stopPropagation(); if (selectedMatchId) { setPickerVideoId(v.id); setPickerMatchId(selectedMatchId); } }}>Pick Event</Button>
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
		</div>
	);
};

export default IvrHistoryPanel;

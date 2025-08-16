import React, { useEffect, useState } from 'react';
import VideoEventPicker from './VideoEventPicker';

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
		<div className="theme-card p-4 text-gray-200 shadow-lg accent-ivr">
			{loading && <div className="text-xs text-gray-400 mb-2">Loading…</div>}
			{error && <div className="text-xs text-red-400 mb-2">{error}</div>}
			<div className="grid grid-cols-3 gap-3 mb-3">
				<div className="bg-gray-800/60 rounded p-3 min-h-[200px]">
					<div className="text-sm font-semibold mb-2">Tournament / Day</div>
					<div className="space-y-1 max-h-64 overflow-auto pr-1">
						{days.map((d) => (
							<button key={`${d.tournament_id}-${d.day_id}`} className={`w-full text-left text-xs px-2 py-1 rounded ${selectedDayId===d.day_id?'bg-blue-700/60':'hover:bg-gray-700/40'}`} onClick={() => { setSelectedDayId(d.day_id); setSelectedMatchId(null); }}>
								<span className="font-semibold">{d.tournament_name}</span>
								<span className="ml-2 text-gray-400">Day {d.day_number}</span>
								<span className="ml-2 text-gray-500">{new Date(d.date).toLocaleDateString()}</span>
							</button>
						))}
					</div>
				</div>
				<div className="bg-gray-800/60 rounded p-3 min-h-[200px]">
					<div className="text-sm font-semibold mb-2">Matches</div>
					<div className="space-y-1 max-h-64 overflow-auto pr-1">
						{matches.map((m) => (
							<button key={m.id} className={`w-full text-left text-xs px-2 py-1 rounded ${selectedMatchId===m.id?'bg-blue-700/60':'hover:bg-gray-700/40'}`} onClick={() => setSelectedMatchId(m.id)}>
								<span className="font-semibold">#{m.match_number ?? ''}</span>
								<span className="ml-2 text-gray-400">{m.category ?? ''}</span>
							</button>
						))}
					</div>
				</div>
				<div className="bg-gray-800/60 rounded p-3 min-h-[200px]">
					<div className="text-sm font-semibold mb-2">Events</div>
					<div className="space-y-1 max-h-64 overflow-auto pr-1 text-xs">
						{events.map((e) => (
							<div key={e.id} className="flex items-center justify-between px-2 py-1 rounded hover:bg-gray-700/40 cursor-pointer" onDoubleClick={async () => {
								try {
									const { invoke } = await import('@tauri-apps/api/core');
									const num = String(e.id).replace(/[^0-9]/g, '');
									if (num) { await invoke('ivr_open_event_video', { eventId: Number(num) }); }
								} catch (err) { console.warn('Failed to open event video', err); }
							}}>
								<div className="truncate mr-2">R{e.round ?? 1} • {e.time ?? ''}</div>
								<div className="text-gray-200 ml-2 whitespace-nowrap font-semibold">{e.event_code ?? e.eventCode ?? ''}</div>
							</div>
						))}
					</div>
				</div>
			</div>
			<div className="bg-gray-800/60 rounded p-3 min-h-[140px] mb-3">
				<div className="text-sm font-semibold mb-2">Recorded Videos</div>
				<div className="space-y-1 max-h-48 overflow-auto pr-1 text-xs">
					{videos.map((v) => (
						<div key={v.id} className={`relative flex items-center justify-between px-2 py-1 rounded hover:bg-gray-700/40 cursor-pointer ${selectedVideoIds.has(v.id)?'bg-blue-700/40':''}`} onClick={() => toggleVideoSelection(v.id)} onDoubleClick={async () => {
							try {
								const { invoke } = await import('@tauri-apps/api/core');
								if (v.id) {
									await invoke('ivr_open_recorded_video', { recordedVideoId: v.id });
								} else if (v.file_path) {
									await invoke('ivr_open_video_path', { filePath: v.file_path, offsetSeconds: 0 });
								} else if (v.record_directory) {
									// If only directory known, do nothing for now (future: browse directory)
								}
							} catch (e) { console.warn('Failed to open video', e); }
						}}>
							<div className="truncate mr-2">{v.file_path ?? v.record_directory ?? ''}</div>
							<div className="text-gray-400 ml-2 whitespace-nowrap">{new Date(v.start_time).toLocaleString()}</div>
							{/* Event picker trigger */}
							<button className="ml-2 text-[10px] px-2 py-0.5 rounded bg-gray-700/60 hover:bg-gray-700 text-gray-300" title="Pick event" onClick={(e) => { e.stopPropagation(); if (selectedMatchId) { setPickerVideoId(v.id); setPickerMatchId(selectedMatchId); } }}>
								Events
							</button>
							{pickerVideoId===v.id && pickerMatchId && (
								<VideoEventPicker recordedVideoId={pickerVideoId!} matchId={pickerMatchId!} onClose={() => setPickerVideoId(null)} />
							)}
						</div>
					))}
				</div>
			</div>
			<div className="flex justify-end gap-2">
				<button className="px-3 py-1 bg-gray-700/70 rounded text-xs text-gray-300 disabled:opacity-50" disabled={selectedVideoIds.size===0} onClick={handleDeleteSelected}>
					Delete
				</button>
				<button className="px-3 py-1 bg-gray-700/70 rounded text-xs text-gray-300" disabled={selectedVideoIds.size===0} onClick={async () => {
					if (selectedVideoIds.size===0) return;
					try {
						const { invoke } = await import('@tauri-apps/api/core');
						const ids = Array.from(selectedVideoIds);
						const res: any = await invoke('ivr_upload_recorded_videos', { ids });
						if (res?.success === false) alert(res?.error || 'Upload failed');
						else alert(`Upload started. Drive file id: ${res?.data?.file_id ?? 'unknown'}`);
					} catch (e: any) { alert(typeof e==='string'?e:(e?.message||'Upload failed')); }
				}}>
					Upload to Drive
				</button>
				<button className="px-3 py-1 bg-gray-700/70 rounded text-xs text-gray-300" disabled={!selectedDayId || !selectedMatchId} onClick={async () => {
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
				}}>
					Import
				</button>
			</div>
		</div>
	);
};

export default IvrHistoryPanel;

import React, { useEffect, useState } from 'react';

interface VideoEventPickerProps {
	recordedVideoId: number;
	matchId: number;
	onClose: () => void;
}

const VideoEventPicker: React.FC<VideoEventPickerProps> = ({ recordedVideoId, matchId, onClose }) => {
	const [events, setEvents] = useState<Array<any>>([]);
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		(async () => {
			try {
				setLoading(true); setError(null);
				const { invoke } = await import('@tauri-apps/api/core');
				// Fetch events for this match; the backend will compute offsets from recorded_videos.start_time
				const ev: any[] = await invoke('pss_get_events_for_match', { matchId: String(matchId) });
				setEvents(Array.isArray(ev) ? ev : (ev?.data ?? []));
			} catch (e: any) {
				setError(typeof e==='string'?e:(e?.message||'Failed to load events'));
			} finally { setLoading(false); }
		})();
	}, [recordedVideoId, matchId]);

	return (
		<div className="absolute z-50 bg-gray-900 border border-gray-700 rounded shadow-xl w-80 max-h-96 overflow-auto">
			<div className="flex items-center justify-between p-2 border-b border-gray-700">
				<div className="text-sm font-semibold text-gray-200">Select event</div>
				<button className="text-xs text-gray-400 hover:text-gray-200" onClick={onClose}>✕</button>
			</div>
			{loading && <div className="p-3 text-xs text-gray-400">Loading…</div>}
			{error && <div className="p-3 text-xs text-red-400">{error}</div>}
			<div className="p-2 space-y-1">
				{events.map((e) => (
					<button key={e.id} className="w-full text-left px-2 py-1 rounded text-xs text-gray-200 hover:bg-gray-800/80" onClick={async () => {
						try {
							const { invoke } = await import('@tauri-apps/api/core');
							const num = String(e.id).replace(/[^0-9]/g, '');
							if (num) {
								await invoke('ivr_open_recorded_video', { recordedVideoId: recordedVideoId, eventId: Number(num) });
								onClose();
							}
						} catch (err) { console.warn('Failed to open event', err); }
					}}>
						<span className="font-semibold">R{e.round ?? 1}</span>
						<span className="ml-2 text-gray-400">{e.time ?? ''}</span>
						<span className="ml-2 text-gray-300">{e.event_code ?? e.eventCode ?? ''}</span>
					</button>
				))}
			</div>
		</div>
	);
};

export default VideoEventPicker;

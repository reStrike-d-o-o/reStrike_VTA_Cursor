import React, { useState, useEffect, useRef } from 'react';
import { formatDate, formatDateTime } from '../../utils/format';
import { invoke } from '@tauri-apps/api/core';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import Icon from '../atoms/Icon';
import { useI18n } from '../../i18n/index';
import { FlagImage } from '../../utils/flagUtils';

interface Tournament {
  id: number;
  name: string;
  duration_days: number;
  city: string;
  country: string;
  country_code?: string;
  logo_path?: string;
  status: 'pending' | 'active' | 'ended';
  start_date?: string;
  end_date?: string;
  created_at: string;
  updated_at: string;
  created?: number;
  updated?: number;
}

interface TournamentDayStats {
  day_id: number;
  day_number: number;
  date: string;
  status: 'pending' | 'active' | 'completed' | 'ended';
  total_matches: number;
  female_athletes: number;
  male_athletes: number;
}

interface TournamentChampion {
  category?: string | null;
  match_uuid: string;
  match_id: string;
  winner_color: string;
  winner_name?: string | null;
  winner_country_code?: string | null;
  blue_score: number;
  red_score: number;
}

interface TournamentDay {
  id: number;
  tournament_id: number;
  day_number: number;
  date: string;
  status: 'pending' | 'active' | 'completed' | 'ended';
  start_time?: string;
  end_time?: string;
  created_at: string;
  updated_at: string;
  created?: number;
  updated?: number;
}

interface TournamentOverview {
  tournament: Tournament;
  days: TournamentDay[];
  total_matches: number;
  total_events: number;
  total_scores: number;
  total_warnings: number;
  day_stats: TournamentDayStats[];
  champions: TournamentChampion[];
  active_day?: TournamentDay;
  completed_days: number;
  pending_days: number;
}

interface LocationVerification {
  verified: boolean;
  country_code?: string;
  display_name?: string;
  error?: string;
}

const TournamentManagementPanel: React.FC = () => {
	const { t } = useI18n();
	const [tournaments, setTournaments] = useState<Tournament[]>([]);
	const [selectedTournament, setSelectedTournament] = useState<Tournament | null>(null);
	const [tournamentDays, setTournamentDays] = useState<TournamentDay[]>([]);
	const [tournamentOverview, setTournamentOverview] = useState<TournamentOverview | null>(null);
	const [isLoading, setIsLoading] = useState(false);
	const [isLoadingOverview, setIsLoadingOverview] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const [statusFilter, setStatusFilter] = useState<'all' | 'pending' | 'active' | 'ended'>('all');
  // OVR scraped list and filters
  const [ovrTournaments, setOvrTournaments] = useState<any[]>([]);
  const [ovrQ, setOvrQ] = useState('');
  const [ovrCountry, setOvrCountry] = useState('');
  const [ovrFrom, setOvrFrom] = useState('');
  const [ovrTo, setOvrTo] = useState('');
  const [ovrProvider, setOvrProvider] = useState<string>('all');
  const [providers, setProviders] = useState<any[]>([]);
  // Scroll sync refs for OVR table (match Data Preview styling)
  const ovrHeaderScrollRef = useRef<HTMLDivElement | null>(null);
  const ovrBodyScrollRef = useRef<HTMLDivElement | null>(null);
  const onOvrHeaderScroll = () => {
    if (ovrBodyScrollRef.current && ovrHeaderScrollRef.current) {
      ovrBodyScrollRef.current.scrollLeft = ovrHeaderScrollRef.current.scrollLeft;
    }
  };
  const onOvrBodyScroll = () => {
    if (ovrHeaderScrollRef.current && ovrBodyScrollRef.current) {
      ovrHeaderScrollRef.current.scrollLeft = ovrBodyScrollRef.current.scrollLeft;
    }
  };
  
  // Form states
  const [showAddForm, setShowAddForm] = useState(false);
  const [showEditForm, setShowEditForm] = useState(false);
  const [showOverview, setShowOverview] = useState(false);
  const [formData, setFormData] = useState({
    name: '',
    duration_days: 1,
    city: '',
    country: '',
    country_code: '',
    start_date: '',
  });
  
  // Location verification
  const [locationVerification, setLocationVerification] = useState<LocationVerification>({ verified: false });
  const [isVerifyingLocation, setIsVerifyingLocation] = useState(false);
  
  // Logo upload
  const [logoFile, setLogoFile] = useState<File | null>(null);
  const [isUploadingLogo, setIsUploadingLogo] = useState(false);
  
  // Confirmation modals
  const [showStartDayModal, setShowStartDayModal] = useState(false);
  const [showEndDayModal, setShowEndDayModal] = useState(false);
  const [selectedDay, setSelectedDay] = useState<TournamentDay | null>(null);

  // Load tournaments on component mount
  useEffect(() => {
    loadTournaments();
    // Load providers for dynamic filter
    (async () => {
      try {
        const res: any = await invoke('ovr_get_providers');
        if (res?.success) setProviders(res.providers || []);
      } catch (_) {}
    })();
  }, []);

  // Load tournament days when a tournament is selected
  useEffect(() => {
    if (selectedTournament) {
      loadTournamentDays(selectedTournament.id);
    }
  }, [selectedTournament]);

  const loadTournaments = async () => {
    try {
      setIsLoading(true);
      setError(null);
      const result = await invoke('tournament_get_all');
      const data = result as any;
      if (data.success) {
        setTournaments(data.tournaments);
      } else {
        setError(data.error || 'Failed to load tournaments');
      }
    } catch (err) {
      setError(`Error loading tournaments: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  const loadOvrTournaments = async () => {
    try {
      const params: any = {
        provider_id: ovrProvider !== 'all' ? Number(ovrProvider) : null,
        q: ovrQ ? ovrQ : null,
        from: ovrFrom ? ovrFrom : null,
        to: ovrTo ? ovrTo : null,
        country: ovrCountry ? ovrCountry : null,
        limit: 1000,
        offset: 0,
      };
      const data: any = await invoke('ovr_list_tournaments', params);
      if (data?.success) setOvrTournaments(data.tournaments || []);
    } catch (_) {}
  };

  useEffect(() => {
    const handler = () => { loadOvrTournaments(); };
    window.addEventListener('ovr:refreshed', handler as any);
    return () => { window.removeEventListener('ovr:refreshed', handler as any); };
  }, [ovrQ, ovrCountry, ovrFrom, ovrTo, ovrProvider]);

  const loadTournamentDays = async (tournamentId: number) => {
    try {
      const result = await invoke('tournament_get_days', { tournamentId });
      const data = result as any;
      
      if (data.success) {
        setTournamentDays(data.days);
      } else {
        console.error('Failed to load tournament days:', data.error);
      }
    } catch (err) {
      console.error('Error loading tournament days:', err);
    }
  };

  const loadTournamentOverview = async (tournamentId: number) => {
    try {
      setIsLoadingOverview(true);
      setError(null);
      
      // Get tournament details
      const tournamentResult = await invoke('tournament_get', { tournamentId });
      const tournamentData = tournamentResult as any;
      
      if (!tournamentData.success) {
        throw new Error(tournamentData.error || 'Failed to load tournament');
      }
      
      const tournament = tournamentData.tournament;
      
      // Get tournament days
      const daysResult = await invoke('tournament_get_days', { tournamentId });
      const daysData = daysResult as any;
      
      if (!daysData.success) {
        throw new Error(daysData.error || 'Failed to load tournament days');
      }
      
      const days = daysData.days;
      
      // Get tournament statistics from PSS tables
      const statsResult = await invoke('get_tournament_statistics', { tournamentId });
      const statsData = statsResult as any;
      
      if (!statsData.success) {
        throw new Error(statsData.error || 'Failed to load tournament statistics');
      }
      
      const stats = statsData.statistics;
      
      // Calculate overview data
      const completedStatuses = ['completed', 'ended'];
      const completedDays = days.filter((day: TournamentDay) => completedStatuses.includes(day.status)).length;
      const pendingDays = days.filter((day: TournamentDay) => day.status === 'pending').length;
      const activeDay = days.find((day: TournamentDay) => day.status === 'active');
      
      const overview: TournamentOverview = {
        tournament,
        days,
        total_matches: stats.total_matches || 0,
        total_events: stats.total_events || 0,
        total_scores: stats.total_scores || 0,
        total_warnings: stats.total_warnings || 0,
        day_stats: stats.day_stats || [],
        champions: stats.champions || [],
        active_day: activeDay,
        completed_days: completedDays,
        pending_days: pendingDays,
      };
      
      setTournamentOverview(overview);
      setShowOverview(true);
    } catch (err) {
      setError(`Error loading tournament overview: ${err}`);
    } finally {
      setIsLoadingOverview(false);
    }
  };

  const verifyLocation = async () => {
    if (!formData.city || !formData.country) {
      setLocationVerification({ verified: false, error: 'Please enter both city and country' });
      return;
    }

    try {
      setIsVerifyingLocation(true);
      setLocationVerification({ verified: false });
      
      const result = await invoke('tournament_verify_location', {
        city: formData.city,
        country: formData.country
      });
      const data = result as any;
      
      if (data.success) {
        if (data.verified) {
          setLocationVerification({
            verified: true,
            country_code: data.country_code,
            display_name: data.display_name
          });
          setFormData(prev => ({ ...prev, country_code: data.country_code }));
        } else {
          // Location not found or network issue
          setLocationVerification({
            verified: false,
            error: 'Location not found or network unavailable. You can still proceed with manual entry.'
          });
        }
      } else {
        setLocationVerification({
          verified: false,
          error: data.error || 'Location verification failed'
        });
      }
    } catch (err) {
      setLocationVerification({
        verified: false,
        error: 'Network error. You can still proceed with manual entry.'
      });
    } finally {
      setIsVerifyingLocation(false);
    }
  };

  const handleLogoUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setLogoFile(file);
    }
  };

  const uploadLogo = async (tournamentId: number) => {
    if (!logoFile) return;

    try {
      setIsUploadingLogo(true);
      
      // Convert file to base64
      const reader = new FileReader();
      reader.onload = async () => {
        const base64 = reader.result as string;
        const logoPath = `tournament_logos/${tournamentId}_${Date.now()}.png`;
        
        const result = await invoke('tournament_update_logo', {
          tournamentId,
          logoPath
        });
        const data = result as any;
        
        if (data.success) {
          // Update the tournament in the list
          setTournaments(prev => prev.map(t => 
            t.id === tournamentId ? { ...t, logo_path: logoPath } : t
          ));
          setLogoFile(null);
        } else {
          setError(data.error || 'Failed to upload logo');
        }
      };
      reader.readAsDataURL(logoFile);
    } catch (err) {
      setError(`Error uploading logo: ${err}`);
    } finally {
      setIsUploadingLogo(false);
    }
  };

  const createTournament = async () => {
    try {
      setIsLoading(true);
      setError(null);
      
      const result = await invoke('tournament_create', {
        name: formData.name,
        durationDays: formData.duration_days,
        city: formData.city,
        country: formData.country,
        countryCode: formData.country_code || null,
        startDate: formData.start_date && formData.start_date.trim() !== '' ? convertToRFC3339(formData.start_date) : null
      });
      const data = result as any;
      
      if (data.success) {
        setShowAddForm(false);
        setFormData({
          name: '',
          duration_days: 1,
          city: '',
          country: '',
          country_code: '',
          start_date: '',
        });
        setLocationVerification({ verified: false });
        await loadTournaments();
      } else {
        setError(data.error || 'Failed to create tournament');
      }
    } catch (err) {
      setError(`Error creating tournament: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  const updateTournament = async () => {
    if (!selectedTournament) return;

    try {
      setIsLoading(true);
      setError(null);
      
      const result = await invoke('tournament_update', {
        tournamentId: selectedTournament.id,
        name: formData.name,
        durationDays: formData.duration_days,
        city: formData.city,
        country: formData.country,
        countryCode: formData.country_code || null,
        logoPath: selectedTournament.logo_path || null,
        status: selectedTournament.status,
        startDate: formData.start_date && formData.start_date.trim() !== '' ? convertToRFC3339(formData.start_date) : null,
        endDate: selectedTournament.end_date || null
      });
      const data = result as any;
      
      if (data.success) {
        setShowEditForm(false);
        setSelectedTournament(null);
        setFormData({
          name: '',
          duration_days: 1,
          city: '',
          country: '',
          country_code: '',
          start_date: '',
        });
        setLocationVerification({ verified: false });
        await loadTournaments();
      } else {
        setError(data.error || 'Failed to update tournament');
      }
    } catch (err) {
      setError(`Error updating tournament: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  const deleteTournament = async (tournamentId: number) => {
    if (!confirm('Are you sure you want to delete this tournament? This action cannot be undone.')) {
      return;
    }

    try {
      setIsLoading(true);
      setError(null);
      
      const result = await invoke('tournament_delete', { tournamentId });
      const data = result as any;
      
      if (data.success) {
        await loadTournaments();
        if (selectedTournament?.id === tournamentId) {
          setSelectedTournament(null);
          setTournamentDays([]);
        }
      } else {
        setError(data.error || 'Failed to delete tournament');
      }
    } catch (err) {
      setError(`Error deleting tournament: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  const startTournamentDay = async (dayId: number) => {
    try {
      setIsLoading(true);
      setError(null);
      
      const result = await invoke('tournament_start_day', { tournamentDayId: dayId });
      const data = result as any;
      
      if (data.success) {
        setShowStartDayModal(false);
        setSelectedDay(null);
        if (selectedTournament) {
          await loadTournamentDays(selectedTournament.id);
        }
      } else {
        setError(data.error || 'Failed to start tournament day');
      }
    } catch (err) {
      setError(`Error starting tournament day: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  const endTournamentDay = async (dayId: number) => {
    try {
      setIsLoading(true);
      setError(null);
      
      const result = await invoke('tournament_end_day', { tournamentDayId: dayId });
      const data = result as any;
      
      if (data.success) {
        setShowEndDayModal(false);
        setSelectedDay(null);
        if (selectedTournament) {
          await loadTournamentDays(selectedTournament.id);
        }
      } else {
        setError(data.error || 'Failed to end tournament day');
      }
    } catch (err) {
      setError(`Error ending tournament day: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  const openEditForm = (tournament: Tournament) => {
    setSelectedTournament(tournament);
    setFormData({
      name: tournament.name,
      duration_days: tournament.duration_days,
      city: tournament.city,
      country: tournament.country,
      country_code: tournament.country_code || '',
      start_date: tournament.start_date ? convertFromRFC3339(tournament.start_date) : '',
    });
    setLocationVerification({ verified: !!tournament.country_code });
    setShowEditForm(true);
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'pending': return 'bg-yellow-400';
      case 'active': return 'bg-green-400';
      case 'ended': return 'bg-red-400';
      default: return 'bg-gray-400';
    }
  };

  const getDayStatusColor = (status: string) => {
    switch (status) {
      case 'pending': return 'bg-yellow-400';
      case 'active': return 'bg-green-400';
      case 'completed': return 'bg-red-400';
      default: return 'bg-gray-400';
    }
  };

  // Date formatting helpers
  const formatDate = (iso?: string) => {
    if (!iso) return '';
    const d = new Date(iso);
    const dd = String(d.getDate()).padStart(2, '0');
    const mm = String(d.getMonth() + 1).padStart(2, '0');
    const yyyy = d.getFullYear();
    return `${dd}.${mm}.${yyyy}`;
  };
  const formatTime = (iso?: string) => {
    if (!iso) return '';
    const d = new Date(iso);
    const hh = String(d.getHours()).padStart(2, '0');
    const mi = String(d.getMinutes()).padStart(2, '0');
    return `${hh}:${mi}`;
  };
  const formatDateTime = (iso?: string) => {
    if (!iso) return '';
    return `${formatDate(iso)} ${formatTime(iso)}`;
  };

  const getStatusText = (status: string) => {
    switch (status) {
      case 'pending': return t('tournament.status.pending', 'Pending');
      case 'active': return t('tournament.status.active', 'Active');
      case 'ended': return t('tournament.status.ended', 'Ended');
      default: return status;
    }
  };

  // Helper function to convert datetime-local to RFC3339
  const convertToRFC3339 = (datetimeLocal: string): string => {
    if (!datetimeLocal || datetimeLocal.trim() === '') {
      return '';
    }
    
    // datetime-local format: "2024-01-15T10:30"
    // RFC3339 format: "2024-01-15T10:30:00Z"
    const date = new Date(datetimeLocal);
    return date.toISOString();
  };

  // Helper function to convert RFC3339 to datetime-local
  const convertFromRFC3339 = (rfc3339: string): string => {
    if (!rfc3339 || rfc3339.trim() === '') {
      return '';
    }
    
    try {
      const date = new Date(rfc3339);
      // Convert to local timezone and format for datetime-local input
      const year = date.getFullYear();
      const month = String(date.getMonth() + 1).padStart(2, '0');
      const day = String(date.getDate()).padStart(2, '0');
      const hours = String(date.getHours()).padStart(2, '0');
      const minutes = String(date.getMinutes()).padStart(2, '0');
      
      return `${year}-${month}-${day}T${hours}:${minutes}`;
    } catch (error) {
      console.error('Error converting RFC3339 to datetime-local:', error);
      return '';
    }
  };

  return (
    <div className="space-y-6">
      {/* Error Display */}
      {error && (
        <div className="p-4 bg-red-900/20 border border-red-600/30 rounded-lg text-red-300">
          {error}
        </div>
      )}

      {/* Tournament List */}
      <div className="theme-card p-6">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-semibold text-gray-100">{t('tournament.title', 'Tournaments')}</h3>
          <div className="flex items-center gap-2">
            <Button onClick={() => setShowAddForm(true)} disabled={isLoading} className="bg-blue-600 hover:bg-blue-700 text-white">{t('tournament.add', 'Add Tournament')}</Button>
            <Button onClick={() => loadTournaments()} disabled={isLoading} className="bg-gray-600 hover:bg-gray-700 text-white">{t('common.refresh','Refresh')}</Button>
          </div>
        </div>

        {/* Filters */}
        <div className="flex gap-2 mb-4">
          {(['all','pending','active','ended'] as const).map((s) => (
            <Button
              key={s}
              size="sm"
              className={`${statusFilter === s ? 'bg-blue-600' : 'bg-gray-600 hover:bg-gray-700'}`}
              onClick={() => setStatusFilter(s)}
            >
              {s === 'all' ? t('tournament.filters.all', 'All') : s === 'pending' ? t('tournament.filters.pending', 'Pending') : s === 'active' ? t('tournament.filters.active', 'Active') : t('tournament.filters.ended', 'Ended')}
            </Button>
          ))}
        </div>

        {isLoading ? (
          <div className="text-center py-8 text-gray-400">{t('tournament.loading', 'Loading tournaments...')}</div>
        ) : tournaments.length === 0 ? (
          <div className="text-center py-8 text-gray-400">{t('tournament.none', 'No tournaments found. Create your first tournament to get started.')}</div>
        ) : (
          <div className="space-y-3">
            {tournaments
              .filter(t => statusFilter === 'all' ? true : (t.status as any) === statusFilter)
              .map((tournament: any) => (
              <div
                key={tournament.id}
                className={`p-4 rounded-lg border transition-colors cursor-pointer ${
                  selectedTournament?.id === tournament.id
                    ? 'bg-blue-900/30 border-blue-600/50'
                    : 'bg-gray-700/30 border-gray-600/30 hover:bg-gray-700/50'
                }`}
                onClick={() => setSelectedTournament(tournament)}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    {tournament.logo_path && (
                      <img
                        src={tournament.logo_path}
                        alt="Tournament logo"
                        className="w-8 h-8 rounded"
                      />
                    )}
                    <div>
                      <h4 className="font-medium text-gray-100">{tournament.name}</h4>
                      <p className="text-sm text-gray-400">{tournament.city || ''}{tournament.city && tournament.country ? ', ' : ''}{tournament.country || ''}</p>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <StatusDot color={getStatusColor(tournament.status || 'pending')} />
                    <span className="text-sm text-gray-400">{getStatusText(tournament.status || 'pending')}</span>
                    <div className="flex space-x-1">
                      <Button
                        onClick={(e) => {
                          e.stopPropagation();
                          loadTournamentOverview(tournament.id);
                        }}
                        size="sm"
                        className="bg-purple-600 hover:bg-purple-700 text-white"
                        disabled={isLoadingOverview}
                      >
                        {isLoadingOverview ? t('common.loading', 'Loading...') : t('tournament.overview', 'Overview')}
                      </Button>
                      <Button
                        onClick={(e) => {
                          e.stopPropagation();
                          openEditForm(tournament);
                        }}
                        size="sm"
                        className="bg-gray-600 hover:bg-gray-700 text-white"
                      >
                        {t('common.edit', 'Edit')}
                      </Button>
                      <Button
                        onClick={(e) => {
                          e.stopPropagation();
                          deleteTournament(tournament.id);
                        }}
                        size="sm"
                        className="bg-red-600 hover:bg-red-700 text-white"
                      >
                        {t('common.delete', 'Delete')}
                      </Button>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* External OVR Tournaments (scraped) */}
      <div className="theme-card p-6">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-semibold text-gray-100">{t('ovr.scraped.title', 'External OVR Tournaments')}</h3>
          <div className="flex items-center gap-2 flex-wrap">
            <Input aria-label={t('common.search','Search')} placeholder={t('common.search','Search')} value={ovrQ} onChange={(e)=>setOvrQ(e.target.value)} />
            <Input aria-label={t('common.country','Country')} placeholder={t('common.country','Country')} value={ovrCountry} onChange={(e)=>setOvrCountry(e.target.value)} />
            <div className="flex items-center gap-2">
              <label className="text-xs text-gray-400" htmlFor="ovr-from">{t('common.from','From')}</label>
              <input id="ovr-from" type="date" aria-label={t('common.from','From')} className="text-xs bg-gray-900 border border-gray-700 rounded px-2 py-1 text-gray-200" value={ovrFrom} onChange={(e)=>setOvrFrom(e.target.value)} />
              <label className="text-xs text-gray-400" htmlFor="ovr-to">{t('common.to','To')}</label>
              <input id="ovr-to" type="date" aria-label={t('common.to','To')} className="text-xs bg-gray-900 border border-gray-700 rounded px-2 py-1 text-gray-200" value={ovrTo} onChange={(e)=>setOvrTo(e.target.value)} />
              <label className="text-xs text-gray-400" htmlFor="ovr-provider">{t('common.provider','Provider')}</label>
              <select id="ovr-provider" aria-label={t('common.provider','Provider')} className="text-xs bg-gray-900 border border-gray-700 rounded px-2 py-1 text-gray-200" value={ovrProvider} onChange={(e)=>setOvrProvider(e.target.value)}>
                <option value="all">{t('common.all','All providers')}</option>
                {providers.map((p: any) => (
                  <option key={p.id} value={String(p.id)}>{p.name}</option>
                ))}
              </select>
            </div>
            <Button onClick={loadOvrTournaments} className="bg-gray-600 hover:bg-gray-700 text-white">{t('common.refresh','Refresh')}</Button>
            <Button onClick={async ()=>{ try { const res: any = await invoke('ovr_clear_all_tournaments'); if (res?.success) { setOvrTournaments([]); try { window.dispatchEvent(new CustomEvent('ovr:refreshed')); } catch(_) {} } } catch(_) {} }} className="bg-red-600 hover:bg-red-700 text-white">{t('ovr.clear_all','Clear imported')}</Button>
          </div>
        </div>

        {ovrTournaments.length === 0 ? (
          <div className="text-center py-8 text-gray-400">{t('ovr.scraped.none','No external tournaments loaded yet. Use External sources to update.')}</div>
        ) : (
          <div className="w-full">
            <div className="w-[1360px] border border-gray-700 rounded overflow-hidden">
              <div className="overflow-x-auto" ref={ovrHeaderScrollRef} onScroll={onOvrHeaderScroll}>
                <table className="min-w-full table-fixed text-sm text-gray-200">
                  <thead className="bg-[#101820] sticky top-0 z-10">
                    <tr>
                      <th className="px-3 py-2 font-semibold whitespace-nowrap border-b border-gray-700 text-center w-[40%]">{t('common.name','Name')}</th>
                      <th className="px-3 py-2 font-semibold whitespace-nowrap border-b border-gray-700 text-center w-[12%]">{t('common.city','City')}</th>
                      <th className="px-3 py-2 font-semibold whitespace-nowrap border-b border-gray-700 text-center w-[12%]">{t('common.country','Country')}</th>
                      <th className="px-3 py-2 font-semibold whitespace-nowrap border-b border-gray-700 text-center w-[10%]">{t('common.start','Start')}</th>
                      <th className="px-3 py-2 font-semibold whitespace-nowrap border-b border-gray-700 text-center w-[10%]">{t('common.end','End')}</th>
                      <th className="px-3 py-2 font-semibold whitespace-nowrap border-b border-gray-700 text-center w-[8%]">URL</th>
                      <th className="px-3 py-2 font-semibold whitespace-nowrap border-b border-gray-700 text-center w-[8%]">{t('common.actions','Actions')}</th>
                    </tr>
                  </thead>
                </table>
              </div>
              <div className="max-h-64 overflow-y-auto overflow-x-auto" ref={ovrBodyScrollRef} onScroll={onOvrBodyScroll}>
                <table className="min-w-full table-fixed text-sm text-gray-200">
                  <tbody>
                    {ovrTournaments.map((ot: any) => (
                      <tr key={ot.id} className="hover:bg-blue-900/10">
                        <td className="px-3 py-2 whitespace-nowrap border-b border-gray-700/30 w-[40%]">{ot.name}</td>
                        <td className="px-3 py-2 whitespace-nowrap border-b border-gray-700/30 w-[12%]">{ot.city || ''}</td>
                        <td className="px-3 py-2 whitespace-nowrap border-b border-gray-700/30 w-[12%]">{ot.country || ''}</td>
                        <td className="px-3 py-2 whitespace-nowrap border-b border-gray-700/30 text-center w-[10%]">{ot.start_date ? formatDate(ot.start_date) : ''}</td>
                        <td className="px-3 py-2 whitespace-nowrap border-b border-gray-700/30 text-center w-[10%]">{ot.end_date ? formatDate(ot.end_date) : ''}</td>
                        <td className="px-3 py-2 whitespace-nowrap border-b border-gray-700/30 text-center w-[8%]">
                          {ot.url ? (
                            <a className="text-blue-400 hover:text-blue-300 underline" href={ot.url} target="_blank" rel="noreferrer">{t('common.open','Open')}</a>
                          ) : ''}
                        </td>
                        <td className="px-3 py-2 whitespace-nowrap border-b border-gray-700/30 text-center w-[8%]">
                          <Button size="sm" className="bg-green-600 hover:bg-green-700 text-white" onClick={()=>invoke('ovr_promote_tournament', { ovr_tournament_id: ot.id, local_name: ot.name })}>{t('ovr.promote','Promote')}</Button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Tournament Days */}
      {selectedTournament && (
        <div className="theme-card p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-100">{t('tournament.days.title', 'Tournament Days')} - {selectedTournament.name}</h3>
            <div className="flex gap-2">
              <Button
                size="sm"
                className="bg-green-600 hover:bg-green-700 text-white"
                onClick={async () => {
                  // Start tournament: find first pending day
                  const firstPending = tournamentDays.find(d => d.status === 'pending');
                  if (firstPending) {
                    setSelectedDay(firstPending);
                    setShowStartDayModal(true);
                  }
                }}
              >
                {t('tournament.start', 'Start Tournament')}
              </Button>
              <Button
                size="sm"
                className="bg-blue-600 hover:bg-blue-700 text-white"
                onClick={async () => {
                  // End tournament: find last active day, else last pending
                  const active = [...tournamentDays].reverse().find(d => d.status === 'active');
                  const lastPending = [...tournamentDays].reverse().find(d => d.status === 'pending');
                  const target = active || lastPending;
                  if (target) {
                    setSelectedDay(target);
                    setShowEndDayModal(true);
                  }
                }}
              >
                {t('tournament.end', 'End Tournament')}
              </Button>
            </div>
          </div>
          
          {tournamentDays.length === 0 ? (
            <div className="text-center py-8 text-gray-400">{t('tournament.days.none', 'No tournament days found.')}</div>
          ) : (
            <div className="space-y-3">
              {tournamentDays.map((day) => (
                <div
                  key={day.id}
                  className="p-4 rounded-lg border border-gray-600/30 bg-gray-700/30"
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="font-medium text-gray-100">{t('tournament.day_label', 'Day')} {day.day_number}</h4>
                      <p className="text-sm text-gray-400">
                        {formatDate(day.date)}
                        {day.start_time && ` • ${t('started_at', 'started at')}: ${formatDateTime(day.start_time)}`}
                        {day.end_time && ` • ${t('ended_at', 'ended at')}: ${formatDateTime(day.end_time)}`}
                      </p>
                    </div>
                    <div className="flex items-center space-x-2">
                      <StatusDot color={getDayStatusColor(day.status)} />
                      <span className="text-sm text-gray-400 capitalize">{day.status === 'pending' ? t('tournament.day_status.pending', 'Pending') : day.status === 'active' ? t('tournament.day_status.active', 'Active') : t('tournament.day_status.completed', 'Completed')}</span>
                      {day.status === 'pending' && (
                        <Button
                          onClick={() => {
                            setSelectedDay(day);
                            setShowStartDayModal(true);
                          }}
                          size="sm"
                          className="bg-green-600 hover:bg-green-700 text-white"
                        >
                          {t('start_day_button', 'Start Day')}
                        </Button>
                      )}
                      {day.status === 'active' && (
                        <Button
                          onClick={() => {
                            setSelectedDay(day);
                            setShowEndDayModal(true);
                          }}
                          size="sm"
                          className="bg-blue-600 hover:bg-blue-700 text-white"
                        >
                          {t('end_day_button', 'End Day')}
                        </Button>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Tournament Overview Modal */}
      {showOverview && tournamentOverview && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
          <div className="theme-card shadow-xl w-full max-w-4xl max-h-[90vh] flex flex-col overflow-hidden">
            <div className="flex flex-wrap items-start justify-between gap-4 px-6 pt-6">
              <div className="flex items-start gap-4 flex-wrap">
                {tournamentOverview.tournament.logo_path && (
                  <img
                    src={tournamentOverview.tournament.logo_path}
                    alt={t('tournament.logo_alt', 'Tournament logo')}
                    className="w-16 h-16 rounded-lg"
                  />
                )}
                <div>
                  <h3 className="text-2xl font-bold text-gray-100">{tournamentOverview.tournament.name}</h3>
                  <p className="text-gray-400">
                    {tournamentOverview.tournament.city}, {tournamentOverview.tournament.country}
                  </p>
                </div>
              </div>
              <Button
                onClick={() => setShowOverview(false)}
                className="bg-gray-600 hover:bg-gray-700 text-white"
              >
                {t('common.close', 'Close')}
              </Button>
            </div>
            
            <div className="flex-1 overflow-y-auto px-6 pb-6 space-y-6">
              {/* Tournament Statistics */}
              <div className="theme-surface-2 rounded-lg p-6">
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                  <div className="bg-gray-700/30 rounded-lg p-4 text-center">
                    <div className="text-2xl font-bold text-blue-400">{tournamentOverview.total_matches}</div>
                    <div className="text-sm text-gray-400">{t('tournament.stats.total_matches', 'Total Matches')}</div>
                  </div>
                <div className="bg-gray-700/30 rounded-lg p-4 text-center">
                  <div className="text-2xl font-bold text-green-400">{tournamentOverview.total_events}</div>
                  <div className="text-sm text-gray-400">{t('tournament.stats.total_events', 'Total Events')}</div>
                </div>
                <div className="bg-gray-700/30 rounded-lg p-4 text-center">
                  <div className="text-2xl font-bold text-yellow-400">{tournamentOverview.total_scores}</div>
                  <div className="text-sm text-gray-400">{t('tournament.stats.total_scores', 'Total Scores')}</div>
                </div>
                <div className="bg-gray-700/30 rounded-lg p-4 text-center">
                  <div className="text-2xl font-bold text-red-400">{tournamentOverview.total_warnings}</div>
                  <div className="text-sm text-gray-400">{t('tournament.stats.total_warnings', 'Total Warnings')}</div>
                </div>
              </div>
            </div>
            
            {/* Tournament Days Overview */}
            <div className="theme-surface-2 rounded-lg p-6">
              <div className="grid grid-cols-3 gap-4 mb-4">
                <div className="text-center">
                  <div className="text-2xl font-bold text-yellow-400">{tournamentOverview.pending_days}</div>
                  <div className="text-sm text-gray-400">{t('tournament.days.pending', 'Pending Days')}</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-green-400">
                    {tournamentOverview.active_day ? 1 : 0}
                  </div>
                  <div className="text-sm text-gray-400">{t('tournament.days.active', 'Active Day')}</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-red-400">{tournamentOverview.completed_days}</div>
                  <div className="text-sm text-gray-400">{t('tournament.days.completed', 'Completed Days')}</div>
                </div>
              </div>
              
              <div className="space-y-4">
                {tournamentOverview.days.map((day) => {
                  const dayStats =
                    tournamentOverview.day_stats?.find((stat) => stat.day_number === day.day_number) || {
                      total_matches: 0,
                      female_athletes: 0,
                      male_athletes: 0,
                    };
                  const totalAthletes = dayStats.female_athletes + dayStats.male_athletes;
                  return (
                    <div key={day.id} className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                      <div className="bg-gray-700/30 rounded-lg p-4 text-center lg:text-left">
                        <div className="text-2xl font-bold text-green-400">
                          {t('tournament.day.label', 'Day {n}', { n: day.day_number })}
                        </div>
                        <div className="text-sm text-gray-400">{formatDate(day.date)}</div>
                      </div>
                      <div className="bg-gray-700/30 rounded-lg p-4 text-center">
                        <div className="text-2xl font-bold text-blue-400">{totalAthletes}</div>
                        <div className="text-sm text-gray-400">
                          {t('tournament.day.total_athletes', 'Total Athletes')}
                        </div>
                      </div>
                      <div className="bg-gray-700/30 rounded-lg p-4 text-center">
                        <div className="text-2xl font-bold text-pink-400">{dayStats.female_athletes}</div>
                        <div className="text-sm text-gray-400">
                          {t('tournament.day.female', 'Female Athletes')}
                        </div>
                      </div>
                      <div className="bg-gray-700/30 rounded-lg p-4 text-center">
                        <div className="text-2xl font-bold text-cyan-400">{dayStats.male_athletes}</div>
                        <div className="text-sm text-gray-400">
                          {t('tournament.day.male', 'Male Athletes')}
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
            
            {/* Champions Table */}
            <div className="theme-surface-2 rounded-lg p-0">
              {tournamentOverview.champions.length > 0 ? (
                <div className="max-h-80 overflow-y-auto">
                  <table className="min-w-full divide-y divide-gray-700">
                    <thead className="sticky top-0 z-10 bg-gray-800/80 backdrop-blur">
                      <tr>
                        <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-gray-300">
                          {t('tournament.champion.category', 'Category')}
                        </th>
                        <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-gray-300">
                          {t('tournament.champion.match', 'Match')}
                        </th>
                        <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-gray-300">
                          {t('tournament.champion.score', 'Score')}
                        </th>
                        <th className="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-gray-300">
                          {t('tournament.champion.nation', 'Nation')}
                        </th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-800">
                      {tournamentOverview.champions.map((champ) => {
                        const flagCode = champ.winner_country_code?.toUpperCase();
                        return (
                          <tr
                            key={champ.match_uuid}
                            className="hover:bg-gray-800/40 transition-colors"
                          >
                            <td className="px-4 py-3 text-sm text-gray-100">
                              {champ.category || t('tournament.unknown_category', 'Unknown Category')}
                            </td>
                            <td className="px-4 py-3 text-sm text-gray-300">
                              <div className="flex flex-col">
                                <span className="font-semibold text-gray-100">
                                  {champ.winner_name || t('common.unknown', 'Unknown')}
                                </span>
                                <span className="text-xs text-gray-400">
                                  {t('tournament.match_label', 'Match {id}', { id: champ.match_id })}
                                </span>
                              </div>
                            </td>
                            <td className="px-4 py-3 text-gray-100">
                              <span className="text-2xl font-semibold text-blue-300">{champ.blue_score}</span>
                              <span className="mx-2 text-xl text-gray-400 font-semibold">:</span>
                              <span className="text-2xl font-semibold text-red-300">{champ.red_score}</span>
                            </td>
                            <td className="px-4 py-3">
                              {flagCode ? (
                                <FlagImage
                                  countryCode={flagCode}
                                  className="w-10 h-6 rounded-md shadow-md ring-1 ring-white/30"
                                />
                              ) : (
                                <span className="text-xs text-gray-500">
                                  {t('tournament.champion.no_flag', 'No flag')}
                                </span>
                              )}
                            </td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              ) : (
                <div className="p-6 text-sm text-gray-400">
                  {t('tournament.champion.none', 'No champions recorded yet.')}
                </div>
              )}
            </div>
          </div>
          </div>
        </div>
      )}

      {/* Add Tournament Form */}
      {showAddForm && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="theme-card shadow-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold text-gray-100 mb-4">{t('tournament.add_new', 'Add New Tournament')}</h3>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="name">{t('tournament.form.name', 'Tournament Name')}</Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                  placeholder={t('tournament.form.name_ph', 'Enter tournament name')}
                />
              </div>
              
              <div>
                <Label htmlFor="duration">{t('tournament.form.duration', 'Duration (days)')}</Label>
                <Input
                  id="duration"
                  type="number"
                  min="1"
                  value={formData.duration_days}
                  onChange={(e) => setFormData(prev => ({ ...prev, duration_days: parseInt(e.target.value) || 1 }))}
                />
              </div>
              
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="city">{t('tournament.form.city', 'City')}</Label>
                  <Input
                    id="city"
                    value={formData.city}
                    onChange={(e) => setFormData(prev => ({ ...prev, city: e.target.value }))}
                    placeholder={t('tournament.form.city_ph', 'Enter city')}
                  />
                </div>
                <div>
                  <Label htmlFor="country">{t('tournament.form.country', 'Country')}</Label>
                  <Input
                    id="country"
                    value={formData.country}
                    onChange={(e) => setFormData(prev => ({ ...prev, country: e.target.value }))}
                    placeholder={t('tournament.form.country_ph', 'Enter country')}
                  />
                </div>
              </div>
              
              <div className="flex items-center space-x-2">
                <Button
                  onClick={verifyLocation}
                  disabled={isVerifyingLocation || !formData.city || !formData.country}
                  size="sm"
                  className="bg-blue-600 hover:bg-blue-700 text-white"
                >
                  {isVerifyingLocation ? t('tournament.location.verifying', 'Verifying...') : t('tournament.location.verify', 'Verify Location')}
                </Button>
                {locationVerification.verified && (
                  <div className="flex items-center space-x-1 text-green-400">
                    <Icon name="✅" size="text-sm" />
                    <span className="text-sm">{t('tournament.location.verified', 'Verified')}</span>
                  </div>
                )}
                {locationVerification.error && (
                  <div className="flex items-center space-x-1 text-red-400">
                    <Icon name="❌" size="text-sm" />
                    <span className="text-sm">{locationVerification.error}</span>
                  </div>
                )}
              </div>
              
              <div>
                <Label htmlFor="start_date">{t('tournament.form.start_date', 'Start Date (Optional)')}</Label>
                <Input
                  id="start_date"
                  type="datetime-local"
                  value={formData.start_date}
                  onChange={(e) => setFormData(prev => ({ ...prev, start_date: e.target.value }))}
                />
              </div>
              
              <div>
                <Label htmlFor="logo">{t('tournament.form.logo', 'Tournament Logo (Optional)')}</Label>
                <Input
                  id="logo"
                  type="file"
                  accept="image/*"
                  onChange={handleLogoUpload}
                />
              </div>
            </div>
            
            <div className="flex justify-end space-x-3 mt-6">
              <Button
                onClick={() => setShowAddForm(false)}
                className="bg-gray-600 hover:bg-gray-700 text-white"
              >
                {t('common.cancel', 'Cancel')}
              </Button>
              <Button
                onClick={createTournament}
                disabled={isLoading || !formData.name || !formData.city || !formData.country}
                className="bg-blue-600 hover:bg-blue-700 text-white"
              >
                {isLoading ? t('common.creating', 'Creating...') : t('tournament.create', 'Create Tournament')}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Edit Tournament Form */}
      {showEditForm && selectedTournament && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="theme-card shadow-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold text-gray-100 mb-4">{t('tournament.edit', 'Edit Tournament')}</h3>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="edit-name">{t('tournament.form.name', 'Tournament Name')}</Label>
                <Input
                  id="edit-name"
                  value={formData.name}
                  onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                  placeholder={t('tournament.form.name_ph', 'Enter tournament name')}
                />
              </div>
              
              <div>
                <Label htmlFor="edit-duration">{t('tournament.form.duration', 'Duration (days)')}</Label>
                <Input
                  id="edit-duration"
                  type="number"
                  min="1"
                  value={formData.duration_days}
                  onChange={(e) => setFormData(prev => ({ ...prev, duration_days: parseInt(e.target.value) || 1 }))}
                />
              </div>
              
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="edit-city">{t('tournament.form.city', 'City')}</Label>
                  <Input
                    id="edit-city"
                    value={formData.city}
                    onChange={(e) => setFormData(prev => ({ ...prev, city: e.target.value }))}
                    placeholder={t('tournament.form.city_ph', 'Enter city')}
                  />
                </div>
                <div>
                  <Label htmlFor="edit-country">{t('tournament.form.country', 'Country')}</Label>
                  <Input
                    id="edit-country"
                    value={formData.country}
                    onChange={(e) => setFormData(prev => ({ ...prev, country: e.target.value }))}
                    placeholder={t('tournament.form.country_ph', 'Enter country')}
                  />
                </div>
              </div>
              
              <div className="flex items-center space-x-2">
                <Button
                  onClick={verifyLocation}
                  disabled={isVerifyingLocation || !formData.city || !formData.country}
                  size="sm"
                  className="bg-blue-600 hover:bg-blue-700 text-white"
                >
                  {isVerifyingLocation ? t('tournament.location.verifying', 'Verifying...') : t('tournament.location.verify', 'Verify Location')}
                </Button>
                {locationVerification.verified && (
                  <div className="flex items-center space-x-1 text-green-400">
                    <Icon name="✅" size="text-sm" />
                    <span className="text-sm">{t('tournament.location.verified', 'Verified')}</span>
                  </div>
                )}
                {locationVerification.error && (
                  <div className="flex items-center space-x-1 text-red-400">
                    <Icon name="❌" size="text-sm" />
                    <span className="text-sm">{locationVerification.error}</span>
                  </div>
                )}
              </div>
              
              <div>
                <Label htmlFor="edit-start_date">{t('tournament.form.start_date', 'Start Date (Optional)')}</Label>
                <Input
                  id="edit-start_date"
                  type="datetime-local"
                  value={formData.start_date}
                  onChange={(e) => setFormData(prev => ({ ...prev, start_date: e.target.value }))}
                />
              </div>
              
              <div>
                <Label htmlFor="edit-logo">{t('tournament.form.logo', 'Tournament Logo (Optional)')}</Label>
                <Input
                  id="edit-logo"
                  type="file"
                  accept="image/*"
                  onChange={handleLogoUpload}
                />
                {logoFile && (
                  <Button
                    onClick={() => uploadLogo(selectedTournament.id)}
                    disabled={isUploadingLogo}
                    size="sm"
                    className="mt-2 bg-green-600 hover:bg-green-700 text-white"
                  >
                    {isUploadingLogo ? t('common.uploading', 'Uploading...') : t('tournament.upload_logo', 'Upload Logo')}
                  </Button>
                )}
              </div>
            </div>
            
            <div className="flex justify-end space-x-3 mt-6">
              <Button
                onClick={() => setShowEditForm(false)}
                className="bg-gray-600 hover:bg-gray-700 text-white"
              >
                {t('common.cancel', 'Cancel')}
              </Button>
              <Button
                onClick={updateTournament}
                disabled={isLoading || !formData.name || !formData.city || !formData.country}
                className="bg-blue-600 hover:bg-blue-700 text-white"
              >
                {isLoading ? t('common.updating', 'Updating...') : t('tournament.update', 'Update Tournament')}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Start Day Confirmation Modal */}
      {showStartDayModal && selectedDay && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="theme-card shadow-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold text-gray-100 mb-4">{t('tournament.start_day.title', 'Start Tournament Day')}</h3>
            <p className="text-gray-300 mb-6">
              {t('tournament.start_day.confirm', "Are you sure you want to start Day {n}? This will automatically start the tournament if it's the first day.", { n: selectedDay.day_number })}
            </p>
            <div className="flex justify-end space-x-3">
              <Button
                onClick={() => setShowStartDayModal(false)}
                className="bg-gray-600 hover:bg-gray-700 text-white"
              >
                {t('cancel_button')}
              </Button>
              <Button
                onClick={() => startTournamentDay(selectedDay.id)}
                disabled={isLoading}
                className="bg-green-600 hover:bg-green-700 text-white"
              >
                {isLoading ? t('tournament.starting', 'Starting...') : t('tournament.start_day', 'Start Day')}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* End Day Confirmation Modal */}
      {showEndDayModal && selectedDay && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="theme-card shadow-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold text-gray-100 mb-4">{t('tournament.end_day.title', 'End Tournament Day')}</h3>
            <p className="text-gray-300 mb-6">
              {t('tournament.end_day.confirm', "Are you sure you want to end Day {n}? This will automatically end the tournament if it's the final day.", { n: selectedDay.day_number })}
            </p>
            <div className="flex justify-end space-x-3">
              <Button
                onClick={() => setShowEndDayModal(false)}
                className="bg-gray-600 hover:bg-gray-700 text-white"
              >
                {t('cancel_button')}
              </Button>
              <Button
                onClick={() => endTournamentDay(selectedDay.id)}
                disabled={isLoading}
                className="bg-blue-600 hover:bg-blue-700 text-white"
              >
                {isLoading ? t('tournament.ending', 'Ending...') : t('tournament.end_day', 'End Day')}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default TournamentManagementPanel;

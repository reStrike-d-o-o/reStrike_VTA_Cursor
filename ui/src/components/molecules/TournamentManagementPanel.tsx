import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import Icon from '../atoms/Icon';

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
}

interface TournamentDay {
  id: number;
  tournament_id: number;
  day_number: number;
  date: string;
  status: 'pending' | 'active' | 'completed';
  start_time?: string;
  end_time?: string;
  created_at: string;
  updated_at: string;
}

interface TournamentOverview {
  tournament: Tournament;
  days: TournamentDay[];
  total_matches: number;
  total_events: number;
  total_scores: number;
  total_warnings: number;
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
  const [tournaments, setTournaments] = useState<Tournament[]>([]);
  const [selectedTournament, setSelectedTournament] = useState<Tournament | null>(null);
  const [tournamentDays, setTournamentDays] = useState<TournamentDay[]>([]);
  const [tournamentOverview, setTournamentOverview] = useState<TournamentOverview | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isLoadingOverview, setIsLoadingOverview] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
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
      const data = JSON.parse(result as string);
      
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

  const loadTournamentDays = async (tournamentId: number) => {
    try {
      const result = await invoke('tournament_get_days', { tournamentId });
      const data = JSON.parse(result as string);
      
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
      const tournamentData = JSON.parse(tournamentResult as string);
      
      if (!tournamentData.success) {
        throw new Error(tournamentData.error || 'Failed to load tournament');
      }
      
      const tournament = tournamentData.tournament;
      
      // Get tournament days
      const daysResult = await invoke('tournament_get_days', { tournamentId });
      const daysData = JSON.parse(daysResult as string);
      
      if (!daysData.success) {
        throw new Error(daysData.error || 'Failed to load tournament days');
      }
      
      const days = daysData.days;
      
      // Get tournament statistics from PSS tables
      const statsResult = await invoke('get_tournament_statistics', { tournamentId });
      const statsData = JSON.parse(statsResult as string);
      
      if (!statsData.success) {
        throw new Error(statsData.error || 'Failed to load tournament statistics');
      }
      
      const stats = statsData.statistics;
      
      // Calculate overview data
      const completedDays = days.filter((day: TournamentDay) => day.status === 'completed').length;
      const pendingDays = days.filter((day: TournamentDay) => day.status === 'pending').length;
      const activeDay = days.find((day: TournamentDay) => day.status === 'active');
      
      const overview: TournamentOverview = {
        tournament,
        days,
        total_matches: stats.total_matches || 0,
        total_events: stats.total_events || 0,
        total_scores: stats.total_scores || 0,
        total_warnings: stats.total_warnings || 0,
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
      const data = JSON.parse(result as string);
      
      if (data.success) {
        setLocationVerification({
          verified: true,
          country_code: data.country_code,
          display_name: data.display_name
        });
        setFormData(prev => ({ ...prev, country_code: data.country_code }));
      } else {
        setLocationVerification({
          verified: false,
          error: data.error || 'Location verification failed'
        });
      }
    } catch (err) {
      setLocationVerification({
        verified: false,
        error: `Verification error: ${err}`
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
        const data = JSON.parse(result as string);
        
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
        startDate: formData.start_date || null
      });
      const data = JSON.parse(result as string);
      
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
        startDate: formData.start_date || null,
        endDate: selectedTournament.end_date || null
      });
      const data = JSON.parse(result as string);
      
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
      const data = JSON.parse(result as string);
      
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
      const data = JSON.parse(result as string);
      
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
      const data = JSON.parse(result as string);
      
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
      start_date: tournament.start_date || '',
    });
    setLocationVerification({ verified: !!tournament.country_code });
    setShowEditForm(true);
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'pending': return 'bg-yellow-400';
      case 'active': return 'bg-green-400';
      case 'ended': return 'bg-blue-400';
      default: return 'bg-gray-400';
    }
  };

  const getDayStatusColor = (status: string) => {
    switch (status) {
      case 'pending': return 'bg-gray-400';
      case 'active': return 'bg-green-400';
      case 'completed': return 'bg-blue-400';
      default: return 'bg-gray-400';
    }
  };

  const getStatusText = (status: string) => {
    switch (status) {
      case 'pending': return 'Pending';
      case 'active': return 'Active';
      case 'ended': return 'Ended';
      default: return status;
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
      <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg p-6">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-semibold text-gray-100">Tournaments</h3>
          <Button
            onClick={() => setShowAddForm(true)}
            disabled={isLoading}
            className="bg-blue-600 hover:bg-blue-700 text-white"
          >
            Add Tournament
          </Button>
        </div>

        {isLoading ? (
          <div className="text-center py-8 text-gray-400">Loading tournaments...</div>
        ) : tournaments.length === 0 ? (
          <div className="text-center py-8 text-gray-400">No tournaments found. Create your first tournament to get started.</div>
        ) : (
          <div className="space-y-3">
            {tournaments.map((tournament) => (
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
                      <p className="text-sm text-gray-400">
                        {tournament.city}, {tournament.country} • {tournament.duration_days} day{tournament.duration_days !== 1 ? 's' : ''}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <StatusDot color={getStatusColor(tournament.status)} />
                    <span className="text-sm text-gray-400">{getStatusText(tournament.status)}</span>
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
                        {isLoadingOverview ? 'Loading...' : 'Overview'}
                      </Button>
                      <Button
                        onClick={(e) => {
                          e.stopPropagation();
                          openEditForm(tournament);
                        }}
                        size="sm"
                        className="bg-gray-600 hover:bg-gray-700 text-white"
                      >
                        Edit
                      </Button>
                      <Button
                        onClick={(e) => {
                          e.stopPropagation();
                          deleteTournament(tournament.id);
                        }}
                        size="sm"
                        className="bg-red-600 hover:bg-red-700 text-white"
                      >
                        Delete
                      </Button>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Tournament Days */}
      {selectedTournament && (
        <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-lg p-6">
          <h3 className="text-lg font-semibold text-gray-100 mb-4">
            Tournament Days - {selectedTournament.name}
          </h3>
          
          {tournamentDays.length === 0 ? (
            <div className="text-center py-8 text-gray-400">No tournament days found.</div>
          ) : (
            <div className="space-y-3">
              {tournamentDays.map((day) => (
                <div
                  key={day.id}
                  className="p-4 rounded-lg border border-gray-600/30 bg-gray-700/30"
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="font-medium text-gray-100">Day {day.day_number}</h4>
                      <p className="text-sm text-gray-400">
                        {new Date(day.date).toLocaleDateString()}
                        {day.start_time && ` • Started: ${new Date(day.start_time).toLocaleTimeString()}`}
                        {day.end_time && ` • Ended: ${new Date(day.end_time).toLocaleTimeString()}`}
                      </p>
                    </div>
                    <div className="flex items-center space-x-2">
                      <StatusDot color={getDayStatusColor(day.status)} />
                      <span className="text-sm text-gray-400 capitalize">{day.status}</span>
                      {day.status === 'pending' && (
                        <Button
                          onClick={() => {
                            setSelectedDay(day);
                            setShowStartDayModal(true);
                          }}
                          size="sm"
                          className="bg-green-600 hover:bg-green-700 text-white"
                        >
                          Start Day
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
                          End Day
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
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-gradient-to-br from-gray-800/90 to-gray-900/95 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-xl p-6 w-full max-w-4xl max-h-[90vh] overflow-y-auto">
            <div className="flex justify-between items-center mb-6">
              <h3 className="text-xl font-semibold text-gray-100">Tournament Overview</h3>
              <Button
                onClick={() => setShowOverview(false)}
                className="bg-gray-600 hover:bg-gray-700 text-white"
              >
                Close
              </Button>
            </div>
            
            {/* Tournament Header */}
            <div className="bg-gradient-to-br from-gray-700/50 to-gray-800/50 rounded-lg p-6 mb-6 border border-gray-600/30">
              <div className="flex items-center space-x-4 mb-4">
                {tournamentOverview.tournament.logo_path && (
                  <img
                    src={tournamentOverview.tournament.logo_path}
                    alt="Tournament logo"
                    className="w-16 h-16 rounded-lg"
                  />
                )}
                <div>
                  <h4 className="text-2xl font-bold text-gray-100">{tournamentOverview.tournament.name}</h4>
                  <p className="text-gray-400">
                    {tournamentOverview.tournament.city}, {tournamentOverview.tournament.country}
                  </p>
                  <div className="flex items-center space-x-2 mt-2">
                    <StatusDot color={getStatusColor(tournamentOverview.tournament.status)} />
                    <span className="text-gray-300">{getStatusText(tournamentOverview.tournament.status)}</span>
                  </div>
                </div>
              </div>
              
              {/* Tournament Statistics */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="bg-gray-700/30 rounded-lg p-4 text-center">
                  <div className="text-2xl font-bold text-blue-400">{tournamentOverview.total_matches}</div>
                  <div className="text-sm text-gray-400">Total Matches</div>
                </div>
                <div className="bg-gray-700/30 rounded-lg p-4 text-center">
                  <div className="text-2xl font-bold text-green-400">{tournamentOverview.total_events}</div>
                  <div className="text-sm text-gray-400">Total Events</div>
                </div>
                <div className="bg-gray-700/30 rounded-lg p-4 text-center">
                  <div className="text-2xl font-bold text-yellow-400">{tournamentOverview.total_scores}</div>
                  <div className="text-sm text-gray-400">Total Scores</div>
                </div>
                <div className="bg-gray-700/30 rounded-lg p-4 text-center">
                  <div className="text-2xl font-bold text-red-400">{tournamentOverview.total_warnings}</div>
                  <div className="text-sm text-gray-400">Total Warnings</div>
                </div>
              </div>
            </div>
            
            {/* Tournament Days Overview */}
            <div className="bg-gradient-to-br from-gray-700/50 to-gray-800/50 rounded-lg p-6 mb-6 border border-gray-600/30">
              <h5 className="text-lg font-semibold text-gray-100 mb-4">Tournament Days Progress</h5>
              <div className="grid grid-cols-3 gap-4 mb-4">
                <div className="text-center">
                  <div className="text-2xl font-bold text-yellow-400">{tournamentOverview.pending_days}</div>
                  <div className="text-sm text-gray-400">Pending Days</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-green-400">
                    {tournamentOverview.active_day ? 1 : 0}
                  </div>
                  <div className="text-sm text-gray-400">Active Day</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-blue-400">{tournamentOverview.completed_days}</div>
                  <div className="text-sm text-gray-400">Completed Days</div>
                </div>
              </div>
              
              <div className="space-y-3">
                {tournamentOverview.days.map((day) => (
                  <div
                    key={day.id}
                    className="p-3 rounded-lg border border-gray-600/30 bg-gray-700/30"
                  >
                    <div className="flex items-center justify-between">
                      <div>
                        <h6 className="font-medium text-gray-100">Day {day.day_number}</h6>
                        <p className="text-sm text-gray-400">
                          {new Date(day.date).toLocaleDateString()}
                          {day.start_time && ` • Started: ${new Date(day.start_time).toLocaleTimeString()}`}
                          {day.end_time && ` • Ended: ${new Date(day.end_time).toLocaleTimeString()}`}
                        </p>
                      </div>
                      <div className="flex items-center space-x-2">
                        <StatusDot color={getDayStatusColor(day.status)} />
                        <span className="text-sm text-gray-400 capitalize">{day.status}</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
            
            {/* Tournament Timeline */}
            <div className="bg-gradient-to-br from-gray-700/50 to-gray-800/50 rounded-lg p-6 border border-gray-600/30">
              <h5 className="text-lg font-semibold text-gray-100 mb-4">Tournament Timeline</h5>
              <div className="space-y-3">
                <div className="flex items-center space-x-3">
                  <div className="w-3 h-3 bg-green-400 rounded-full"></div>
                  <div>
                    <div className="text-gray-100">Created</div>
                    <div className="text-sm text-gray-400">
                      {new Date(tournamentOverview.tournament.created_at).toLocaleString()}
                    </div>
                  </div>
                </div>
                {tournamentOverview.tournament.start_date && (
                  <div className="flex items-center space-x-3">
                    <div className="w-3 h-3 bg-blue-400 rounded-full"></div>
                    <div>
                      <div className="text-gray-100">Started</div>
                      <div className="text-sm text-gray-400">
                        {new Date(tournamentOverview.tournament.start_date).toLocaleString()}
                      </div>
                    </div>
                  </div>
                )}
                {tournamentOverview.tournament.end_date && (
                  <div className="flex items-center space-x-3">
                    <div className="w-3 h-3 bg-red-400 rounded-full"></div>
                    <div>
                      <div className="text-gray-100">Ended</div>
                      <div className="text-sm text-gray-400">
                        {new Date(tournamentOverview.tournament.end_date).toLocaleString()}
                      </div>
                    </div>
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
          <div className="bg-gradient-to-br from-gray-800/90 to-gray-900/95 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold text-gray-100 mb-4">Add New Tournament</h3>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="name">Tournament Name</Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                  placeholder="Enter tournament name"
                />
              </div>
              
              <div>
                <Label htmlFor="duration">Duration (days)</Label>
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
                  <Label htmlFor="city">City</Label>
                  <Input
                    id="city"
                    value={formData.city}
                    onChange={(e) => setFormData(prev => ({ ...prev, city: e.target.value }))}
                    placeholder="Enter city"
                  />
                </div>
                <div>
                  <Label htmlFor="country">Country</Label>
                  <Input
                    id="country"
                    value={formData.country}
                    onChange={(e) => setFormData(prev => ({ ...prev, country: e.target.value }))}
                    placeholder="Enter country"
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
                  {isVerifyingLocation ? 'Verifying...' : 'Verify Location'}
                </Button>
                {locationVerification.verified && (
                  <div className="flex items-center space-x-1 text-green-400">
                    <Icon name="✅" size="text-sm" />
                    <span className="text-sm">Verified</span>
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
                <Label htmlFor="start_date">Start Date (Optional)</Label>
                <Input
                  id="start_date"
                  type="datetime-local"
                  value={formData.start_date}
                  onChange={(e) => setFormData(prev => ({ ...prev, start_date: e.target.value }))}
                />
              </div>
              
              <div>
                <Label htmlFor="logo">Tournament Logo (Optional)</Label>
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
                Cancel
              </Button>
              <Button
                onClick={createTournament}
                disabled={isLoading || !formData.name || !formData.city || !formData.country}
                className="bg-blue-600 hover:bg-blue-700 text-white"
              >
                {isLoading ? 'Creating...' : 'Create Tournament'}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Edit Tournament Form */}
      {showEditForm && selectedTournament && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-gradient-to-br from-gray-800/90 to-gray-900/95 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold text-gray-100 mb-4">Edit Tournament</h3>
            
            <div className="space-y-4">
              <div>
                <Label htmlFor="edit-name">Tournament Name</Label>
                <Input
                  id="edit-name"
                  value={formData.name}
                  onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                  placeholder="Enter tournament name"
                />
              </div>
              
              <div>
                <Label htmlFor="edit-duration">Duration (days)</Label>
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
                  <Label htmlFor="edit-city">City</Label>
                  <Input
                    id="edit-city"
                    value={formData.city}
                    onChange={(e) => setFormData(prev => ({ ...prev, city: e.target.value }))}
                    placeholder="Enter city"
                  />
                </div>
                <div>
                  <Label htmlFor="edit-country">Country</Label>
                  <Input
                    id="edit-country"
                    value={formData.country}
                    onChange={(e) => setFormData(prev => ({ ...prev, country: e.target.value }))}
                    placeholder="Enter country"
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
                  {isVerifyingLocation ? 'Verifying...' : 'Verify Location'}
                </Button>
                {locationVerification.verified && (
                  <div className="flex items-center space-x-1 text-green-400">
                    <Icon name="✅" size="text-sm" />
                    <span className="text-sm">Verified</span>
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
                <Label htmlFor="edit-start_date">Start Date (Optional)</Label>
                <Input
                  id="edit-start_date"
                  type="datetime-local"
                  value={formData.start_date}
                  onChange={(e) => setFormData(prev => ({ ...prev, start_date: e.target.value }))}
                />
              </div>
              
              <div>
                <Label htmlFor="edit-logo">Tournament Logo (Optional)</Label>
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
                    {isUploadingLogo ? 'Uploading...' : 'Upload Logo'}
                  </Button>
                )}
              </div>
            </div>
            
            <div className="flex justify-end space-x-3 mt-6">
              <Button
                onClick={() => setShowEditForm(false)}
                className="bg-gray-600 hover:bg-gray-700 text-white"
              >
                Cancel
              </Button>
              <Button
                onClick={updateTournament}
                disabled={isLoading || !formData.name || !formData.city || !formData.country}
                className="bg-blue-600 hover:bg-blue-700 text-white"
              >
                {isLoading ? 'Updating...' : 'Update Tournament'}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Start Day Confirmation Modal */}
      {showStartDayModal && selectedDay && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-gradient-to-br from-gray-800/90 to-gray-900/95 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold text-gray-100 mb-4">Start Tournament Day</h3>
            <p className="text-gray-300 mb-6">
              Are you sure you want to start Day {selectedDay.day_number}? 
              This will automatically start the tournament if it's the first day.
            </p>
            <div className="flex justify-end space-x-3">
              <Button
                onClick={() => setShowStartDayModal(false)}
                className="bg-gray-600 hover:bg-gray-700 text-white"
              >
                Cancel
              </Button>
              <Button
                onClick={() => startTournamentDay(selectedDay.id)}
                disabled={isLoading}
                className="bg-green-600 hover:bg-green-700 text-white"
              >
                {isLoading ? 'Starting...' : 'Start Day'}
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* End Day Confirmation Modal */}
      {showEndDayModal && selectedDay && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-gradient-to-br from-gray-800/90 to-gray-900/95 backdrop-blur-sm rounded-lg border border-gray-600/30 shadow-xl p-6 w-full max-w-md">
            <h3 className="text-lg font-semibold text-gray-100 mb-4">End Tournament Day</h3>
            <p className="text-gray-300 mb-6">
              Are you sure you want to end Day {selectedDay.day_number}? 
              This will automatically end the tournament if it's the final day.
            </p>
            <div className="flex justify-end space-x-3">
              <Button
                onClick={() => setShowEndDayModal(false)}
                className="bg-gray-600 hover:bg-gray-700 text-white"
              >
                Cancel
              </Button>
              <Button
                onClick={() => endTournamentDay(selectedDay.id)}
                disabled={isLoading}
                className="bg-blue-600 hover:bg-blue-700 text-white"
              >
                {isLoading ? 'Ending...' : 'End Day'}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default TournamentManagementPanel;
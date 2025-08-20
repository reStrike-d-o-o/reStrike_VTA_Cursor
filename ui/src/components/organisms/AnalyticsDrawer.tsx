import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../atoms/Card';
import { Badge } from '../atoms/Badge';
import Button from '../atoms/Button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../atoms/Tabs';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../atoms/Select';
import { AthleteAnalytics } from '../molecules/AthleteAnalytics';
import { MatchAnalytics } from '../molecules/MatchAnalytics';
import { TournamentAnalytics } from '../molecules/TournamentAnalytics';
import { DayAnalytics } from '../molecules/DayAnalytics';
import { usePssMatchStore } from '../../stores/pssMatchStore';
import { useI18n } from '../../i18n/index';

interface AnalyticsDrawerProps {
  isOpen: boolean;
  onClose: () => void;
}

export const AnalyticsDrawer: React.FC<AnalyticsDrawerProps> = ({
  isOpen,
  onClose,
}) => {
  const { t } = useI18n();
  const [selectedAthlete, setSelectedAthlete] = useState<string>('');
  const [selectedMatch, setSelectedMatch] = useState<string>('');
  const [selectedDate, setSelectedDate] = useState<string>(new Date().toISOString().split('T')[0]);
  const [selectedTournament, setSelectedTournament] = useState<string>('');
  const [activeTab, setActiveTab] = useState<string>('tournament');

  const { matchData } = usePssMatchStore();
  
  // Mock events data for now - this should be replaced with actual PSS events
  const events = React.useRef<any[]>([]).current;

  // Extract unique athletes, matches, and tournaments from events
  const athletes = Array.from(new Set(
    events
      .filter(event => event.athlete1_code || event.athlete2_code)
      .flatMap(event => [
        { id: event.athlete1_code, name: event.athlete1_name || event.athlete1_code, country: event.athlete1_country },
        { id: event.athlete2_code, name: event.athlete2_name || event.athlete2_code, country: event.athlete2_country }
      ])
      .filter(athlete => athlete.id)
  ));

  const matches = Array.from(new Set(
    events
      .filter(event => event.match_id)
      .map(event => event.match_id)
  ));

  const tournaments = Array.from(new Set(
    events
      .filter(event => event.tournament_id)
      .map(event => event.tournament_id)
  ));

  const dates = Array.from(new Set(
    events
      .map(event => new Date(event.timestamp).toISOString().split('T')[0])
      .sort()
      .reverse()
  ));

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 z-50 flex justify-end">
      <div className="w-full max-w-4xl bg-background h-full overflow-y-auto">
        <div className="p-6">
          <div className="flex items-center justify-between mb-6">
            <div className="flex items-center gap-2">
              <h2 className="text-2xl font-bold">{t('analytics.title', '📊 Analytics Dashboard')}</h2>
              <Badge variant="secondary">{t('analytics.realtime', 'Real-time')}</Badge>
            </div>
            <Button variant="ghost" onClick={onClose}>
              ✕ {t('common.close', 'Close')}
            </Button>
          </div>

          <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
            <TabsList className="grid w-full grid-cols-4">
              <TabsTrigger value="tournament">🏆 {t('analytics.tabs.tournament', 'Tournament')}</TabsTrigger>
              <TabsTrigger value="athlete">👤 {t('analytics.tabs.athlete', 'Athlete')}</TabsTrigger>
              <TabsTrigger value="match">🥋 {t('analytics.tabs.match', 'Match')}</TabsTrigger>
              <TabsTrigger value="day">📅 {t('analytics.tabs.day', 'Day')}</TabsTrigger>
            </TabsList>

            <TabsContent value="tournament" className="space-y-4">
              <div className="space-y-4">
                <div className="flex items-center gap-4">
                  <div className="flex-1">
                    <Label htmlFor="tournament-select">{t('analytics.select.tournament', 'Select Tournament')}</Label>
                    <Select value={selectedTournament} onValueChange={setSelectedTournament}>
                      <SelectTrigger>
                        <SelectValue placeholder={t('analytics.placeholder.tournament', 'Choose a tournament')} />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="">{t('analytics.all_tournaments', 'All Tournaments')}</SelectItem>
                        {tournaments.map((tournamentId) => (
                          <SelectItem key={tournamentId} value={tournamentId}>
                            {t('analytics.item.tournament', 'Tournament {id}', { id: String(tournamentId) })}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <TournamentAnalytics 
                  tournamentName={selectedTournament ? t('analytics.item.tournament', 'Tournament {id}', { id: String(selectedTournament) }) : t('analytics.all_tournaments', 'All Tournaments')}
                  tournamentId={selectedTournament || undefined}
                />
              </div>
            </TabsContent>

            <TabsContent value="athlete" className="space-y-4">
              <div className="space-y-4">
                <div className="flex items-center gap-4">
                  <div className="flex-1">
                    <Label htmlFor="athlete-select">{t('analytics.select.athlete', 'Select Athlete')}</Label>
                    <Select value={selectedAthlete} onValueChange={setSelectedAthlete}>
                      <SelectTrigger>
                        <SelectValue placeholder={t('analytics.placeholder.athlete', 'Choose an athlete')} />
                      </SelectTrigger>
                      <SelectContent>
                        {athletes.map((athlete) => (
                          <SelectItem key={athlete.id} value={athlete.id}>
                            {athlete.name} {athlete.country && `(${athlete.country})`}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                {selectedAthlete && (
                  <AthleteAnalytics 
                    athleteId={selectedAthlete}
                    athleteName={athletes.find(a => a.id === selectedAthlete)?.name || selectedAthlete}
                    countryCode={athletes.find(a => a.id === selectedAthlete)?.country}
                  />
                )}

                {!selectedAthlete && (
                  <Card>
                    <CardHeader>
                      <CardTitle>👤 {t('analytics.athlete.title', 'Athlete Analytics')}</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <p className="text-muted-foreground">
                        {t('analytics.athlete.help', 'Select an athlete from the dropdown above to view their detailed analytics and performance metrics.')}
                      </p>
                    </CardContent>
                  </Card>
                )}
              </div>
            </TabsContent>

            <TabsContent value="match" className="space-y-4">
              <div className="space-y-4">
                <div className="flex items-center gap-4">
                  <div className="flex-1">
                    <Label htmlFor="match-select">{t('analytics.select.match', 'Select Match')}</Label>
                    <Select value={selectedMatch} onValueChange={setSelectedMatch}>
                      <SelectTrigger>
                        <SelectValue placeholder={t('analytics.placeholder.match', 'Choose a match')} />
                      </SelectTrigger>
                      <SelectContent>
                        {matches.map((matchId) => (
                          <SelectItem key={matchId} value={matchId}>
                            {t('analytics.item.match', 'Match {id}', { id: String(matchId) })}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                {selectedMatch && (
                  <MatchAnalytics 
                    matchId={selectedMatch}
                    athlete1Name="Athlete 1"
                    athlete2Name="Athlete 2"
                    athlete1Country="RUS"
                    athlete2Country="GER"
                  />
                )}

                {!selectedMatch && (
                  <Card>
                    <CardHeader>
                      <CardTitle>🥋 {t('analytics.match.title', 'Match Analytics')}</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <p className="text-muted-foreground">
                        {t('analytics.match.help', 'Select a match from the dropdown above to view detailed match analytics and performance metrics.')}
                      </p>
                    </CardContent>
                  </Card>
                )}
              </div>
            </TabsContent>

            <TabsContent value="day" className="space-y-4">
              <div className="space-y-4">
                <div className="flex items-center gap-4">
                  <div className="flex-1">
                    <Label htmlFor="date-select">{t('analytics.select.date', 'Select Date')}</Label>
                    <Select value={selectedDate} onValueChange={setSelectedDate}>
                      <SelectTrigger>
                        <SelectValue placeholder={t('analytics.placeholder.date', 'Choose a date')} />
                      </SelectTrigger>
                      <SelectContent>
                        {dates.map((date) => (
                          <SelectItem key={date} value={date}>
                            {(() => { const d=new Date(date); const dd=String(d.getDate()).padStart(2,'0'); const mm=String(d.getMonth()+1).padStart(2,'0'); const yyyy=d.getFullYear(); const hh=String(d.getHours()).padStart(2,'0'); const mi=String(d.getMinutes()).padStart(2,'0'); return `${dd}.${mm}.${yyyy} ${hh}:${mi}`; })()}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <DayAnalytics 
                  date={selectedDate}
                  tournamentId={selectedTournament || undefined}
                />
              </div>
            </TabsContent>
          </Tabs>

          <div className="mt-8 p-4 bg-muted rounded-lg">
            <h3 className="font-semibold mb-2">{t('analytics.overview.title', '📈 Analytics Overview')}</h3>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
              <div>
                <div className="font-medium">{t('analytics.metrics.total_events', 'Total Events')}</div>
                <div className="text-muted-foreground">{events.length}</div>
              </div>
              <div>
                <div className="font-medium">{t('analytics.metrics.unique_matches', 'Unique Matches')}</div>
                <div className="text-muted-foreground">{matches.length}</div>
              </div>
              <div>
                <div className="font-medium">{t('analytics.metrics.unique_athletes', 'Unique Athletes')}</div>
                <div className="text-muted-foreground">{athletes.length}</div>
              </div>
              <div>
                <div className="font-medium">{t('analytics.metrics.tournaments', 'Tournaments')}</div>
                <div className="text-muted-foreground">{tournaments.length}</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}; 
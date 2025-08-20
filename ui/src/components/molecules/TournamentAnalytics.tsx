import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../atoms/Card';
import { Badge } from '../atoms/Badge';
import { Progress } from '../atoms/Progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../atoms/Tabs';
import { usePssMatchStore } from '../../stores/pssMatchStore';
import { useI18n } from '../../i18n/index';

interface TournamentStats {
  totalMatches: number;
  totalEvents: number;
  totalAthletes: number;
  totalCountries: number;
  averageMatchDuration: number;
  averageEventsPerMatch: number;
  totalPoints: number;
  totalWarnings: number;
  totalInjuries: number;
  completedMatches: number;
  inProgressMatches: number;
  topAthletes: Array<{
    name: string;
    countryCode?: string;
    points: number;
    matches: number;
    winRate: number;
  }>;
  topCountries: Array<{
    countryCode: string;
    countryName: string;
    athletes: number;
    totalPoints: number;
    averagePoints: number;
  }>;
  eventDistribution: {
    points: number;
    warnings: number;
    injuries: number;
    other: number;
  };
  matchIntensity: number;
}

interface TournamentAnalyticsProps {
  tournamentName?: string;
  tournamentId?: string;
}

export const TournamentAnalytics: React.FC<TournamentAnalyticsProps> = ({
  tournamentName = "Current Tournament",
  tournamentId,
}) => {
  const { t } = useI18n();
  const [stats, setStats] = useState<TournamentStats>({
    totalMatches: 0,
    totalEvents: 0,
    totalAthletes: 0,
    totalCountries: 0,
    averageMatchDuration: 0,
    averageEventsPerMatch: 0,
    totalPoints: 0,
    totalWarnings: 0,
    totalInjuries: 0,
    completedMatches: 0,
    inProgressMatches: 0,
    topAthletes: [],
    topCountries: [],
    eventDistribution: {
      points: 0,
      warnings: 0,
      injuries: 0,
      other: 0,
    },
    matchIntensity: 0,
  });

  const [isLoading, setIsLoading] = useState(true);
  const { matchData } = usePssMatchStore();
  
  // Mock events data for now - this should be replaced with actual PSS events
  const events = React.useRef<any[]>([]).current;

  useEffect(() => {
    const calculateTournamentStats = () => {
      // Filter events for this tournament
      const tournamentEvents = tournamentId 
        ? events.filter((event: any) => event.tournament_id === tournamentId)
        : events;

      if (tournamentEvents.length === 0) {
        setIsLoading(false);
        return;
      }

      // Group events by match
      const matchGroups = new Map<string, any[]>();
      const athleteStats = new Map<string, {
        name: string;
        countryCode?: string;
        points: number;
        matches: Set<string>;
        wins: number;
        totalMatches: number;
      }>();
      const countryStats = new Map<string, {
        countryName: string;
        athletes: Set<string>;
        totalPoints: number;
      }>();

      let totalEvents = tournamentEvents.length;
      let totalPoints = 0;
      let totalWarnings = 0;
      let totalInjuries = 0;
      let totalOther = 0;
      let totalDuration = 0;
      let completedMatches = 0;
      let inProgressMatches = 0;

      tournamentEvents.forEach(event => {
        // Group by match
        if (event.match_id) {
          if (!matchGroups.has(event.match_id)) {
            matchGroups.set(event.match_id, []);
          }
          matchGroups.get(event.match_id)!.push(event);
        }

        // Track athlete statistics
        if (event.athlete1_code) {
          if (!athleteStats.has(event.athlete1_code)) {
            athleteStats.set(event.athlete1_code, {
              name: event.athlete1_name || event.athlete1_code,
              countryCode: event.athlete1_country,
              points: 0,
              matches: new Set(),
              wins: 0,
              totalMatches: 0,
            });
          }
          const athlete = athleteStats.get(event.athlete1_code)!;
          athlete.matches.add(event.match_id || '');
          if (event.score1) athlete.points += event.score1;
        }

        if (event.athlete2_code) {
          if (!athleteStats.has(event.athlete2_code)) {
            athleteStats.set(event.athlete2_code, {
              name: event.athlete2_name || event.athlete2_code,
              countryCode: event.athlete2_country,
              points: 0,
              matches: new Set(),
              wins: 0,
              totalMatches: 0,
            });
          }
          const athlete = athleteStats.get(event.athlete2_code)!;
          athlete.matches.add(event.match_id || '');
          if (event.score2) athlete.points += event.score2;
        }

        // Track country statistics
        if (event.athlete1_country) {
          if (!countryStats.has(event.athlete1_country)) {
            countryStats.set(event.athlete1_country, {
              countryName: event.athlete1_country,
              athletes: new Set(),
              totalPoints: 0,
            });
          }
          const country = countryStats.get(event.athlete1_country)!;
          country.athletes.add(event.athlete1_code || '');
          if (event.score1) country.totalPoints += event.score1;
        }

        if (event.athlete2_country) {
          if (!countryStats.has(event.athlete2_country)) {
            countryStats.set(event.athlete2_country, {
              countryName: event.athlete2_country,
              athletes: new Set(),
              totalPoints: 0,
            });
          }
          const country = countryStats.get(event.athlete2_country)!;
          country.athletes.add(event.athlete2_code || '');
          if (event.score2) country.totalPoints += event.score2;
        }

        // Count event types
        if (event.event_type === 'point' || event.event_type === 'score') {
          totalPoints++;
        } else if (event.event_type === 'warning' || event.event_type === 'gam_jeom') {
          totalWarnings++;
        } else if (event.event_type === 'injury') {
          totalInjuries++;
        } else {
          totalOther++;
        }
      });

      // Analyze matches
      matchGroups.forEach((matchEvents, matchId) => {
        let matchDuration = 0;
        let isCompleted = false;

        if (matchEvents.length > 0) {
          const startTime = new Date(matchEvents[0].timestamp);
          const endTime = new Date(matchEvents[matchEvents.length - 1].timestamp);
          matchDuration = (endTime.getTime() - startTime.getTime()) / 1000;
          totalDuration += matchDuration;

          // Check if match is completed
          const hasMatchEnd = matchEvents.some(event => event.event_type === 'match_end');
          if (hasMatchEnd) {
            completedMatches++;
          } else {
            inProgressMatches++;
          }
        }
      });

      // Calculate athlete win rates
      athleteStats.forEach((athlete, athleteCode) => {
        athlete.totalMatches = athlete.matches.size;
        // Simplified win rate calculation (would need more complex logic for actual wins)
        athlete.wins = Math.floor(athlete.totalMatches * 0.5); // Placeholder
      });

      // Convert to arrays and sort
      const topAthletes = Array.from(athleteStats.entries())
        .map(([code, athlete]) => ({
          name: athlete.name,
          countryCode: athlete.countryCode,
          points: athlete.points,
          matches: athlete.totalMatches,
          winRate: athlete.totalMatches > 0 ? (athlete.wins / athlete.totalMatches) * 100 : 0,
        }))
        .sort((a, b) => b.points - a.points)
        .slice(0, 10);

      const topCountries = Array.from(countryStats.entries())
        .map(([code, country]) => ({
          countryCode: code,
          countryName: country.countryName,
          athletes: country.athletes.size,
          totalPoints: country.totalPoints,
          averagePoints: country.athletes.size > 0 ? country.totalPoints / country.athletes.size : 0,
        }))
        .sort((a, b) => b.totalPoints - a.totalPoints)
        .slice(0, 10);

      const totalMatches = matchGroups.size;
      const averageMatchDuration = totalMatches > 0 ? totalDuration / totalMatches : 0;
      const averageEventsPerMatch = totalMatches > 0 ? totalEvents / totalMatches : 0;
      const matchIntensity = totalDuration > 0 ? totalEvents / (totalDuration / 60) : 0;

      setStats({
        totalMatches,
        totalEvents,
        totalAthletes: athleteStats.size,
        totalCountries: countryStats.size,
        averageMatchDuration,
        averageEventsPerMatch,
        totalPoints,
        totalWarnings,
        totalInjuries,
        completedMatches,
        inProgressMatches,
        topAthletes,
        topCountries,
        eventDistribution: {
          points: totalPoints,
          warnings: totalWarnings,
          injuries: totalInjuries,
          other: totalOther,
        },
        matchIntensity,
      });

      setIsLoading(false);
    };

    calculateTournamentStats();
  }, [events, tournamentId]);

  if (isLoading) {
    return (
      <Card className="w-full">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <div className="w-6 h-6 bg-gray-200 rounded animate-pulse"></div>
            <div className="h-6 bg-gray-200 rounded w-32 animate-pulse"></div>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {[...Array(6)].map((_, i) => (
              <div key={i} className="h-4 bg-gray-200 rounded animate-pulse"></div>
            ))}
          </div>
        </CardContent>
      </Card>
    );
  }

  const formatDuration = (seconds: number) => {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  };

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <span>🏆 {tournamentName}</span>
          <Badge variant="secondary">{t('analytics.tournament.badge', 'Tournament Analytics')}</Badge>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="overview" className="w-full">
          <TabsList className="grid w-full grid-cols-4">
            <TabsTrigger value="overview">{t('analytics.sections.overview', 'Overview')}</TabsTrigger>
            <TabsTrigger value="athletes">{t('analytics.tournament.tabs.athletes', 'Top Athletes')}</TabsTrigger>
            <TabsTrigger value="countries">{t('analytics.tournament.tabs.countries', 'Countries')}</TabsTrigger>
            <TabsTrigger value="matches">
              <img src="/icons/bar-graph.json" alt="Statistics" className="w-4 h-4 mr-2" />
              {t('analytics.sections.trends', 'Statistics')}
            </TabsTrigger>
          </TabsList>

          <TabsContent value="overview" className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">{t('analytics.metrics.total_matches', 'Total Matches')}</span>
                  <span className="text-sm text-muted-foreground">{stats.totalMatches}</span>
                </div>
                <Progress value={stats.totalMatches > 0 ? Math.min((stats.totalMatches / 100) * 100, 100) : 0} />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">{t('analytics.metrics.total_events', 'Total Events')}</span>
                  <span className="text-sm text-muted-foreground">{stats.totalEvents}</span>
                </div>
                <Progress value={stats.totalEvents > 0 ? Math.min((stats.totalEvents / 1000) * 100, 100) : 0} />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">{t('analytics.metrics.total_athletes', 'Total Athletes')}</span>
                  <span className="text-sm text-muted-foreground">{stats.totalAthletes}</span>
                </div>
                <Progress value={stats.totalAthletes > 0 ? Math.min((stats.totalAthletes / 50) * 100, 100) : 0} />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">{t('analytics.metrics.total_countries', 'Total Countries')}</span>
                  <span className="text-sm text-muted-foreground">{stats.totalCountries}</span>
                </div>
                <Progress value={stats.totalCountries > 0 ? Math.min((stats.totalCountries / 20) * 100, 100) : 0} />
              </div>
            </div>

            <div className="grid grid-cols-3 gap-4 pt-4">
              <div className="text-center">
                <div className="text-2xl font-bold text-green-600">{stats.completedMatches}</div>
                <div className="text-sm text-muted-foreground">{t('analytics.tournament.completed', 'Completed')}</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-blue-600">{stats.inProgressMatches}</div>
                <div className="text-sm text-muted-foreground">{t('analytics.tournament.in_progress', 'In Progress')}</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-orange-600">{stats.totalPoints}</div>
                <div className="text-sm text-muted-foreground">{t('analytics.tournament.total_points', 'Total Points')}</div>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="p-4 border rounded-lg">
                <div className="text-2xl font-bold text-purple-600">{formatDuration(stats.averageMatchDuration)}</div>
                <div className="text-sm text-muted-foreground">{t('analytics.tournament.avg_duration', 'Avg Match Duration')}</div>
              </div>
              <div className="p-4 border rounded-lg">
                <div className="text-2xl font-bold text-indigo-600">{stats.averageEventsPerMatch.toFixed(1)}</div>
                <div className="text-sm text-muted-foreground">{t('analytics.tournament.avg_events_per_match', 'Avg Events/Match')}</div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="athletes" className="space-y-4">
            <div className="space-y-4">
              <h4 className="font-semibold">{t('analytics.tournament.top_athletes_title', 'Top 10 Athletes by Points')}</h4>
              <div className="space-y-2">
                {stats.topAthletes.map((athlete, index) => (
                  <div key={index} className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex items-center gap-3">
                      <div className="w-8 h-8 bg-muted rounded-full flex items-center justify-center text-sm font-bold">
                        {index + 1}
                      </div>
                      <div className="flex items-center gap-2">
                        {athlete.countryCode && (
                          <span className="text-lg">
                            {athlete.countryCode === 'RUS' ? '🇷🇺' : 
                             athlete.countryCode === 'GER' ? '🇩🇪' : 
                             athlete.countryCode === 'USA' ? '🇺🇸' : 
                             athlete.countryCode === 'KOR' ? '🇰🇷' : '🏳️'}
                          </span>
                        )}
                        <span className="font-medium">{athlete.name}</span>
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="font-bold text-green-600">{athlete.points}</div>
                      <div className="text-sm text-muted-foreground">
                        {t('analytics.tournament.athlete_summary', '{matches} matches • {rate}% win rate', { matches: String(athlete.matches), rate: athlete.winRate.toFixed(1) })}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </TabsContent>

          <TabsContent value="countries" className="space-y-4">
            <div className="space-y-4">
              <h4 className="font-semibold">{t('analytics.tournament.top_countries_title', 'Top 10 Countries by Points')}</h4>
              <div className="space-y-2">
                {stats.topCountries.map((country, index) => (
                  <div key={index} className="flex items-center justify-between p-3 border rounded-lg">
                    <div className="flex items-center gap-3">
                      <div className="w-8 h-8 bg-muted rounded-full flex items-center justify-center text-sm font-bold">
                        {index + 1}
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-lg">
                          {country.countryCode === 'RUS' ? '🇷🇺' : 
                           country.countryCode === 'GER' ? '🇩🇪' : 
                           country.countryCode === 'USA' ? '🇺🇸' : 
                           country.countryCode === 'KOR' ? '🇰🇷' : '🏳️'}
                        </span>
                        <span className="font-medium">{country.countryName}</span>
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="font-bold text-green-600">{country.totalPoints}</div>
                      <div className="text-sm text-muted-foreground">
                        {t('analytics.tournament.country_summary', '{athletes} athletes • {avg} avg', { athletes: String(country.athletes), avg: country.averagePoints.toFixed(1) })}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </TabsContent>

          <TabsContent value="matches" className="space-y-4">
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-green-600">{stats.eventDistribution.points}</div>
                  <div className="text-sm text-muted-foreground">{t('analytics.match.points_events', 'Points Events')}</div>
                </div>
                <div className="p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-orange-600">{stats.eventDistribution.warnings}</div>
                  <div className="text-sm text-muted-foreground">{t('analytics.match.warning_events', 'Warning Events')}</div>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-red-600">{stats.eventDistribution.injuries}</div>
                  <div className="text-sm text-muted-foreground">{t('analytics.match.injury_events', 'Injury Events')}</div>
                </div>
                <div className="p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-gray-600">{stats.eventDistribution.other}</div>
                  <div className="text-sm text-muted-foreground">{t('analytics.match.other_events', 'Other Events')}</div>
                </div>
              </div>

              <div className="p-4 border rounded-lg">
                <h4 className="font-semibold mb-2">{t('analytics.tournament.statistics_title', 'Tournament Statistics')}</h4>
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm">{t('analytics.tournament.match_intensity', 'Match Intensity:')}</span>
                    <span className="text-sm font-medium">{stats.matchIntensity.toFixed(2)} {t('analytics.match.events_per_min', 'events/min')}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">{t('analytics.tournament.completion_rate', 'Completion Rate:')}</span>
                    <span className="text-sm font-medium">
                      {stats.totalMatches > 0 ? ((stats.completedMatches / stats.totalMatches) * 100).toFixed(1) : 0}%
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">{t('analytics.tournament.avg_match_duration', 'Average Match Duration:')}</span>
                    <span className="text-sm font-medium">{formatDuration(stats.averageMatchDuration)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">{t('analytics.tournament.avg_events_match', 'Average Events per Match:')}</span>
                    <span className="text-sm font-medium">{stats.averageEventsPerMatch.toFixed(1)}</span>
                  </div>
                </div>
              </div>
            </div>
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  );
}; 
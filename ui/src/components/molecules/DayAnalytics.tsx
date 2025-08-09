import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../atoms/Card';
import { Badge } from '../atoms/Badge';
import { Progress } from '../atoms/Progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../atoms/Tabs';
import { usePssMatchStore } from '../../stores/pssMatchStore';

interface DayStats {
  date: string;
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
  peakHour: number;
  peakHourEvents: number;
  eventTimeline: Array<{
    hour: number;
    events: number;
    matches: number;
  }>;
  topAthletes: Array<{
    name: string;
    countryCode?: string;
    points: number;
    matches: number;
    events: number;
  }>;
  eventDistribution: {
    points: number;
    warnings: number;
    injuries: number;
    other: number;
  };
  matchIntensity: number;
  dayEfficiency: number;
}

interface DayAnalyticsProps {
  date?: string;
  tournamentId?: string;
}

export const DayAnalytics: React.FC<DayAnalyticsProps> = ({
  date,
  tournamentId,
}) => {
  const [stats, setStats] = useState<DayStats>({
    date: date || new Date().toISOString().split('T')[0],
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
    peakHour: 0,
    peakHourEvents: 0,
    eventTimeline: [],
    topAthletes: [],
    eventDistribution: {
      points: 0,
      warnings: 0,
      injuries: 0,
      other: 0,
    },
    matchIntensity: 0,
    dayEfficiency: 0,
  });

  const [isLoading, setIsLoading] = useState(true);
  const { matchData } = usePssMatchStore();
  
  // Mock events data for now - this should be replaced with actual PSS events
  const events = React.useRef<any[]>([]).current;

  useEffect(() => {
    const calculateDayStats = () => {
      const targetDate = date || new Date().toISOString().split('T')[0];
      
      // Filter events for the specified day
      const dayEvents = events.filter((event: any) => {
        const eventDate = new Date(event.timestamp).toISOString().split('T')[0];
        return eventDate === targetDate && (!tournamentId || event.tournament_id === tournamentId);
      });

      if (dayEvents.length === 0) {
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
        events: number;
      }>();
      const countryStats = new Set<string>();
      const hourlyStats = new Map<number, { events: number; matches: Set<string> }>();

      let totalEvents = dayEvents.length;
      let totalPoints = 0;
      let totalWarnings = 0;
      let totalInjuries = 0;
      let totalOther = 0;
      let totalDuration = 0;
      let completedMatches = 0;
      let inProgressMatches = 0;

      dayEvents.forEach(event => {
        // Group by match
        if (event.match_id) {
          if (!matchGroups.has(event.match_id)) {
            matchGroups.set(event.match_id, []);
          }
          matchGroups.get(event.match_id)!.push(event);
        }

        // Track hourly statistics
        const eventHour = new Date(event.timestamp).getHours();
        if (!hourlyStats.has(eventHour)) {
          hourlyStats.set(eventHour, { events: 0, matches: new Set() });
        }
        const hourStat = hourlyStats.get(eventHour)!;
        hourStat.events++;
        if (event.match_id) hourStat.matches.add(event.match_id);

        // Track athlete statistics
        if (event.athlete1_code) {
          if (!athleteStats.has(event.athlete1_code)) {
            athleteStats.set(event.athlete1_code, {
              name: event.athlete1_name || event.athlete1_code,
              countryCode: event.athlete1_country,
              points: 0,
              matches: new Set(),
              events: 0,
            });
          }
          const athlete = athleteStats.get(event.athlete1_code)!;
          athlete.matches.add(event.match_id || '');
          athlete.events++;
          if (event.score1) athlete.points += event.score1;
          if (event.athlete1_country) countryStats.add(event.athlete1_country);
        }

        if (event.athlete2_code) {
          if (!athleteStats.has(event.athlete2_code)) {
            athleteStats.set(event.athlete2_code, {
              name: event.athlete2_name || event.athlete2_code,
              countryCode: event.athlete2_country,
              points: 0,
              matches: new Set(),
              events: 0,
            });
          }
          const athlete = athleteStats.get(event.athlete2_code)!;
          athlete.matches.add(event.match_id || '');
          athlete.events++;
          if (event.score2) athlete.points += event.score2;
          if (event.athlete2_country) countryStats.add(event.athlete2_country);
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

      // Find peak hour
      let peakHour = 0;
      let peakHourEvents = 0;
      hourlyStats.forEach((hourStat, hour) => {
        if (hourStat.events > peakHourEvents) {
          peakHourEvents = hourStat.events;
          peakHour = hour;
        }
      });

      // Create event timeline
      const eventTimeline = Array.from({ length: 24 }, (_, hour) => ({
        hour,
        events: hourlyStats.get(hour)?.events || 0,
        matches: hourlyStats.get(hour)?.matches.size || 0,
      }));

      // Convert athlete stats to array and sort
      const topAthletes = Array.from(athleteStats.entries())
        .map(([code, athlete]) => ({
          name: athlete.name,
          countryCode: athlete.countryCode,
          points: athlete.points,
          matches: athlete.matches.size,
          events: athlete.events,
        }))
        .sort((a, b) => b.points - a.points)
        .slice(0, 10);

      const totalMatches = matchGroups.size;
      const averageMatchDuration = totalMatches > 0 ? totalDuration / totalMatches : 0;
      const averageEventsPerMatch = totalMatches > 0 ? totalEvents / totalMatches : 0;
      const matchIntensity = totalDuration > 0 ? totalEvents / (totalDuration / 60) : 0;
      const dayEfficiency = totalMatches > 0 ? (completedMatches / totalMatches) * 100 : 0;

      setStats({
        date: targetDate,
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
        peakHour,
        peakHourEvents,
        eventTimeline,
        topAthletes,
        eventDistribution: {
          points: totalPoints,
          warnings: totalWarnings,
          injuries: totalInjuries,
          other: totalOther,
        },
        matchIntensity,
        dayEfficiency,
      });

      setIsLoading(false);
    };

    calculateDayStats();
  }, [events, date, tournamentId]);

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

  const formatHour = (hour: number) => {
    return `${hour.toString().padStart(2, '0')}:00`;
  };

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <span>📅 {require('../../utils/format').formatDateTime(stats.date)}</span>
          <Badge variant="secondary">Day Analytics</Badge>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="overview" className="w-full">
          <TabsList className="grid w-full grid-cols-4">
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="timeline">Timeline</TabsTrigger>
            <TabsTrigger value="athletes">Top Athletes</TabsTrigger>
            <TabsTrigger value="performance">
              <img src="/icons/bar-graph.json" alt="Statistics" className="w-4 h-4 mr-2" />
              Statistics
            </TabsTrigger>
          </TabsList>

          <TabsContent value="overview" className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">Total Matches</span>
                  <span className="text-sm text-muted-foreground">{stats.totalMatches}</span>
                </div>
                <Progress value={stats.totalMatches > 0 ? Math.min((stats.totalMatches / 50) * 100, 100) : 0} />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">Total Events</span>
                  <span className="text-sm text-muted-foreground">{stats.totalEvents}</span>
                </div>
                <Progress value={stats.totalEvents > 0 ? Math.min((stats.totalEvents / 500) * 100, 100) : 0} />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">Day Efficiency</span>
                  <span className="text-sm text-muted-foreground">{stats.dayEfficiency.toFixed(1)}%</span>
                </div>
                <Progress value={stats.dayEfficiency} />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">Peak Hour</span>
                  <span className="text-sm text-muted-foreground">{formatHour(stats.peakHour)}</span>
                </div>
                <Progress value={stats.peakHourEvents > 0 ? Math.min((stats.peakHourEvents / 100) * 100, 100) : 0} />
              </div>
            </div>

            <div className="grid grid-cols-3 gap-4 pt-4">
              <div className="text-center">
                <div className="text-2xl font-bold text-green-600">{stats.completedMatches}</div>
                <div className="text-sm text-muted-foreground">Completed</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-blue-600">{stats.inProgressMatches}</div>
                <div className="text-sm text-muted-foreground">In Progress</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-orange-600">{stats.totalPoints}</div>
                <div className="text-sm text-muted-foreground">Total Points</div>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="p-4 border rounded-lg">
                <div className="text-2xl font-bold text-purple-600">{formatDuration(stats.averageMatchDuration)}</div>
                <div className="text-sm text-muted-foreground">Avg Match Duration</div>
              </div>
              <div className="p-4 border rounded-lg">
                <div className="text-2xl font-bold text-indigo-600">{stats.averageEventsPerMatch.toFixed(1)}</div>
                <div className="text-sm text-muted-foreground">Avg Events/Match</div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="timeline" className="space-y-4">
            <div className="space-y-4">
              <div className="p-4 bg-muted rounded-lg">
                <h4 className="font-semibold mb-2">Peak Activity Hour</h4>
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm">Peak Hour:</span>
                    <span className="text-sm font-medium">{formatHour(stats.peakHour)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Events in Peak Hour:</span>
                    <span className="text-sm font-medium">{stats.peakHourEvents}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Matches in Peak Hour:</span>
                    <span className="text-sm font-medium">
                      {stats.eventTimeline[stats.peakHour]?.matches || 0}
                    </span>
                  </div>
                </div>
              </div>

              <div className="space-y-2">
                <h4 className="font-semibold">Hourly Activity</h4>
                <div className="space-y-2">
                  {stats.eventTimeline.map((hourData, index) => (
                    <div key={index} className="flex items-center justify-between p-2 border rounded">
                      <span className="text-sm font-medium">{formatHour(hourData.hour)}</span>
                      <div className="flex items-center gap-4">
                        <span className="text-sm text-muted-foreground">{hourData.events} events</span>
                        <span className="text-sm text-muted-foreground">{hourData.matches} matches</span>
                        {hourData.events > 0 && (
                          <div className="w-20 bg-gray-200 rounded-full h-2">
                            <div 
                              className="bg-blue-600 h-2 rounded-full" 
                              style={{ width: `${(hourData.events / stats.peakHourEvents) * 100}%` }}
                            ></div>
                          </div>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="athletes" className="space-y-4">
            <div className="space-y-4">
              <h4 className="font-semibold">Top 10 Athletes of the Day</h4>
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
                        {athlete.matches} matches • {athlete.events} events
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </TabsContent>

          <TabsContent value="performance" className="space-y-4">
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-green-600">{stats.eventDistribution.points}</div>
                  <div className="text-sm text-muted-foreground">Points Events</div>
                </div>
                <div className="p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-orange-600">{stats.eventDistribution.warnings}</div>
                  <div className="text-sm text-muted-foreground">Warning Events</div>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-red-600">{stats.eventDistribution.injuries}</div>
                  <div className="text-sm text-muted-foreground">Injury Events</div>
                </div>
                <div className="p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-gray-600">{stats.eventDistribution.other}</div>
                  <div className="text-sm text-muted-foreground">Other Events</div>
                </div>
              </div>

              <div className="p-4 border rounded-lg">
                <h4 className="font-semibold mb-2">Day Performance Metrics</h4>
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm">Match Intensity:</span>
                    <span className="text-sm font-medium">{stats.matchIntensity.toFixed(2)} events/min</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Completion Rate:</span>
                    <span className="text-sm font-medium">{stats.dayEfficiency.toFixed(1)}%</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Average Match Duration:</span>
                    <span className="text-sm font-medium">{formatDuration(stats.averageMatchDuration)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Average Events per Match:</span>
                    <span className="text-sm font-medium">{stats.averageEventsPerMatch.toFixed(1)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Total Athletes:</span>
                    <span className="text-sm font-medium">{stats.totalAthletes}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Total Countries:</span>
                    <span className="text-sm font-medium">{stats.totalCountries}</span>
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
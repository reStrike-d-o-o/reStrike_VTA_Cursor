import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../atoms/Card';
import { Badge } from '../atoms/Badge';
import { Progress } from '../atoms/Progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../atoms/Tabs';
import { usePssMatchStore } from '../../stores/pssMatchStore';

interface MatchStats {
  matchId: string;
  duration: number;
  totalEvents: number;
  athlete1Stats: {
    name: string;
    countryCode?: string;
    points: number;
    warnings: number;
    injuries: number;
    events: number;
  };
  athlete2Stats: {
    name: string;
    countryCode?: string;
    points: number;
    warnings: number;
    injuries: number;
    events: number;
  };
  eventBreakdown: {
    points: number;
    warnings: number;
    injuries: number;
    other: number;
  };
  matchIntensity: number;
  winner?: string;
  isCompleted: boolean;
}

interface MatchAnalyticsProps {
  matchId: string;
  athlete1Name: string;
  athlete2Name: string;
  athlete1Country?: string;
  athlete2Country?: string;
}

export const MatchAnalytics: React.FC<MatchAnalyticsProps> = ({
  matchId,
  athlete1Name,
  athlete2Name,
  athlete1Country,
  athlete2Country,
}) => {
  const [stats, setStats] = useState<MatchStats>({
    matchId,
    duration: 0,
    totalEvents: 0,
    athlete1Stats: {
      name: athlete1Name,
      countryCode: athlete1Country,
      points: 0,
      warnings: 0,
      injuries: 0,
      events: 0,
    },
    athlete2Stats: {
      name: athlete2Name,
      countryCode: athlete2Country,
      points: 0,
      warnings: 0,
      injuries: 0,
      events: 0,
    },
    eventBreakdown: {
      points: 0,
      warnings: 0,
      injuries: 0,
      other: 0,
    },
    matchIntensity: 0,
    isCompleted: false,
  });

  const [isLoading, setIsLoading] = useState(true);
  const { matchData } = usePssMatchStore();
  
  // Mock events data for now - this should be replaced with actual PSS events
  const events = React.useRef<any[]>([]).current;

  useEffect(() => {
    const calculateMatchStats = () => {
      // Filter events for this specific match
      const matchEvents = events.filter((event: any) => event.match_id === matchId);
      
      if (matchEvents.length === 0) {
        setIsLoading(false);
        return;
      }

      let startTime: Date | null = null;
      let endTime: Date | null = null;
      let totalEvents = matchEvents.length;
      let points = 0;
      let warnings = 0;
      let injuries = 0;
      let other = 0;

      const athlete1Stats = {
        name: athlete1Name,
        countryCode: athlete1Country,
        points: 0,
        warnings: 0,
        injuries: 0,
        events: 0,
      };

      const athlete2Stats = {
        name: athlete2Name,
        countryCode: athlete2Country,
        points: 0,
        warnings: 0,
        injuries: 0,
        events: 0,
      };

      let winner: string | undefined;

      matchEvents.forEach(event => {
        const eventTime = new Date(event.timestamp);
        
        if (!startTime || eventTime < startTime) {
          startTime = eventTime;
        }
        if (!endTime || eventTime > endTime) {
          endTime = eventTime;
        }

        // Count event types
        if (event.event_type === 'point' || event.event_type === 'score') {
          points++;
          // Track points per athlete
          if (event.athlete1_code && event.score1) {
            athlete1Stats.points += event.score1;
            athlete1Stats.events++;
          }
          if (event.athlete2_code && event.score2) {
            athlete2Stats.points += event.score2;
            athlete2Stats.events++;
          }
        } else if (event.event_type === 'warning' || event.event_type === 'gam_jeom') {
          warnings++;
          if (event.athlete1_code) {
            athlete1Stats.warnings++;
            athlete1Stats.events++;
          }
          if (event.athlete2_code) {
            athlete2Stats.warnings++;
            athlete2Stats.events++;
          }
        } else if (event.event_type === 'injury') {
          injuries++;
          if (event.athlete1_code) {
            athlete1Stats.injuries++;
            athlete1Stats.events++;
          }
          if (event.athlete2_code) {
            athlete2Stats.injuries++;
            athlete2Stats.events++;
          }
        } else {
          other++;
          if (event.athlete1_code) athlete1Stats.events++;
          if (event.athlete2_code) athlete2Stats.events++;
        }

        // Determine winner
        if (event.event_type === 'match_end') {
          if (event.score1 && event.score2) {
            if (event.score1 > event.score2) {
              winner = athlete1Name;
            } else if (event.score2 > event.score1) {
              winner = athlete2Name;
            }
          }
        }
      });

      const duration = startTime && endTime ? ((endTime as Date).getTime() - (startTime as Date).getTime()) / 1000 : 0;
      const matchIntensity = duration > 0 ? totalEvents / duration : 0;

      setStats({
        matchId,
        duration,
        totalEvents,
        athlete1Stats,
        athlete2Stats,
        eventBreakdown: { points, warnings, injuries, other },
        matchIntensity,
        winner,
        isCompleted: winner !== undefined,
      });

      setIsLoading(false);
    };

    calculateMatchStats();
  }, [events, matchId, athlete1Name, athlete2Name, athlete1Country, athlete2Country]);

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
          <span>Match Analytics</span>
          <Badge variant={stats.isCompleted ? "default" : "secondary"}>
            {stats.isCompleted ? "Completed" : "In Progress"}
          </Badge>
          {stats.winner && (
            <Badge variant="outline" className="text-green-600">
              Winner: {stats.winner}
            </Badge>
          )}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="overview" className="w-full">
          <TabsList className="grid w-full grid-cols-4">
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="athletes">Athletes</TabsTrigger>
            <TabsTrigger value="events">Events</TabsTrigger>
            <TabsTrigger value="timeline">
              <img src="/icons/bar-graph.json" alt="Statistics" className="w-4 h-4 mr-2" />
              Statistics
            </TabsTrigger>
          </TabsList>

          <TabsContent value="overview" className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">Duration</span>
                  <span className="text-sm text-muted-foreground">{formatDuration(stats.duration)}</span>
                </div>
                <Progress value={stats.duration > 0 ? Math.min((stats.duration / 600) * 100, 100) : 0} />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">Total Events</span>
                  <span className="text-sm text-muted-foreground">{stats.totalEvents}</span>
                </div>
                <Progress value={stats.totalEvents > 0 ? Math.min((stats.totalEvents / 100) * 100, 100) : 0} />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">Match Intensity</span>
                  <span className="text-sm text-muted-foreground">{stats.matchIntensity.toFixed(2)} events/min</span>
                </div>
                <Progress value={stats.matchIntensity > 0 ? Math.min((stats.matchIntensity / 5) * 100, 100) : 0} />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">Points Scored</span>
                  <span className="text-sm text-muted-foreground">{stats.eventBreakdown.points}</span>
                </div>
                <Progress value={stats.eventBreakdown.points > 0 ? Math.min((stats.eventBreakdown.points / 50) * 100, 100) : 0} />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4 pt-4">
              <div className="p-4 border rounded-lg">
                <div className="flex items-center gap-2 mb-2">
                  {stats.athlete1Stats.countryCode && (
                    <span className="text-xl">
                      {stats.athlete1Stats.countryCode === 'RUS' ? '🇷🇺' : 
                       stats.athlete1Stats.countryCode === 'GER' ? '🇩🇪' : 
                       stats.athlete1Stats.countryCode === 'USA' ? '🇺🇸' : 
                       stats.athlete1Stats.countryCode === 'KOR' ? '🇰🇷' : '🏳️'}
                    </span>
                  )}
                  <span className="font-semibold">{stats.athlete1Stats.name}</span>
                </div>
                <div className="text-2xl font-bold text-blue-600">{stats.athlete1Stats.points}</div>
                <div className="text-sm text-muted-foreground">Points</div>
              </div>

              <div className="p-4 border rounded-lg">
                <div className="flex items-center gap-2 mb-2">
                  {stats.athlete2Stats.countryCode && (
                    <span className="text-xl">
                      {stats.athlete2Stats.countryCode === 'RUS' ? '🇷🇺' : 
                       stats.athlete2Stats.countryCode === 'GER' ? '🇩🇪' : 
                       stats.athlete2Stats.countryCode === 'USA' ? '🇺🇸' : 
                       stats.athlete2Stats.countryCode === 'KOR' ? '🇰🇷' : '🏳️'}
                    </span>
                  )}
                  <span className="font-semibold">{stats.athlete2Stats.name}</span>
                </div>
                <div className="text-2xl font-bold text-red-600">{stats.athlete2Stats.points}</div>
                <div className="text-sm text-muted-foreground">Points</div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="athletes" className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="p-4 border rounded-lg">
                <h4 className="font-semibold mb-3 flex items-center gap-2">
                  {stats.athlete1Stats.countryCode && (
                    <span className="text-xl">
                      {stats.athlete1Stats.countryCode === 'RUS' ? '🇷🇺' : 
                       stats.athlete1Stats.countryCode === 'GER' ? '🇩🇪' : 
                       stats.athlete1Stats.countryCode === 'USA' ? '🇺🇸' : 
                       stats.athlete1Stats.countryCode === 'KOR' ? '🇰🇷' : '🏳️'}
                    </span>
                  )}
                  {stats.athlete1Stats.name}
                </h4>
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm">Points:</span>
                    <span className="text-sm font-medium">{stats.athlete1Stats.points}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Warnings:</span>
                    <span className="text-sm font-medium">{stats.athlete1Stats.warnings}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Injuries:</span>
                    <span className="text-sm font-medium">{stats.athlete1Stats.injuries}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Total Events:</span>
                    <span className="text-sm font-medium">{stats.athlete1Stats.events}</span>
                  </div>
                </div>
              </div>

              <div className="p-4 border rounded-lg">
                <h4 className="font-semibold mb-3 flex items-center gap-2">
                  {stats.athlete2Stats.countryCode && (
                    <span className="text-xl">
                      {stats.athlete2Stats.countryCode === 'RUS' ? '🇷🇺' : 
                       stats.athlete2Stats.countryCode === 'GER' ? '🇩🇪' : 
                       stats.athlete2Stats.countryCode === 'USA' ? '🇺🇸' : 
                       stats.athlete2Stats.countryCode === 'KOR' ? '🇰🇷' : '🏳️'}
                    </span>
                  )}
                  {stats.athlete2Stats.name}
                </h4>
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm">Points:</span>
                    <span className="text-sm font-medium">{stats.athlete2Stats.points}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Warnings:</span>
                    <span className="text-sm font-medium">{stats.athlete2Stats.warnings}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Injuries:</span>
                    <span className="text-sm font-medium">{stats.athlete2Stats.injuries}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Total Events:</span>
                    <span className="text-sm font-medium">{stats.athlete2Stats.events}</span>
                  </div>
                </div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="events" className="space-y-4">
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-green-600">{stats.eventBreakdown.points}</div>
                  <div className="text-sm text-muted-foreground">Points Events</div>
                </div>
                <div className="p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-orange-600">{stats.eventBreakdown.warnings}</div>
                  <div className="text-sm text-muted-foreground">Warning Events</div>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-red-600">{stats.eventBreakdown.injuries}</div>
                  <div className="text-sm text-muted-foreground">Injury Events</div>
                </div>
                <div className="p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-gray-600">{stats.eventBreakdown.other}</div>
                  <div className="text-sm text-muted-foreground">Other Events</div>
                </div>
              </div>

              <div className="p-4 border rounded-lg">
                <h4 className="font-semibold mb-2">Event Distribution</h4>
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm">Points:</span>
                    <span className="text-sm font-medium">
                      {stats.totalEvents > 0 ? ((stats.eventBreakdown.points / stats.totalEvents) * 100).toFixed(1) : 0}%
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Warnings:</span>
                    <span className="text-sm font-medium">
                      {stats.totalEvents > 0 ? ((stats.eventBreakdown.warnings / stats.totalEvents) * 100).toFixed(1) : 0}%
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Injuries:</span>
                    <span className="text-sm font-medium">
                      {stats.totalEvents > 0 ? ((stats.eventBreakdown.injuries / stats.totalEvents) * 100).toFixed(1) : 0}%
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Other:</span>
                    <span className="text-sm font-medium">
                      {stats.totalEvents > 0 ? ((stats.eventBreakdown.other / stats.totalEvents) * 100).toFixed(1) : 0}%
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="timeline" className="space-y-4">
            <div className="space-y-4">
              <div className="p-4 bg-muted rounded-lg">
                <h4 className="font-semibold mb-2">Match Timeline</h4>
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm">Start Time:</span>
                    <span className="text-sm font-medium">
                      {stats.duration > 0 ? (()=>{ const d=new Date(Date.now() - stats.duration*1000); const hh=String(d.getHours()).padStart(2,'0'); const mi=String(d.getMinutes()).padStart(2,'0'); const ss=String(d.getSeconds()).padStart(2,'0'); return `${hh}:${mi}:${ss}`; })() : 'N/A'}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">End Time:</span>
                    <span className="text-sm font-medium">
                      {stats.duration > 0 ? (()=>{ const d=new Date(); const hh=String(d.getHours()).padStart(2,'0'); const mi=String(d.getMinutes()).padStart(2,'0'); const ss=String(d.getSeconds()).padStart(2,'0'); return `${hh}:${mi}:${ss}`; })() : 'N/A'}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Duration:</span>
                    <span className="text-sm font-medium">{formatDuration(stats.duration)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Events per Minute:</span>
                    <span className="text-sm font-medium">{stats.matchIntensity.toFixed(2)}</span>
                  </div>
                </div>
              </div>

              <div className="p-4 border rounded-lg">
                <h4 className="font-semibold mb-2">Match Status</h4>
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm">Status:</span>
                    <Badge variant={stats.isCompleted ? "default" : "secondary"}>
                      {stats.isCompleted ? "Completed" : "In Progress"}
                    </Badge>
                  </div>
                  {stats.winner && (
                    <div className="flex justify-between">
                      <span className="text-sm">Winner:</span>
                      <span className="text-sm font-medium text-green-600">{stats.winner}</span>
                    </div>
                  )}
                  <div className="flex justify-between">
                    <span className="text-sm">Match ID:</span>
                    <span className="text-sm font-mono">{stats.matchId}</span>
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
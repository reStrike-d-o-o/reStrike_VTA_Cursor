/**
 * AthleteAnalytics
 * - Athlete-level analytics view
 */
import React, { useMemo, useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../atoms/Card';
import { Badge } from '../atoms/Badge';
import { Progress } from '../atoms/Progress';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../atoms/Tabs';
import { usePssMatchStore } from '../../stores/pssMatchStore';

interface AthleteStats {
  totalMatches: number;
  wins: number;
  losses: number;
  totalPoints: number;
  averagePointsPerMatch: number;
  totalWarnings: number;
  totalInjuries: number;
  winRate: number;
  lastMatchDate?: string;
  bestPerformance?: {
    matchId: string;
    points: number;
    date: string;
  };
}

interface AthleteAnalyticsProps {
  athleteId: string;
  athleteName: string;
  countryCode?: string;
}

export const AthleteAnalytics: React.FC<AthleteAnalyticsProps> = ({
  athleteId,
  athleteName,
  countryCode,
}) => {
  const [stats, setStats] = useState<AthleteStats>({
    totalMatches: 0,
    wins: 0,
    losses: 0,
    totalPoints: 0,
    averagePointsPerMatch: 0,
    totalWarnings: 0,
    totalInjuries: 0,
    winRate: 0,
  });

  const [isLoading, setIsLoading] = useState(true);
  const { matchData } = usePssMatchStore();
  
  // Mock events data for now - this should be replaced with actual PSS events
  const events = React.useRef<any[]>([]).current;

  useEffect(() => {
    const calculateStats = () => {
      // Filter events for this athlete
      const athleteEvents = events.filter((event: any) => 
        event.athlete1_code === athleteId || event.athlete2_code === athleteId
      );

      // Group events by match
      const matchGroups = new Map<string, any[]>();
      athleteEvents.forEach(event => {
        if (event.match_id) {
          if (!matchGroups.has(event.match_id)) {
            matchGroups.set(event.match_id, []);
          }
          matchGroups.get(event.match_id)!.push(event);
        }
      });

      let totalMatches = matchGroups.size;
      let wins = 0;
      let losses = 0;
      let totalPoints = 0;
      let totalWarnings = 0;
      let totalInjuries = 0;
      let bestPerformance = { matchId: '', points: 0, date: '' };

      // Analyze each match
      matchGroups.forEach((matchEvents, matchId) => {
        let matchPoints = 0;
        let matchWarnings = 0;
        let matchInjuries = 0;
        let isWinner = false;

        matchEvents.forEach((event: any) => {
          // Count points
          if (event.event_type === 'point' || event.event_type === 'score') {
            if (event.athlete1_code === athleteId && event.score1) {
              matchPoints += event.score1;
            } else if (event.athlete2_code === athleteId && event.score2) {
              matchPoints += event.score2;
            }
          }

          // Count warnings
          if (event.event_type === 'warning' || event.event_type === 'gam_jeom') {
            if (event.athlete1_code === athleteId || event.athlete2_code === athleteId) {
              matchWarnings++;
            }
          }

          // Count injuries
          if (event.event_type === 'injury') {
            if (event.athlete1_code === athleteId || event.athlete2_code === athleteId) {
              matchInjuries++;
            }
          }

          // Determine winner (simplified logic)
          if (event.event_type === 'match_end') {
            if (event.athlete1_code === athleteId && event.score1 && event.score2 && event.score1 > event.score2) {
              isWinner = true;
            } else if (event.athlete2_code === athleteId && event.score1 && event.score2 && event.score2 > event.score1) {
              isWinner = true;
            }
          }
        });

        totalPoints += matchPoints;
        totalWarnings += matchWarnings;
        totalInjuries += matchInjuries;

        if (isWinner) {
          wins++;
        } else if (matchEvents.length > 0) {
          losses++;
        }

        // Track best performance
        if (matchPoints > bestPerformance.points) {
          bestPerformance = {
            matchId,
            points: matchPoints,
            date: matchEvents[0]?.timestamp || '',
          };
        }
      });

      const winRate = totalMatches > 0 ? (wins / totalMatches) * 100 : 0;
      const averagePointsPerMatch = totalMatches > 0 ? totalPoints / totalMatches : 0;

      setStats({
        totalMatches,
        wins,
        losses,
        totalPoints,
        averagePointsPerMatch,
        totalWarnings,
        totalInjuries,
        winRate,
        bestPerformance: bestPerformance.points > 0 ? bestPerformance : undefined,
      });

      setIsLoading(false);
    };

    calculateStats();
  }, [events, athleteId]);

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

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          {countryCode && (
            <span className="text-2xl">
              {countryCode === 'RUS' ? '🇷🇺' : 
               countryCode === 'GER' ? '🇩🇪' : 
               countryCode === 'USA' ? '🇺🇸' : 
               countryCode === 'KOR' ? '🇰🇷' : '🏳️'}
            </span>
          )}
          <span>{athleteName}</span>
          <Badge variant="secondary">Analytics</Badge>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="overview" className="w-full">
          <TabsList className="grid w-full grid-cols-4">
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="performance">Performance</TabsTrigger>
            <TabsTrigger value="matches">Matches</TabsTrigger>
            <TabsTrigger value="trends">
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
                  <span className="text-sm font-medium">Win Rate</span>
                  <span className="text-sm text-muted-foreground">{stats.winRate.toFixed(1)}%</span>
                </div>
                <Progress value={stats.winRate} />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">Total Points</span>
                  <span className="text-sm text-muted-foreground">{stats.totalPoints}</span>
                </div>
                <Progress value={stats.totalPoints > 0 ? Math.min((stats.totalPoints / 1000) * 100, 100) : 0} />
              </div>

              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-sm font-medium">Avg Points/Match</span>
                  <span className="text-sm text-muted-foreground">{stats.averagePointsPerMatch.toFixed(1)}</span>
                </div>
                <Progress value={stats.averagePointsPerMatch > 0 ? Math.min((stats.averagePointsPerMatch / 50) * 100, 100) : 0} />
              </div>
            </div>

            <div className="grid grid-cols-3 gap-4 pt-4">
              <div className="text-center">
                <div className="text-2xl font-bold text-green-600">{stats.wins}</div>
                <div className="text-sm text-muted-foreground">Wins</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-red-600">{stats.losses}</div>
                <div className="text-sm text-muted-foreground">Losses</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-blue-600">{stats.totalWarnings}</div>
                <div className="text-sm text-muted-foreground">Warnings</div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="performance" className="space-y-4">
            <div className="space-y-4">
              <div className="p-4 bg-muted rounded-lg">
                <h4 className="font-semibold mb-2">Best Performance</h4>
                {stats.bestPerformance ? (
                  <div className="space-y-1">
                    <div className="flex justify-between">
                      <span className="text-sm">Match Points:</span>
                      <span className="text-sm font-medium">{stats.bestPerformance.points}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm">Match ID:</span>
                      <span className="text-sm font-mono">{stats.bestPerformance.matchId}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm">Date:</span>
                      <span className="text-sm">{require('../../utils/format').formatDateTime(stats.bestPerformance.date)}</span>
                    </div>
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground">No performance data available</p>
                )}
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="p-4 border rounded-lg">
                  <div className="text-2xl font-bold text-green-600">{stats.totalPoints}</div>
                  <div className="text-sm text-muted-foreground">Total Points Scored</div>
                </div>
                <div className="p-4 border rounded-lg">
                  <div className="text-2xl font-bold text-orange-600">{stats.totalWarnings}</div>
                  <div className="text-sm text-muted-foreground">Total Warnings</div>
                </div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="matches" className="space-y-4">
            <div className="space-y-2">
              <h4 className="font-semibold">Recent Match History</h4>
              <div className="space-y-2">
                {stats.totalMatches > 0 ? (
                  <div className="text-sm text-muted-foreground">
                    {stats.wins} wins, {stats.losses} losses in {stats.totalMatches} total matches
                  </div>
                ) : (
                  <div className="text-sm text-muted-foreground">No match history available</div>
                )}
              </div>
            </div>
          </TabsContent>

          <TabsContent value="trends" className="space-y-4">
            <div className="space-y-4">
              <div className="p-4 bg-muted rounded-lg">
                <h4 className="font-semibold mb-2">Performance Trends</h4>
                <div className="space-y-2">
                  <div className="flex justify-between">
                    <span className="text-sm">Win Rate Trend:</span>
                    <span className={`text-sm font-medium ${stats.winRate > 50 ? 'text-green-600' : 'text-red-600'}`}>
                      {stats.winRate > 50 ? '↗️ Improving' : '↘️ Declining'}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Points Per Match:</span>
                    <span className="text-sm font-medium">{stats.averagePointsPerMatch.toFixed(1)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm">Discipline:</span>
                    <span className={`text-sm font-medium ${stats.totalWarnings < 5 ? 'text-green-600' : 'text-red-600'}`}>
                      {stats.totalWarnings < 5 ? 'Good' : 'Needs Improvement'}
                    </span>
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
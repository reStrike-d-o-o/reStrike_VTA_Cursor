import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { useAppStore } from '../../stores';
import Button from '../atoms/Button';
import Label from '../atoms/Label';
import { StatusDot } from '../atoms/StatusDot';
import { Icon } from '../atoms/Icon';

interface Event {
  id: string;
  timestamp: string;
  type: 'point' | 'warning' | 'clock' | 'round' | 'score' | 'athlete';
  player: 'RED' | 'BLUE' | 'YELLOW' | 'NONE';
  description: string;
  value?: string;
}

const SidebarTest: React.FC = () => {
  const { obsConnections } = useAppStore();
  const [events, setEvents] = useState<Event[]>([]);
  const [filteredEvents, setFilteredEvents] = useState<Event[]>([]);
  const [selectedPlayer, setSelectedPlayer] = useState<'ALL' | 'RED' | 'BLUE' | 'YELLOW'>('ALL');
  const [selectedType, setSelectedType] = useState<'ALL' | Event['type']>('ALL');

  // Windows-specific initialization
  useEffect(() => {
    console.log('Initializing Windows-only sidebar component');
    initializeWindowsFeatures();
  }, []);

  const initializeWindowsFeatures = async () => {
    try {
      // Initialize Tauri commands for real-time data
      if (window.__TAURI__) {
        console.log('✅ Tauri environment detected for sidebar');
        
        // Initialize PSS protocol listener
        // Initialize real-time event processing
        // Initialize OBS status monitoring
      }
    } catch (error) {
      console.error('❌ Failed to initialize Windows features:', error);
    }
  };

  // Generate sample events for demonstration
  useEffect(() => {
    const sampleEvents: Event[] = [
      {
        id: '1',
        timestamp: '14:30:15',
        type: 'point',
        player: 'RED',
        description: 'Point scored',
        value: '3 points'
      },
      {
        id: '2',
        timestamp: '14:30:20',
        type: 'warning',
        player: 'BLUE',
        description: 'Warning issued',
        value: 'Kyong-go'
      },
      {
        id: '3',
        timestamp: '14:30:25',
        type: 'clock',
        player: 'NONE',
        description: 'Time remaining',
        value: '1:45'
      },
      {
        id: '4',
        timestamp: '14:30:30',
        type: 'round',
        player: 'NONE',
        description: 'Round start',
        value: 'Round 2'
      },
      {
        id: '5',
        timestamp: '14:30:35',
        type: 'score',
        player: 'RED',
        description: 'Score update',
        value: '15-12'
      }
    ];
    setEvents(sampleEvents);
    setFilteredEvents(sampleEvents);
  }, []);

  // Filter events based on selection
  useEffect(() => {
    let filtered = events;
    
    if (selectedPlayer !== 'ALL') {
      filtered = filtered.filter(event => event.player === selectedPlayer);
    }
    
    if (selectedType !== 'ALL') {
      filtered = filtered.filter(event => event.type === selectedType);
    }
    
    setFilteredEvents(filtered);
  }, [events, selectedPlayer, selectedType]);

  const clearFilters = () => {
    setSelectedPlayer('ALL');
    setSelectedType('ALL');
  };

  const getPlayerColor = (player: string) => {
    switch (player) {
      case 'RED': return 'text-red-400';
      case 'BLUE': return 'text-blue-400';
      case 'YELLOW': return 'text-yellow-400';
      default: return 'text-gray-400';
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'point': return 'point';
      case 'warning': return 'warning';
      case 'clock': return 'clock';
      case 'round': return 'round';
      case 'score': return 'score';
      case 'athlete': return 'athlete';
      default: return 'default';
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-gray-800 rounded-lg p-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold text-white">Event Sidebar</h2>
            <p className="text-gray-400 mt-1">Real-time competition events and status</p>
          </div>
          <div className="flex items-center space-x-4">
            <span className="px-3 py-1 bg-blue-600 text-white text-sm rounded-full">
              Windows Native
            </span>
            <span className="px-3 py-1 bg-green-600 text-white text-sm rounded-full">
              {filteredEvents.length} Events
            </span>
          </div>
        </div>
      </div>

      {/* OBS Status */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">OBS Status</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {obsConnections.map((connection) => (
            <div key={connection.name} className="flex items-center justify-between p-3 bg-gray-700 rounded-lg">
              <div className="flex items-center space-x-3">
                <StatusDot color={
                  connection.status === 'Connected' || connection.status === 'Authenticated'
                    ? 'bg-green-500'
                    : connection.status === 'Error'
                    ? 'bg-red-500'
                    : 'bg-yellow-500'
                } />
                <span className="text-white font-medium">{connection.name}</span>
              </div>
              <span className="text-gray-400 text-sm">{connection.status}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Filters */}
      <div className="bg-gray-800 rounded-lg p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Event Filters</h3>
          <Button
            onClick={clearFilters}
            variant="secondary"
            size="sm"
          >
            Clear Filters
          </Button>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* Player Filter */}
          <div>
            <Label htmlFor="player-filter" className="block text-sm font-medium text-gray-300 mb-2">Player</Label>
            <select
              id="player-filter"
              value={selectedPlayer}
              onChange={(e) => setSelectedPlayer(e.target.value as any)}
              className="w-full bg-gray-700 border border-gray-600 text-white rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
              title="Sidebar option"
              aria-label="Sidebar option"
            >
              <option value="ALL">All Players</option>
              <option value="RED">Red Player</option>
              <option value="BLUE">Blue Player</option>
              <option value="YELLOW">Yellow Player</option>
            </select>
          </div>

          {/* Type Filter */}
          <div>
            <Label htmlFor="type-filter" className="block text-sm font-medium text-gray-300 mb-2">Event Type</Label>
            <select
              id="type-filter"
              value={selectedType}
              onChange={(e) => setSelectedType(e.target.value as any)}
              className="w-full bg-gray-700 border border-gray-600 text-white rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
              title="Sidebar option"
              aria-label="Sidebar option"
            >
              <option value="ALL">All Events</option>
              <option value="point">Points</option>
              <option value="warning">Warnings</option>
              <option value="clock">Clock</option>
              <option value="round">Rounds</option>
              <option value="score">Scores</option>
              <option value="athlete">Athletes</option>
            </select>
          </div>
        </div>
      </div>

      {/* Events Table */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Recent Events</h3>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="text-left py-3 px-4 text-gray-300 font-medium">Time</th>
                <th className="text-left py-3 px-4 text-gray-300 font-medium">Type</th>
                <th className="text-left py-3 px-4 text-gray-300 font-medium">Player</th>
                <th className="text-left py-3 px-4 text-gray-300 font-medium">Description</th>
                <th className="text-left py-3 px-4 text-gray-300 font-medium">Value</th>
              </tr>
            </thead>
            <tbody>
              {filteredEvents.map((event) => (
                <motion.tr
                  key={event.id}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  className="border-b border-gray-700 hover:bg-gray-700 transition-colors"
                >
                  <td className="py-3 px-4 text-gray-400 font-mono text-sm">
                    {event.timestamp}
                  </td>
                  <td className="py-3 px-4">
                    <span className="flex items-center space-x-2">
                      {getTypeIcon(event.type) === 'point' && (
                        <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="2"/>
                          <circle cx="12" cy="12" r="3" fill="currentColor"/>
                        </svg>
                      )}
                      {getTypeIcon(event.type) === 'warning' && (
                        <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
                        </svg>
                      )}
                      {getTypeIcon(event.type) === 'clock' && (
                        <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="2"/>
                          <polyline points="12,6 12,12 16,14" stroke="currentColor" strokeWidth="2"/>
                        </svg>
                      )}
                      {getTypeIcon(event.type) === 'round' && (
                        <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                        </svg>
                      )}
                      {getTypeIcon(event.type) === 'score' && (
                        <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                        </svg>
                      )}
                      {getTypeIcon(event.type) === 'athlete' && (
                        <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                        </svg>
                      )}
                      {getTypeIcon(event.type) === 'default' && (
                        <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                        </svg>
                      )}
                      <span className="text-white capitalize">{event.type}</span>
                    </span>
                  </td>
                  <td className={`py-3 px-4 ${getPlayerColor(event.player)}`}>{event.player}</td>
                  <td className="py-3 px-4 text-gray-300">{event.description}</td>
                  <td className="py-3 px-4 text-gray-300">{event.value}</td>
                </motion.tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};

export default SidebarTest; 
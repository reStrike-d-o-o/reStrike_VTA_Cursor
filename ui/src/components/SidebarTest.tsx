import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { useAppStore } from '../stores';
import { createComponentLogger } from '../utils/logger';

const logger = createComponentLogger('SidebarTest');

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
    logger.info('Initializing Windows-only sidebar component');
    initializeWindowsFeatures();
  }, []);

  const initializeWindowsFeatures = async () => {
    try {
      // Initialize Tauri commands for real-time data
      if (window.__TAURI__) {
        logger.info('✅ Tauri environment detected for sidebar');
        
        // Initialize PSS protocol listener
        // Initialize real-time event processing
        // Initialize OBS status monitoring
      }
    } catch (error) {
      logger.error('❌ Failed to initialize Windows features:', error);
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
      case 'point': return '🎯';
      case 'warning': return '⚠️';
      case 'clock': return '⏰';
      case 'round': return '🔄';
      case 'score': return '📊';
      case 'athlete': return '👤';
      default: return '📝';
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
                <div className={`w-3 h-3 rounded-full ${
                  connection.status === 'Connected' || connection.status === 'Authenticated'
                    ? 'bg-green-500'
                    : connection.status === 'Error'
                    ? 'bg-red-500'
                    : 'bg-yellow-500'
                }`} />
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
          <button
            onClick={clearFilters}
            className="flex items-center space-x-2 px-3 py-1 bg-gray-600 hover:bg-gray-500 text-white text-sm rounded transition-colors"
          >
            <span>↑</span>
            <span>Clear Filters</span>
          </button>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {/* Player Filter */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">Player</label>
            <select
              value={selectedPlayer}
              onChange={(e) => setSelectedPlayer(e.target.value as any)}
              className="w-full bg-gray-700 border border-gray-600 text-white rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="ALL">All Players</option>
              <option value="RED">Red Player</option>
              <option value="BLUE">Blue Player</option>
              <option value="YELLOW">Yellow Player</option>
            </select>
          </div>

          {/* Type Filter */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">Event Type</label>
            <select
              value={selectedType}
              onChange={(e) => setSelectedType(e.target.value as any)}
              className="w-full bg-gray-700 border border-gray-600 text-white rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
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
                      <span>{getTypeIcon(event.type)}</span>
                      <span className="text-white capitalize">{event.type}</span>
                    </span>
                  </td>
                  <td className="py-3 px-4">
                    <span className={`font-medium ${getPlayerColor(event.player)}`}>
                      {event.player}
                    </span>
                  </td>
                  <td className="py-3 px-4 text-white">
                    {event.description}
                  </td>
                  <td className="py-3 px-4 text-gray-400">
                    {event.value || '-'}
                  </td>
                </motion.tr>
              ))}
            </tbody>
          </table>
        </div>
        
        {filteredEvents.length === 0 && (
          <div className="text-center py-8 text-gray-400">
            No events match the current filters
          </div>
        )}
      </div>
    </div>
  );
};

export default SidebarTest;
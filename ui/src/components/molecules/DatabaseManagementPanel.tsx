import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import Input from '../atoms/Input';
import Label from '../atoms/Label';
import StatusDot from '../atoms/StatusDot';
import TabGroup from '../molecules/TabGroup';
import TabIcons from '../atoms/TabIcons';

interface DatabaseStats {
  totalEvents: number;
  totalConnections: number;
  totalConfigs: number;
  totalFlags: number;
  databaseSize: string;
  encoding: string;
}

interface PssEvent {
  id: number;
  event_type: string;
  timestamp: string;
  player_name: string;
  country_code: string;
  match_id: string;
  details: string;
}

interface ObsConnection {
  id: number;
  name: string;
  host: string;
  port: number;
  password: string;
  is_active: boolean;
  created_at: string;
}

interface AppConfig {
  id: number;
  key: string;
  value: string;
  category: string;
  updated_at: string;
}

interface FlagMapping {
  id: number;
  pss_code: string;
  ioc_code: string;
  country_name: string;
  flag_url: string;
}

const DatabaseManagementPanel: React.FC = () => {
  const [activeTab, setActiveTab] = useState('overview');
  const [stats, setStats] = useState<DatabaseStats | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [pssEvents, setPssEvents] = useState<PssEvent[]>([]);
  const [obsConnections, setObsConnections] = useState<ObsConnection[]>([]);
  const [appConfigs, setAppConfigs] = useState<AppConfig[]>([]);
  const [flagMappings, setFlagMappings] = useState<FlagMapping[]>([]);
  const [searchTerm, setSearchTerm] = useState('');

  // Load database statistics
  const loadStats = async () => {
    try {
      setIsLoading(true);
      // TODO: Replace with actual Tauri commands
      const mockStats: DatabaseStats = {
        totalEvents: 1250,
        totalConnections: 3,
        totalConfigs: 15,
        totalFlags: 253,
        databaseSize: '2.4 MB',
        encoding: 'UTF-8'
      };
      setStats(mockStats);
    } catch (error) {
      console.error('Failed to load database stats:', error);
    } finally {
      setIsLoading(false);
    }
  };

  // Load PSS events
  const loadPssEvents = async () => {
    try {
      // TODO: Replace with actual Tauri commands
      const mockEvents: PssEvent[] = [
        {
          id: 1,
          event_type: 'GOAL',
          timestamp: '2024-01-28 14:30:25',
          player_name: 'José Martínez',
          country_code: 'ESP',
          match_id: 'MATCH_001',
          details: 'Goal scored from penalty kick'
        },
        {
          id: 2,
          event_type: 'YELLOW_CARD',
          timestamp: '2024-01-28 14:32:10',
          player_name: 'Björk Andersson',
          country_code: 'SWE',
          match_id: 'MATCH_001',
          details: 'Foul committed in midfield'
        }
      ];
      setPssEvents(mockEvents);
    } catch (error) {
      console.error('Failed to load PSS events:', error);
    }
  };

  // Load OBS connections
  const loadObsConnections = async () => {
    try {
      // TODO: Replace with actual Tauri commands
      const mockConnections: ObsConnection[] = [
        {
          id: 1,
          name: 'Main Studio',
          host: '192.168.1.100',
          port: 4455,
          password: '****',
          is_active: true,
          created_at: '2024-01-15 10:00:00'
        },
        {
          id: 2,
          name: 'Backup Studio',
          host: '192.168.1.101',
          port: 4455,
          password: '****',
          is_active: false,
          created_at: '2024-01-20 14:30:00'
        }
      ];
      setObsConnections(mockConnections);
    } catch (error) {
      console.error('Failed to load OBS connections:', error);
    }
  };

  // Load app configs
  const loadAppConfigs = async () => {
    try {
      // TODO: Replace with actual Tauri commands
      const mockConfigs: AppConfig[] = [
        {
          id: 1,
          key: 'udp_port',
          value: '8888',
          category: 'network',
          updated_at: '2024-01-28 12:00:00'
        },
        {
          id: 2,
          key: 'auto_connect',
          value: 'true',
          category: 'obs',
          updated_at: '2024-01-28 12:00:00'
        }
      ];
      setAppConfigs(mockConfigs);
    } catch (error) {
      console.error('Failed to load app configs:', error);
    }
  };

  // Load flag mappings
  const loadFlagMappings = async () => {
    try {
      // TODO: Replace with actual Tauri commands
      const mockFlags: FlagMapping[] = [
        {
          id: 1,
          pss_code: 'ESP',
          ioc_code: 'ESP',
          country_name: 'Spain',
          flag_url: '/assets/flags/ESP.png'
        },
        {
          id: 2,
          pss_code: 'SWE',
          ioc_code: 'SWE',
          country_name: 'Sweden',
          flag_url: '/assets/flags/SWE.png'
        }
      ];
      setFlagMappings(mockFlags);
    } catch (error) {
      console.error('Failed to load flag mappings:', error);
    }
  };

  // Load data based on active tab
  useEffect(() => {
    loadStats();
    if (activeTab === 'pss-events') {
      loadPssEvents();
    } else if (activeTab === 'obs-connections') {
      loadObsConnections();
    } else if (activeTab === 'app-configs') {
      loadAppConfigs();
    } else if (activeTab === 'flag-mappings') {
      loadFlagMappings();
    }
  }, [activeTab]);

  // Filter data based on search term
  const filteredPssEvents = pssEvents.filter(event =>
    event.player_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    event.event_type.toLowerCase().includes(searchTerm.toLowerCase()) ||
    event.country_code.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const filteredObsConnections = obsConnections.filter(conn =>
    conn.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    conn.host.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const filteredAppConfigs = appConfigs.filter(config =>
    config.key.toLowerCase().includes(searchTerm.toLowerCase()) ||
    config.category.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const filteredFlagMappings = flagMappings.filter(flag =>
    flag.country_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    flag.pss_code.toLowerCase().includes(searchTerm.toLowerCase()) ||
    flag.ioc_code.toLowerCase().includes(searchTerm.toLowerCase())
  );

  // Helper function to get status color
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'connected':
        return 'bg-green-500';
      case 'disconnected':
        return 'bg-red-500';
      default:
        return 'bg-gray-500';
    }
  };

  return (
    <div className="space-y-6">
      <TabGroup
        tabs={[
          {
            id: 'overview',
            label: 'Overview',
            icon: TabIcons.overview,
            content: (
              <div className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                  <div className="bg-gradient-to-br from-blue-900/20 to-blue-800/30 backdrop-blur-sm rounded-lg p-4 border border-blue-600/30">
                                          <div className="flex items-center justify-between">
                        <div>
                          <p className="text-sm text-gray-400">PSS Events</p>
                          <p className="text-2xl font-bold text-blue-200">{stats?.totalEvents || 0}</p>
                        </div>
                        <StatusDot color="bg-blue-500" />
                      </div>
                  </div>
                  <div className="bg-gradient-to-br from-green-900/20 to-green-800/30 backdrop-blur-sm rounded-lg p-4 border border-green-600/30">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-gray-400">OBS Connections</p>
                        <p className="text-2xl font-bold text-green-200">{stats?.totalConnections || 0}</p>
                      </div>
                      <StatusDot color="bg-green-500" />
                    </div>
                  </div>
                  <div className="bg-gradient-to-br from-purple-900/20 to-purple-800/30 backdrop-blur-sm rounded-lg p-4 border border-purple-600/30">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-gray-400">App Configs</p>
                        <p className="text-2xl font-bold text-purple-200">{stats?.totalConfigs || 0}</p>
                      </div>
                      <StatusDot color="bg-purple-500" />
                    </div>
                  </div>
                  <div className="bg-gradient-to-br from-yellow-900/20 to-yellow-800/30 backdrop-blur-sm rounded-lg p-4 border border-yellow-600/30">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-gray-400">Flag Mappings</p>
                        <p className="text-2xl font-bold text-yellow-200">{stats?.totalFlags || 0}</p>
                      </div>
                      <StatusDot color="bg-yellow-500" />
                    </div>
                  </div>
                </div>
                
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-6 border border-gray-600/30">
                    <h3 className="text-lg font-semibold mb-4 text-gray-100">Database Info</h3>
                    <div className="space-y-3">
                      <div className="flex justify-between">
                        <span className="text-gray-400">Size:</span>
                        <span className="text-gray-200">{stats?.databaseSize || 'Unknown'}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-400">Encoding:</span>
                        <span className="text-gray-200">{stats?.encoding || 'Unknown'}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-400">Status:</span>
                        <StatusDot color="bg-green-500" />
                      </div>
                    </div>
                  </div>
                  
                  <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg p-6 border border-gray-600/30">
                    <h3 className="text-lg font-semibold mb-4 text-gray-100">Quick Actions</h3>
                    <div className="space-y-3">
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={() => setActiveTab('pss-events')}
                        className="w-full"
                      >
                        View PSS Events
                      </Button>
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={() => setActiveTab('obs-connections')}
                        className="w-full"
                      >
                        Manage OBS Connections
                      </Button>
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={() => setActiveTab('flag-mappings')}
                        className="w-full"
                      >
                        View Flag Mappings
                      </Button>
                    </div>
                  </div>
                </div>
              </div>
            )
          },
          {
            id: 'pss-events',
            label: 'PSS Events',
            icon: TabIcons.events,
            content: (
              <div className="space-y-4">
                <div className="flex justify-between items-center">
                  <h3 className="text-lg font-semibold text-gray-100">PSS Events</h3>
                  <div className="flex gap-2">
                    <Input
                      type="text"
                      placeholder="Search events..."
                      value={searchTerm}
                      onChange={(e) => setSearchTerm(e.target.value)}
                      className="w-64"
                    />
                    <Button variant="secondary" size="sm">
                      Export
                    </Button>
                  </div>
                </div>
                
                <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 overflow-hidden">
                  <div className="overflow-x-auto">
                    <table className="w-full">
                      <thead className="bg-gray-700/50">
                        <tr>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">ID</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Event Type</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Timestamp</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Player</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Country</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Match ID</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Details</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-gray-700">
                        {filteredPssEvents.map((event) => (
                          <tr key={event.id} className="hover:bg-gray-700/30">
                            <td className="px-4 py-3 text-sm text-gray-300">{event.id}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">
                              <span className={`px-2 py-1 text-xs rounded-full ${
                                event.event_type === 'GOAL' ? 'bg-green-900/50 text-green-200' :
                                event.event_type === 'YELLOW_CARD' ? 'bg-yellow-900/50 text-yellow-200' :
                                'bg-gray-700/50 text-gray-200'
                              }`}>
                                {event.event_type}
                              </span>
                            </td>
                            <td className="px-4 py-3 text-sm text-gray-300">{event.timestamp}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">{event.player_name}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">{event.country_code}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">{event.match_id}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">{event.details}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              </div>
            )
          },
          {
            id: 'obs-connections',
            label: 'OBS Connections',
            icon: TabIcons.websocket,
            content: (
              <div className="space-y-4">
                <div className="flex justify-between items-center">
                  <h3 className="text-lg font-semibold text-gray-100">OBS Connections</h3>
                  <div className="flex gap-2">
                    <Input
                      type="text"
                      placeholder="Search connections..."
                      value={searchTerm}
                      onChange={(e) => setSearchTerm(e.target.value)}
                      className="w-64"
                    />
                    <Button variant="primary" size="sm">
                      Add Connection
                    </Button>
                  </div>
                </div>
                
                <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 overflow-hidden">
                  <div className="overflow-x-auto">
                    <table className="w-full">
                      <thead className="bg-gray-700/50">
                        <tr>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">ID</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Name</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Host</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Port</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Status</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Created</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Actions</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-gray-700">
                        {filteredObsConnections.map((conn) => (
                          <tr key={conn.id} className="hover:bg-gray-700/30">
                            <td className="px-4 py-3 text-sm text-gray-300">{conn.id}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">{conn.name}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">{conn.host}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">{conn.port}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">
                              <StatusDot color={getStatusColor(conn.is_active ? 'connected' : 'disconnected')} />
                            </td>
                            <td className="px-4 py-3 text-sm text-gray-300">{conn.created_at}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">
                              <div className="flex gap-2">
                                <Button variant="secondary" size="sm">Edit</Button>
                                <Button variant="danger" size="sm">Delete</Button>
                              </div>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              </div>
            )
          },
          {
            id: 'flag-mappings',
            label: 'Flag Mappings',
            icon: TabIcons.flags,
            content: (
              <div className="space-y-4">
                <div className="flex justify-between items-center">
                  <h3 className="text-lg font-semibold text-gray-100">Flag Mappings</h3>
                  <div className="flex gap-2">
                    <Input
                      type="text"
                      placeholder="Search flags..."
                      value={searchTerm}
                      onChange={(e) => setSearchTerm(e.target.value)}
                      className="w-64"
                    />
                    <Button variant="primary" size="sm">
                      Import Flags
                    </Button>
                  </div>
                </div>
                
                <div className="bg-gradient-to-br from-gray-800/80 to-gray-900/90 backdrop-blur-sm rounded-lg border border-gray-600/30 overflow-hidden">
                  <div className="overflow-x-auto">
                    <table className="w-full">
                      <thead className="bg-gray-700/50">
                        <tr>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">ID</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Flag</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">PSS Code</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">IOC Code</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Country</th>
                          <th className="px-4 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">Actions</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-gray-700">
                        {filteredFlagMappings.map((flag) => (
                          <tr key={flag.id} className="hover:bg-gray-700/30">
                            <td className="px-4 py-3 text-sm text-gray-300">{flag.id}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">
                              <img 
                                src={flag.flag_url} 
                                alt={flag.country_name}
                                className="w-6 h-4 object-cover rounded"
                              />
                            </td>
                            <td className="px-4 py-3 text-sm text-gray-300">{flag.pss_code}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">{flag.ioc_code}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">{flag.country_name}</td>
                            <td className="px-4 py-3 text-sm text-gray-300">
                              <div className="flex gap-2">
                                <Button variant="secondary" size="sm">Edit</Button>
                                <Button variant="danger" size="sm">Delete</Button>
                              </div>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              </div>
            )
          }
        ]}
        activeTab={activeTab}
        onTabChange={setActiveTab}
      />
    </div>
  );
};

export default DatabaseManagementPanel; 
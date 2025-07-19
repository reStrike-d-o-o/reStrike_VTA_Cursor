import { create } from 'zustand';
import { devtools } from 'zustand/middleware';

// Types
export interface ObsConnection {
  name: string;
  host: string;
  port: number;
  password?: string;
  enabled: boolean;
  status: 'disconnected' | 'connecting' | 'connected' | 'authenticating' | 'authenticated' | 'error';
  error?: string;
}

export interface ObsEvent {
  eventType: string;
  connection_name: string;
  timestamp: string;
  scene_name?: string;
  is_recording?: boolean;
  is_streaming?: boolean;
  is_active?: boolean;
  error?: string;
  obs_event_type?: string;
  data?: any;
}

export interface ObsStatusInfo {
  is_recording: boolean;
  is_streaming: boolean;
  cpu_usage: number;
  recording_connection?: string;
  streaming_connection?: string;
}

interface ObsState {
  // Connections
  connections: ObsConnection[];
  activeConnection: string | null;
  
  // Status
  obsStatus: ObsStatusInfo | null;
  
  // Events
  events: ObsEvent[];
  maxEvents: number;
  
  // UI State
  loading: boolean;
  error: string | null;
}

interface ObsActions {
  // Connection Management
  setConnections: (connections: ObsConnection[]) => void;
  updateConnectionStatus: (name: string, status: ObsConnection['status'], error?: string) => void;
  setActiveConnection: (name: string | null) => void;
  
  // Status Management
  updateObsStatus: (status: ObsStatusInfo) => void;
  
  // Event Management
  addEvent: (event: ObsEvent) => void;
  clearEvents: () => void;
  setMaxEvents: (max: number) => void;
  
  // UI State
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
  
  // Utility Actions
  getConnectionByName: (name: string) => ObsConnection | undefined;
  getConnectedConnections: () => ObsConnection[];
  getConnectionCount: () => { total: number; connected: number; connecting: number; disconnected: number; error: number };
}

export type ObsStore = ObsState & ObsActions;

// Initial state
const initialState: ObsState = {
  connections: [],
  activeConnection: null,
  obsStatus: null,
  events: [],
  maxEvents: 50,
  loading: false,
  error: null,
};

// Create store
export const useObsStore = create<ObsStore>()(
  devtools(
    (set, get) => ({
      ...initialState,

      // Connection Management
      setConnections: (connections) => {
        set({ connections });
      },

      updateConnectionStatus: (name, status, error) => {
        set((state) => ({
          connections: state.connections.map(c =>
            c.name === name ? { ...c, status, error } : c
          ),
        }));
      },

      setActiveConnection: (name) => {
        set({ activeConnection: name });
      },

      // Status Management
      updateObsStatus: (status) => {
        set({ obsStatus: status });
      },

      // Event Management
      addEvent: (event) => {
        set((state) => ({
          events: [event, ...state.events.slice(0, state.maxEvents - 1)],
        }));
      },

      clearEvents: () => {
        set({ events: [] });
      },

      setMaxEvents: (max) => {
        set((state) => ({
          maxEvents: max,
          events: state.events.slice(0, max),
        }));
      },

      // UI State
      setLoading: (loading) => {
        set({ loading });
      },

      setError: (error) => {
        set({ error });
      },

      clearError: () => {
        set({ error: null });
      },

      // Utility Actions
      getConnectionByName: (name) => {
        return get().connections.find(c => c.name === name);
      },

      getConnectedConnections: () => {
        return get().connections.filter(c => 
          c.status === 'connected' || c.status === 'authenticated'
        );
      },

      getConnectionCount: () => {
        const connections = get().connections;
        return {
          total: connections.length,
          connected: connections.filter(c => c.status === 'connected' || c.status === 'authenticated').length,
          connecting: connections.filter(c => c.status === 'connecting' || c.status === 'authenticating').length,
          disconnected: connections.filter(c => c.status === 'disconnected').length,
          error: connections.filter(c => c.status === 'error').length,
        };
      },
    }),
    {
      name: 'obs-store',
    }
  )
); 
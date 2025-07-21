import { create } from 'zustand';
import { devtools } from 'zustand/middleware';

// Types
export type LiveDataType = 'pss' | 'obs' | 'udp';

export interface LiveDataEntry {
  timestamp: string;
  subsystem: LiveDataType;
  data: string;
  type: 'info' | 'warning' | 'error' | 'debug';
}

interface LiveDataState {
  // Streaming State
  enabled: boolean;
  selectedType: LiveDataType;
  
  // Data
  data: LiveDataEntry[];
  maxEntries: number;
  
  // UI State
  loading: boolean;
  error: string | null;
  connecting: boolean;
}

interface LiveDataActions {
  // Streaming Control
  setEnabled: (enabled: boolean) => void;
  setSelectedType: (type: LiveDataType) => void;
  
  // Data Management
  addData: (entry: Omit<LiveDataEntry, 'timestamp'>) => void;
  clearData: () => void;
  setMaxEntries: (max: number) => void;
  
  // UI State
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
  setConnecting: (connecting: boolean) => void;
  
  // Utility Actions
  getDataByType: (type: LiveDataType) => LiveDataEntry[];
  getLatestData: (count?: number) => LiveDataEntry[];
}

export type LiveDataStore = LiveDataState & LiveDataActions;

// Initial state
const initialState: LiveDataState = {
  enabled: false,
  selectedType: 'pss',
  data: [],
  maxEntries: 1000,
  loading: false,
  error: null,
  connecting: false,
};

// Create store
export const useLiveDataStore = create<LiveDataStore>()(
  devtools(
    (set, get) => ({
      ...initialState,

      // Streaming Control
      setEnabled: (enabled) => {
        set({ enabled });
      },

      setSelectedType: (type) => {
        set({ selectedType: type });
      },

      // Data Management
      addData: (entry) => {
        const newEntry: LiveDataEntry = {
          ...entry,
          timestamp: new Date().toISOString(),
        };
        
        set((state) => ({
          // Prepend new entries so newest appears first in UI
          data: [newEntry, ...state.data].slice(0, state.maxEntries),
        }));
      },

      clearData: () => {
        set({ data: [] });
      },

      setMaxEntries: (max) => {
        set((state) => ({
          maxEntries: max,
          data: state.data.slice(-max),
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

      setConnecting: (connecting) => {
        set({ connecting });
      },

      // Utility Actions
      getDataByType: (type) => {
        return get().data.filter(entry => entry.subsystem === type);
      },

      getLatestData: (count = 50) => {
        const data = get().data;
        return data.slice(-count);
      },
    }),
    {
      name: 'live-data-store',
    }
  )
); 
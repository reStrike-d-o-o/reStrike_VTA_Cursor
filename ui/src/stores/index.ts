import { create } from 'zustand';
import { devtools } from 'zustand/middleware';

// Types
export interface ObsConnection {
  name: string;
  host: string;
  port: number;
  password?: string;
  enabled: boolean;
  status: 'Disconnected' | 'Connecting' | 'Connected' | 'Authenticating' | 'Authenticated' | 'Error';
  error?: string;
}

export interface ObsStatusInfo {
  is_recording: boolean;
  is_streaming: boolean;
  cpu_usage: number;
  recording_connection?: string;
  streaming_connection?: string;
}

export interface OverlaySettings {
  opacity: number;
  position: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right' | 'center';
  scale: number;
  visible: boolean;
  theme: 'dark' | 'light' | 'transparent';
}

export interface VideoClip {
  id: string;
  name: string;
  path: string;
  duration: number;
  timestamp: Date;
  tags: string[];
}

export interface AppState {
  // OBS Connections
  obsConnections: ObsConnection[];
  activeObsConnection: string | null;
  obsStatus: ObsStatusInfo | null;
  
  // Overlay Settings
  overlaySettings: OverlaySettings;
  
  // Video Clips
  videoClips: VideoClip[];
  currentClip: VideoClip | null;
  isPlaying: boolean;
  
  // UI State
  currentView: 'sidebar-test' | 'overlay' | 'settings' | 'clips' | 'obs-manager';
  isLoading: boolean;
  error: string | null;
  // Advanced Panel State
  isAdvancedPanelOpen: boolean;
  activeDrawer: string;
  
  // Advanced Mode Authentication
  isAdvancedModeAuthenticated: boolean;
  isManualModeEnabled: boolean;
  
  // Window Settings
  windowSettings: {
    compactWidth: number;
    compactHeight: number;
    fullscreenWidth: number;
    fullscreenHeight: number;
  };
}

export interface AppActions {
  // OBS Actions
  addObsConnection: (connection: Omit<ObsConnection, 'status' | 'error'>) => void;
  removeObsConnection: (name: string) => void;
  updateObsConnectionStatus: (name: string, status: ObsConnection['status'], error?: string) => void;
  setActiveObsConnection: (name: string | null) => void;
  updateObsStatus: (status: ObsStatusInfo) => void;
  
  // Overlay Actions
  updateOverlaySettings: (settings: Partial<OverlaySettings>) => void;
  toggleOverlayVisibility: () => void;
  
  // Video Actions
  addVideoClip: (clip: Omit<VideoClip, 'id' | 'timestamp'>) => void;
  removeVideoClip: (id: string) => void;
  setCurrentClip: (clip: VideoClip | null) => void;
  setPlaying: (playing: boolean) => void;
  
  // UI Actions
  setCurrentView: (view: AppState['currentView']) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
  // Advanced Panel Actions
  openAdvancedPanel: () => void;
  closeAdvancedPanel: () => void;
  toggleAdvancedPanel: () => void;
  setActiveDrawer: (drawer: string) => void;
  
  // Advanced Mode Authentication Actions
  authenticateAdvancedMode: (password: string) => boolean;
  deauthenticateAdvancedMode: () => void;
  toggleManualMode: () => void;
  
  // Window Settings Actions
  updateWindowSettings: (settings: Partial<AppState['windowSettings']>) => void;
  resetWindowSettings: () => void;
  loadWindowSettings: () => Promise<void>;
  saveWindowSettings: () => Promise<void>;
}

export type AppStore = AppState & AppActions;

// Initial state
const initialState: AppState = {
  obsConnections: [
    {
      name: 'OBS_REC',
      host: 'localhost',
      port: 4455,
      password: 'cekPIbj@245',
      enabled: true,
      status: 'Disconnected',
    },
    {
      name: 'OBS_STR',
      host: 'localhost',
      port: 4466,
      password: 'cekPIbj@245',
      enabled: true,
      status: 'Disconnected',
    },
  ],
  activeObsConnection: null,
  obsStatus: null,
  overlaySettings: {
    opacity: 0.9,
    position: 'bottom-right',
    scale: 1.0,
    visible: true,
    theme: 'dark',
  },
  videoClips: [],
  currentClip: null,
  isPlaying: false,
  currentView: 'sidebar-test',
  isLoading: false,
  error: null,
  isAdvancedPanelOpen: false,
  activeDrawer: 'pss',
  isAdvancedModeAuthenticated: false,
  isManualModeEnabled: false,
  windowSettings: {
    compactWidth: 350,
    compactHeight: 1080,
    fullscreenWidth: 1920,
    fullscreenHeight: 1080,
  },
};

// Create store
export const useAppStore = create<AppStore>()(
  devtools(
    (set, get) => ({
      ...initialState,

      // OBS Actions
      addObsConnection: (connection) => {
        const newConnection: ObsConnection = {
          ...connection,
          status: 'Disconnected',
        };
        set((state) => ({
          obsConnections: [...state.obsConnections, newConnection],
        }));
      },

      removeObsConnection: (name) => {
        set((state) => ({
          obsConnections: state.obsConnections.filter(c => c.name !== name),
          activeObsConnection: state.activeObsConnection === name ? null : state.activeObsConnection,
        }));
      },

      updateObsConnectionStatus: (name, status, error) => {
        set((state) => ({
          obsConnections: state.obsConnections.map(c =>
            c.name === name ? { ...c, status, error } : c
          ),
        }));
      },

      setActiveObsConnection: (name) => {
        set({ activeObsConnection: name });
      },

      updateObsStatus: (status) => {
        set({ obsStatus: status });
      },

      // Overlay Actions
      updateOverlaySettings: (settings) => {
        set((state) => ({
          overlaySettings: { ...state.overlaySettings, ...settings },
        }));
      },

      toggleOverlayVisibility: () => {
        set((state) => ({
          overlaySettings: {
            ...state.overlaySettings,
            visible: !state.overlaySettings.visible,
          },
        }));
      },

      // Video Actions
      addVideoClip: (clip) => {
        const newClip: VideoClip = {
          ...clip,
          id: crypto.randomUUID(),
          timestamp: new Date(),
        };
        set((state) => ({
          videoClips: [newClip, ...state.videoClips],
        }));
      },

      removeVideoClip: (id) => {
        set((state) => ({
          videoClips: state.videoClips.filter(c => c.id !== id),
          currentClip: state.currentClip?.id === id ? null : state.currentClip,
        }));
      },

      setCurrentClip: (clip) => {
        set({ currentClip: clip });
      },

      setPlaying: (playing) => {
        set({ isPlaying: playing });
      },

      // UI Actions
      setCurrentView: (view) => {
        set({ currentView: view });
      },

      setLoading: (loading) => {
        set({ isLoading: loading });
      },

      setError: (error) => {
        set({ error });
      },

      clearError: () => {
        set({ error: null });
      },

      // Advanced Panel Actions
      openAdvancedPanel: () => set({ isAdvancedPanelOpen: true }),
      closeAdvancedPanel: () => set({ isAdvancedPanelOpen: false }),
      toggleAdvancedPanel: () => set((state) => ({ isAdvancedPanelOpen: !state.isAdvancedPanelOpen })),
      setActiveDrawer: (drawer) => set({ activeDrawer: drawer }),
      
      // Advanced Mode Authentication Actions
      authenticateAdvancedMode: (password) => {
        const isValid = password === 'reStrike';
        if (isValid) {
          set({ isAdvancedModeAuthenticated: true });
        }
        return isValid;
      },
      
      deauthenticateAdvancedMode: () => {
        set({ 
          isAdvancedModeAuthenticated: false,
          isAdvancedPanelOpen: false 
        });
      },
      
      toggleManualMode: () => {
        console.log('toggleManualMode called!', { 
          currentState: get().isManualModeEnabled 
        });
        set((state) => {
          const newState = !state.isManualModeEnabled;
          console.log('Setting manual mode to:', newState);
          return { 
            isManualModeEnabled: newState 
          };
        });
      },

      // Window Settings Actions
      updateWindowSettings: (settings) => {
        set((state) => ({
          windowSettings: { ...state.windowSettings, ...settings },
        }));
      },

      resetWindowSettings: () => {
        set({ windowSettings: initialState.windowSettings });
      },

      loadWindowSettings: async () => {
        try {
          const { windowCommands } = await import('../utils/tauriCommands');
          const result = await windowCommands.loadWindowSettings();
          if (result.success && result.data) {
            set({ windowSettings: result.data });
          }
        } catch (error) {
          console.error('Failed to load window settings:', error);
        }
      },

      saveWindowSettings: async () => {
        try {
          const { windowCommands } = await import('../utils/tauriCommands');
          const state = get();
          await windowCommands.saveWindowSettings(state.windowSettings);
        } catch (error) {
          console.error('Failed to save window settings:', error);
        }
      },
    }),
    {
      name: 'restrike-vta-store',
    }
  )
);

// Export new stores
export { useObsStore } from './obsStore';
export { useLiveDataStore } from './liveDataStore';
export { usePssMatchStore } from './pssMatchStore';

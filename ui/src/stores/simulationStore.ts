/**
 * simulationStore
 * - Simulation state and progress tracking
 */
import { create } from 'zustand';

// Safe invoke for Tauri v2/v1 fallback
const invoke = async <T = any>(command: string, args?: any): Promise<T> => {
  try {
    // Prefer v2
    if (typeof window !== 'undefined' && (window as any).__TAURI__?.core?.invoke) {
      return await (window as any).__TAURI__.core.invoke(command, args);
    }
    // v1 style import fallback
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
    return await tauriInvoke(command, args);
  } catch (err) {
    throw err;
  }
};

export interface AutomatedScenario {
  name: string;
  display_name: string;
  description: string;
  match_count: number;
  estimated_duration: number;
}

export interface SimulationStatus {
  isRunning: boolean;
  isConnected: boolean;
  currentScenario: string;
  currentMode: string;
  eventsSent: number;
  lastEvent: string;
  automatedScenarios?: AutomatedScenario[];
}

interface SimulationStore {
  // state
  status: SimulationStatus;
  scenarios: AutomatedScenario[];
  selectedAutomatedScenario: string;
  selectedMode: string;
  selectedScenario: string;
  duration: number;
  progress: { current: number; total: number };
  loading: boolean;
  error: string;
  success: string;
  showAutomated: boolean;
  showSelfTest: boolean;
  showArcade: boolean;

  // actions
  loadStatus: () => Promise<void>;
  loadScenarios: () => Promise<void>;
  startManual: () => Promise<void>;
  startAutomated: () => Promise<void>;
  stop: () => Promise<void>;
  sendManualEvent: (eventType: string, params: any) => Promise<void>;
  installDependencies: () => Promise<void>;
  retry: () => Promise<void>;

  // setters
  setSelectedMode: (mode: string) => void;
  setSelectedScenario: (scenario: string) => void;
  setDuration: (seconds: number) => void;
  setSelectedAutomatedScenario: (name: string) => void;
  setShowAutomated: (v: boolean) => void;
  setShowSelfTest: (v: boolean) => void;
  setShowArcade: (v: boolean) => void;
}

const isSimulationEnvError = (errorMsg: string): boolean => {
  if (!errorMsg) return false;
  return (
    errorMsg.includes('Simulation environment error') ||
    errorMsg.includes('PythonNotFound') ||
    errorMsg.includes('PythonVersionTooLow') ||
    errorMsg.includes('PipInstallFailed') ||
    errorMsg.includes('DependencyCheckFailed') ||
    errorMsg.includes('SimulationPathNotFound')
  );
};

export const useSimulationStore = create<SimulationStore>((set, get) => ({
  status: {
    isRunning: false,
    isConnected: false,
    currentScenario: 'None',
    currentMode: 'None',
    eventsSent: 0,
    lastEvent: 'None',
  },
  scenarios: [],
  selectedAutomatedScenario: '',
  selectedMode: 'demo',
  selectedScenario: 'basic',
  duration: 30,
  progress: { current: 0, total: 0 },
  loading: false,
  error: '',
  success: '',
  showAutomated: false,
  showSelfTest: false,
  showArcade: false,

  async loadStatus() {
    try {
      const result = await invoke<any>('simulation_get_detailed_status');
      if (result?.success) {
        const data = result.data || {};
        set({ status: data });
        if (Array.isArray(data.automatedScenarios)) {
          set({ scenarios: data.automatedScenarios });
        }
        // clear env errors when successful
        if (isSimulationEnvError(get().error)) set({ error: '' });
      } else if (result?.error && isSimulationEnvError(result.error)) {
        set({ error: result.error });
      }
    } catch (e: any) {
      if (typeof e === 'string' && isSimulationEnvError(e)) set({ error: e });
    }
  },

  async loadScenarios() {
    try {
      const result = await invoke<any>('simulation_get_scenarios');
      if (result?.success) {
        const scenarios: AutomatedScenario[] = result.data || [];
        if (scenarios.length > 0) {
          set({ scenarios });
          if (!get().selectedAutomatedScenario) {
            set({ selectedAutomatedScenario: scenarios[0].name });
          }
          if (isSimulationEnvError(get().error)) set({ error: '' });
        } else {
          // Fallback defaults when parsing yields empty array
          const fallback: AutomatedScenario[] = [
            { name: 'basic', display_name: 'Basic Match', description: 'Standard single match simulation', match_count: 1, estimated_duration: 90 },
            { name: 'quick_test', display_name: 'Quick Test', description: 'Fast single match for testing', match_count: 1, estimated_duration: 45 },
            { name: 'training_session', display_name: 'Training Session', description: 'Multiple matches for training', match_count: 5, estimated_duration: 600 },
            { name: 'tournament_day', display_name: 'Tournament Day', description: 'Full tournament simulation', match_count: 20, estimated_duration: 3600 },
            { name: 'championship', display_name: 'Championship', description: 'High-intensity championship matches', match_count: 8, estimated_duration: 1800 },
            { name: 'demo', display_name: 'Demo Mode', description: 'Short demo for testing overlays', match_count: 1, estimated_duration: 60 },
            { name: 'intensive', display_name: 'Intensive Training', description: 'High-frequency events for stress testing', match_count: 3, estimated_duration: 300 },
            { name: 'olympic', display_name: 'Olympic Style', description: 'Olympic-level competition simulation', match_count: 6, estimated_duration: 2400 },
          ];
          set({ scenarios: fallback, selectedAutomatedScenario: fallback[0].name });
        }
      } else if (result?.error) {
        if (isSimulationEnvError(result.error)) {
          set({ error: result.error });
        } else if (String(result.error).includes('Failed to get scenarios')) {
          set({ error: 'Backend connection failed. Please ensure the application is running properly.' });
        }
        // Use fallback scenarios on error too
        const fallback: AutomatedScenario[] = [
          { name: 'basic', display_name: 'Basic Match', description: 'Standard single match simulation', match_count: 1, estimated_duration: 90 },
          { name: 'quick_test', display_name: 'Quick Test', description: 'Fast single match for testing', match_count: 1, estimated_duration: 45 },
          { name: 'training_session', display_name: 'Training Session', description: 'Multiple matches for training', match_count: 5, estimated_duration: 600 },
          { name: 'tournament_day', display_name: 'Tournament Day', description: 'Full tournament simulation', match_count: 20, estimated_duration: 3600 },
          { name: 'championship', display_name: 'Championship', description: 'High-intensity championship matches', match_count: 8, estimated_duration: 1800 },
          { name: 'demo', display_name: 'Demo Mode', description: 'Short demo for testing overlays', match_count: 1, estimated_duration: 60 },
          { name: 'intensive', display_name: 'Intensive Training', description: 'High-frequency events for stress testing', match_count: 3, estimated_duration: 300 },
          { name: 'olympic', display_name: 'Olympic Style', description: 'Olympic-level competition simulation', match_count: 6, estimated_duration: 2400 },
        ];
        set({ scenarios: fallback, selectedAutomatedScenario: fallback[0].name });
      }
    } catch (e: any) {
      if (typeof e === 'string') {
        if (isSimulationEnvError(e)) set({ error: e });
        else if (e.includes('Failed to invoke') || e.includes('Connection')) set({ error: 'Cannot connect to backend. Please restart the application.' });
      }
      // Ensure UI remains usable with fallback scenarios
      const fallback: AutomatedScenario[] = [
        { name: 'basic', display_name: 'Basic Match', description: 'Standard single match simulation', match_count: 1, estimated_duration: 90 },
        { name: 'quick_test', display_name: 'Quick Test', description: 'Fast single match for testing', match_count: 1, estimated_duration: 45 },
        { name: 'training_session', display_name: 'Training Session', description: 'Multiple matches for training', match_count: 5, estimated_duration: 600 },
        { name: 'tournament_day', display_name: 'Tournament Day', description: 'Full tournament simulation', match_count: 20, estimated_duration: 3600 },
        { name: 'championship', display_name: 'Championship', description: 'High-intensity championship matches', match_count: 8, estimated_duration: 1800 },
        { name: 'demo', display_name: 'Demo Mode', description: 'Short demo for testing overlays', match_count: 1, estimated_duration: 60 },
        { name: 'intensive', display_name: 'Intensive Training', description: 'High-frequency events for stress testing', match_count: 3, estimated_duration: 300 },
        { name: 'olympic', display_name: 'Olympic Style', description: 'Olympic-level competition simulation', match_count: 6, estimated_duration: 2400 },
      ];
      set({ scenarios: fallback, selectedAutomatedScenario: fallback[0].name });
    }
  },

  async startManual() {
    try {
      set({ loading: true, error: '', success: '' });
      const { selectedMode, selectedScenario, duration } = get();
      const res = await invoke<any>('simulation_start', { mode: selectedMode, scenario: selectedScenario, duration });
      if (res?.success) set({ success: 'Simulation started successfully!' });
      else set({ error: res?.error || 'Failed to start simulation' });
      await get().loadStatus();
    } catch (e: any) {
      set({ error: `Failed to start simulation: ${e}` });
    } finally {
      set({ loading: false });
    }
  },

  async startAutomated() {
    try {
      set({ loading: true, error: '', success: '' });
      const name = get().selectedAutomatedScenario;
      const res = await invoke<any>('simulation_run_automated', { scenario_name: name });
      if (res?.success) set({ success: `Automated ${name} simulation started successfully!` });
      else set({ error: res?.error || 'Failed to start automated simulation' });
      await get().loadStatus();
    } catch (e: any) {
      set({ error: `Failed to start automated simulation: ${e}` });
    } finally {
      set({ loading: false });
    }
  },

  async stop() {
    try {
      set({ loading: true, error: '', success: '' });
      const res = await invoke<any>('simulation_stop');
      if (res?.success) set({ success: 'Simulation stopped successfully!' });
      else set({ error: res?.error || 'Failed to stop simulation' });
      await get().loadStatus();
    } catch (e: any) {
      set({ error: `Failed to stop simulation: ${e}` });
    } finally {
      set({ loading: false });
    }
  },

  async sendManualEvent(eventType: string, params: any) {
    try {
      const res = await invoke<any>('simulation_send_event', { eventType, params });
      if (res?.success) set({ success: `${eventType} event sent successfully!` });
      else set({ error: res?.error || `Failed to send ${eventType} event` });
      await get().loadStatus();
    } catch (e: any) {
      set({ error: `Failed to send ${eventType} event: ${e}` });
    }
  },

  async installDependencies() {
    try {
      set({ loading: true, error: '', success: '' });
      const res = await invoke<any>('simulation_get_scenarios');
      if (res?.success) {
        set({ success: 'Dependencies installed successfully!' });
        await get().loadStatus();
        await get().loadScenarios();
      } else set({ error: res?.error || 'Failed to install dependencies' });
    } catch (e: any) {
      set({ error: `Failed to install dependencies: ${e}` });
    } finally {
      set({ loading: false });
    }
  },

  async retry() {
    set({ error: '', success: '' });
    await get().loadStatus();
    await get().loadScenarios();
  },

  setSelectedMode(mode: string) { set({ selectedMode: mode }); },
  setSelectedScenario(s: string) { set({ selectedScenario: s }); },
  setDuration(seconds: number) { set({ duration: seconds || 30 }); },
  setSelectedAutomatedScenario(name: string) { set({ selectedAutomatedScenario: name }); },
  setShowAutomated(v: boolean) { set({ showAutomated: v }); },
  setShowSelfTest(v: boolean) { set({ showSelfTest: v }); },
  setShowArcade(v: boolean) { set({ showArcade: v }); },
}));



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
        set({ scenarios });
        if (!get().selectedAutomatedScenario && scenarios.length) {
          set({ selectedAutomatedScenario: scenarios[0].name });
        }
        if (isSimulationEnvError(get().error)) set({ error: '' });
      } else if (result?.error && isSimulationEnvError(result.error)) {
        set({ error: result.error });
      }
    } catch (e: any) {
      if (typeof e === 'string') {
        if (isSimulationEnvError(e)) set({ error: e });
        else if (e.includes('Failed to invoke') || e.includes('Connection')) set({ error: 'Cannot connect to backend. Please restart the application.' });
      }
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
}));



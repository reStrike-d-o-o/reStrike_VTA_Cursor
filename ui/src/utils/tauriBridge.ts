import '../types/tauri.d';

type TauriCore = {
  invoke: (command: string, args?: Record<string, unknown>) => Promise<any>;
};

type TauriEventApi = {
  listen: (event: string, handler: (payload: any) => void) => Promise<() => void>;
};

const getWindow = (): Window | null => (typeof window !== 'undefined' ? window : null);

export const getTauri = () => getWindow()?.__TAURI__ ?? null;

export const getTauriCore = (): TauriCore | null => {
  const core = getTauri()?.core;
  return core && typeof core.invoke === 'function' ? core : null;
};

export const getTauriEvent = (): TauriEventApi | null => {
  const eventApi = getTauri()?.event;
  return eventApi && typeof eventApi.listen === 'function' ? eventApi : null;
};

export const canInvokeTauri = () => !!getTauriCore();

export const canListenTauri = () => !!getTauriEvent();

export const invokeTauri = async <T = unknown>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> => {
  const core = getTauriCore();
  if (!core) {
    throw new Error('Tauri core.invoke is not available in this environment.');
  }
  return core.invoke(command, args) as Promise<T>;
};

export const listenTauri = async (
  event: string,
  handler: (payload: any) => void,
): Promise<() => void> => {
  const eventApi = getTauriEvent();
  if (!eventApi) {
    throw new Error('Tauri event.listen is not available in this environment.');
  }
  return eventApi.listen(event, handler);
};

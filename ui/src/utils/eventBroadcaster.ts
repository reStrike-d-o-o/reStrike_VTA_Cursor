/**
 * eventBroadcaster
 * - Broadcasts custom events to the app window for UI consumption
 */
export const broadcast = (type: string, detail: any) => window.dispatchEvent(new CustomEvent(type, { detail })); 
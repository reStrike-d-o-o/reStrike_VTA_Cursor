import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

// PSS Event types based on protocol specification
export interface PssEventData {
  id: string;
  eventType: 'pt1' | 'pt2' | 'wg1' | 'wg2' | 'ch0' | 'ch1' | 'ch2' | 'clk' | 'rnd' | 'hl1' | 'hl2';
  eventCode: string; // K, P, H, TH, TB, R
  athlete: 'blue' | 'red' | 'referee';
  round: number;
  time: string; // Clock time from PSS (e.g., "1:45")
  timestamp: string; // When we received the event
  rawData: string;
  description: string;
}

interface LiveDataState {
  events: PssEventData[];
  currentRound: number;
  currentTime: string;
  isConnected: boolean;
  lastUpdate: string;
  
  // Actions
  addEvent: (event: PssEventData) => void;
  clearEvents: () => void;
  storeEventsToDatabase: (matchId: string) => Promise<void>;
  setCurrentRound: (round: number) => void;
  setCurrentTime: (time: string) => void;
  setConnectionStatus: (connected: boolean) => void;
  updateLastUpdate: () => void;
  
  // Computed
  getFilteredEvents: (colorFilter?: string | null, eventTypeFilter?: string | null) => PssEventData[];
  getEventsByRound: (round: number) => PssEventData[];
}

export const useLiveDataStore = create<LiveDataState>()(
  subscribeWithSelector((set, get) => ({
    events: [],
    currentRound: 1,
    currentTime: '2:00',
    isConnected: false,
    lastUpdate: new Date().toISOString(),
    
    addEvent: (event: PssEventData) => {
      set((state) => ({
        events: [event, ...state.events], // Prepend new event to show newest at top
        lastUpdate: new Date().toISOString(),
      }));
    },
    
    clearEvents: () => {
      set({
        events: [],
        lastUpdate: new Date().toISOString(),
      });
    },
    
    storeEventsToDatabase: async (matchId: string) => {
      const { events } = get();
      if (events.length === 0) return;
      
      try {
        // Import invoke dynamically to avoid SSR issues
        const { invoke } = await import('@tauri-apps/api/core');
        
        // Store each event to the database
        for (const event of events) {
          await invoke('store_pss_event', {
            eventData: {
              match_id: matchId,
              event_type: event.eventType,
              event_code: event.eventCode,
              athlete: event.athlete,
              round: event.round,
              time: event.time,
              timestamp: event.timestamp,
              raw_data: event.rawData,
              description: event.description
            }
          });
        }
        
        console.log(`✅ Stored ${events.length} events to database for match ${matchId}`);
      } catch (error) {
        console.error('❌ Failed to store events to database:', error);
        throw error;
      }
    },
    
    setCurrentRound: (round: number) => {
      set({ currentRound: round });
    },
    
    setCurrentTime: (time: string) => {
      set({ currentTime: time });
    },
    
    setConnectionStatus: (connected: boolean) => {
      set({ isConnected: connected });
    },
    
    updateLastUpdate: () => {
      set({ lastUpdate: new Date().toISOString() });
    },
    
    getFilteredEvents: (colorFilter?: string | null, eventTypeFilter?: string | null) => {
      const { events } = get();
      return events.filter(event => {
        const colorMatch = colorFilter ? event.athlete === colorFilter : true;
        const eventTypeMatch = eventTypeFilter ? event.eventCode === eventTypeFilter : true;
        return colorMatch && eventTypeMatch;
      });
    },
    
    getEventsByRound: (round: number) => {
      const { events } = get();
      return events.filter(event => event.round === round);
    },
  }))
);

// Event parsing utilities based on PSS protocol
export const parsePssEvent = (rawData: string, timestamp: string): PssEventData | null => {
  // Handle undefined or null rawData
  if (!rawData || typeof rawData !== 'string') {
    console.warn('parsePssEvent: Invalid rawData:', rawData);
    return null;
  }
  
  const parts = rawData.split(';').filter(p => p.trim() !== '');
  if (parts.length === 0) return null;
  
  const eventType = parts[0];
  const id = `${eventType}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  
  // Parse different event types based on simplified backend format
  switch (eventType) {
    case 'pt1':
    case 'pt2': {
      // Points events: pt1 or pt2 (point type is in the event_code from backend)
      const athlete = eventType === 'pt1' ? 'blue' : 'red';
      
      return {
        id,
        eventType: eventType as any,
        eventCode: 'P', // Will be updated by backend event_code
        athlete,
        round: 1, // Will be updated by round events
        time: '0:00', // Will be updated by clock events
        timestamp,
        rawData,
        description: `${athlete} point`,
      };
    }
    
    case 'wg1':
    case 'wg2': {
      // Warning events: wg1;1;wg2;2;
      const athlete = eventType === 'wg1' ? 'blue' : 'red';
      return {
        id,
        eventType: eventType as any,
        eventCode: 'R',
        athlete,
        round: 1,
        time: '0:00',
        timestamp,
        rawData,
        description: `${athlete} warning`,
      };
    }
    
    case 'ch0':
    case 'ch1':
    case 'ch2': {
      // Challenge events
      const athlete = eventType === 'ch0' ? 'referee' : 
                     eventType === 'ch1' ? 'blue' : 'red';
      return {
        id,
        eventType: eventType as any,
        eventCode: 'R',
        athlete,
        round: 1,
        time: '0:00',
        timestamp,
        rawData,
        description: `${athlete} challenge`,
      };
    }
    
    case 'clk': {
      // Clock events: clk;1:45;
      if (parts.length < 2) return null;
      const time = parts[1];
      return {
        id,
        eventType: eventType as any,
        eventCode: 'T', // Time
        athlete: 'referee',
        round: 1,
        time,
        timestamp,
        rawData,
        description: `Clock: ${time}`,
      };
    }
    
    case 'rnd': {
      // Round events: rnd;2;
      if (parts.length < 2) return null;
      const round = parseInt(parts[1]);
      return {
        id,
        eventType: eventType as any,
        eventCode: 'R', // Round
        athlete: 'referee',
        round,
        time: '0:00',
        timestamp,
        rawData,
        description: `Round ${round}`,
      };
    }
    
    case 'hl1':
    case 'hl2': {
      // Hit level events: hl1;50;
      const athlete = eventType === 'hl1' ? 'blue' : 'red';
      return {
        id,
        eventType: eventType as any,
        eventCode: 'H', // Hit
        athlete,
        round: 1,
        time: '0:00',
        timestamp,
        rawData,
        description: `${athlete} hit`,
      };
    }
    
    default:
      return null;
  }
};

// WebSocket connection for real-time updates
export class LiveDataWebSocket {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  
  constructor(private url: string, private onMessage: (data: any) => void) {}
  
  connect() {
    try {
      this.ws = new WebSocket(this.url);
      
      this.ws.onopen = () => {
        console.log('🔗 Live data WebSocket connected');
        useLiveDataStore.getState().setConnectionStatus(true);
        this.reconnectAttempts = 0;
      };
      
      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          this.onMessage(data);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };
      
      this.ws.onclose = () => {
        console.log('🔌 Live data WebSocket disconnected');
        useLiveDataStore.getState().setConnectionStatus(false);
        this.attemptReconnect();
      };
      
      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };
      
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
    }
  }
  
  private attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      console.log(`🔄 Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
      
      setTimeout(() => {
        this.connect();
      }, this.reconnectDelay * this.reconnectAttempts);
    } else {
      console.error('❌ Max reconnection attempts reached');
    }
  }
  
  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
  
  send(data: any) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data));
    }
  }
} 
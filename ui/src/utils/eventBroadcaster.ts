/**
 * Event Broadcaster for Scoreboard Overlays
 * Uses localStorage events to communicate between main app and HTML overlays
 */

class EventBroadcaster {
  private static instance: EventBroadcaster;
  private eventId: number = 0;

  private constructor() {}

  static getInstance(): EventBroadcaster {
    if (!EventBroadcaster.instance) {
      EventBroadcaster.instance = new EventBroadcaster();
    }
    return EventBroadcaster.instance;
  }

  /**
   * Broadcast PSS event to all overlays
   */
  broadcastPssEvent(event: any): void {
    try {
      const eventData = {
        id: ++this.eventId,
        type: 'pss_event',
        data: event,
        timestamp: Date.now()
      };

      // Store event in localStorage to trigger storage event
      localStorage.setItem('pss_event', JSON.stringify(eventData));
      
      // Clear after a short delay to allow event processing
      setTimeout(() => {
        localStorage.removeItem('pss_event');
      }, 100);

      console.log(`📡 Broadcasted PSS event:`, event.type);
    } catch (error) {
      console.error('❌ Error broadcasting PSS event:', error);
    }
  }

  /**
   * Listen for PSS events (for overlays)
   */
  listenForPssEvents(callback: (event: any) => void): () => void {
    const handleStorageEvent = (e: StorageEvent) => {
      if (e.key === 'pss_event' && e.newValue) {
        try {
          const eventData = JSON.parse(e.newValue);
          if (eventData.type === 'pss_event') {
            console.log('📡 Received PSS event via broadcast:', eventData.data.type);
            callback(eventData.data);
          }
        } catch (error) {
          console.error('❌ Error parsing broadcasted event:', error);
        }
      }
    };

    // Listen for storage events
    window.addEventListener('storage', handleStorageEvent);

    // Return cleanup function
    return () => {
      window.removeEventListener('storage', handleStorageEvent);
    };
  }

  /**
   * Send test event for debugging
   */
  sendTestEvent(): void {
    const testEvent = {
      type: 'test',
      message: 'Test event from main application',
      timestamp: Date.now()
    };
    this.broadcastPssEvent(testEvent);
  }
}

// Create singleton instance
const eventBroadcaster = EventBroadcaster.getInstance();

export default eventBroadcaster; 
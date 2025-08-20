import React, { useState, useEffect } from 'react';
import Button from '../atoms/Button';
import { StatusDot } from '../atoms/StatusDot';
import { usePssMatchStore } from '../../stores/pssMatchStore';
import { useI18n } from '../../i18n/index';

interface EventTableEvent {
  id: string;
  round: string;
  time: string;
  event: string;
  color: string;
  timestamp: number;
}

const EventTable: React.FC = () => {
  console.log('🔍 EventTable - Component rendering');
  const { t } = useI18n();
  
  const [events, setEvents] = useState<EventTableEvent[]>([]);
  const { matchData } = usePssMatchStore();
  
  console.log('🔍 EventTable - Store data:', matchData);
  console.log('🔍 EventTable - Current events state:', events);

  // Debug: Log current store state
  useEffect(() => {
    console.log('🔍 EventTable - Store state useEffect triggered');
    console.log('🔍 EventTable - Current store state:', {
      currentRound: matchData.currentRound,
      currentRoundTime: matchData.currentRoundTime,
      fullMatchData: matchData
    });
  }, [matchData.currentRound, matchData.currentRoundTime]);

  // Listen for PSS events and add them to the table
  useEffect(() => {
    console.log('🔍 EventTable - Main useEffect triggered');
    
    const handlePssEvent = (event: CustomEvent) => {
      console.log('🔍 EventTable - handlePssEvent called');
      const pssEvent = event.detail;
      
      console.log('🔍 EventTable - Received PSS event:', {
        type: pssEvent.type,
        event: pssEvent
      });
      
      // Create event table entry based on PSS event type
      let eventTableEvent: EventTableEvent | null = null;
      
      // Get current round and time from store
      const currentRound = matchData.currentRound || 1;
      const currentTime = matchData.currentRoundTime || '0:00';
      
      console.log('🔍 EventTable - Creating event with:', {
        eventType: pssEvent.type,
        currentRound,
        currentTime,
        storeData: {
          currentRound: matchData.currentRound,
          currentRoundTime: matchData.currentRoundTime
        }
      });
      
      switch (pssEvent.type) {
        case 'points':
          console.log('🔍 EventTable - Processing points event');
          eventTableEvent = {
            id: `point-${Date.now()}`,
            round: `R${currentRound}`,
            time: currentTime,
            event: pssEvent.description || 'Point',
            color: pssEvent.athlete === 'athlete1' ? 'red' : 'blue',
            timestamp: Date.now(),
          };
          break;
        case 'warnings':
          console.log('🔍 EventTable - Processing warnings event');
          eventTableEvent = {
            id: `warning-${Date.now()}`,
            round: `R${currentRound}`,
            time: currentTime,
            event: pssEvent.description || 'Warning',
            color: 'yellow',
            timestamp: Date.now(),
          };
          break;
        case 'round':
          console.log('🔍 EventTable - Processing round event');
          eventTableEvent = {
            id: `round-${Date.now()}`,
            round: `R${pssEvent.round || currentRound}`,
            time: currentTime,
            event: `Round ${pssEvent.round || currentRound}`,
            color: 'green',
            timestamp: Date.now(),
          };
          break;
        case 'clock':
          console.log('🔍 EventTable - Processing clock event');
          eventTableEvent = {
            id: `clock-${Date.now()}`,
            round: `R${currentRound}`,
            time: pssEvent.time || currentTime,
            event: `Clock: ${pssEvent.time || currentTime}`,
            color: 'purple',
            timestamp: Date.now(),
          };
          break;
        case 'current_scores':
          console.log('🔍 EventTable - Processing current_scores event');
          eventTableEvent = {
            id: `scores-${Date.now()}`,
            round: `R${currentRound}`,
            time: currentTime,
            event: `Scores: ${pssEvent.athlete1_score || 0}-${pssEvent.athlete2_score || 0}`,
            color: 'orange',
            timestamp: Date.now(),
          };
          break;
        default:
          console.log('🔍 EventTable - Unknown event type:', pssEvent.type);
      }
      
      if (eventTableEvent) {
        console.log('🔍 EventTable - Adding event to table:', eventTableEvent);
        setEvents(prev => {
          console.log('🔍 EventTable - Previous events:', prev);
          const newEvents = [...prev, eventTableEvent!];
          console.log('🔍 EventTable - New events array:', newEvents);
          // Keep only last 20 events
          return newEvents.slice(-20);
        });
      } else {
        console.log('🔍 EventTable - No event created for type:', pssEvent.type);
      }
    };

    console.log('🔍 EventTable - Adding event listener');
    // Add event listener
    window.addEventListener('pss-event', handlePssEvent as EventListener);
    
    // Test: Add a test event after 2 seconds to see if the table works
    console.log('🔍 EventTable - Setting up test timer');
    const testTimer = setTimeout(() => {
      console.log('🔍 EventTable - Adding test event');
      const testEvent: EventTableEvent = {
        id: 'test-event',
        round: `R${matchData.currentRound || 1}`,
        time: matchData.currentRoundTime || '0:00',
        event: 'Test Event',
        color: 'blue',
        timestamp: Date.now(),
      };
      console.log('🔍 EventTable - Test event created:', testEvent);
      setEvents(prev => {
        console.log('🔍 EventTable - Adding test event to state');
        return [...prev, testEvent];
      });
    }, 2000);
    
    return () => {
      console.log('🔍 EventTable - Cleanup function called');
      window.removeEventListener('pss-event', handlePssEvent as EventListener);
      clearTimeout(testTimer);
    };
  }, [matchData.currentRound, matchData.currentRoundTime]);

  const scrollToTop = () => {
    console.log('🔍 EventTable - Scroll to top clicked');
  };

  console.log('🔍 EventTable - About to render, events count:', events.length);
  
  return (
    <div className="mb-4 relative">
      {/* Header */}
      <div className="grid grid-cols-12 gap-2 text-xs text-gray-400 mb-2 border-b border-gray-700 pb-1">
        <div className="col-span-2">{t('table.rnd', 'RND')}</div>
        <div className="col-span-4">{t('common.time', 'TIME')}</div>
        <div className="col-span-6">{t('common.event', 'EVENT')}</div>
      </div>
      {/* Event Rows */}
      <div className="space-y-1 max-h-32 overflow-y-auto">
        {events.length > 0 ? (
          events.map((event) => {
            console.log('🔍 EventTable - Rendering event:', event);
            return (
              <div key={event.id} className="grid grid-cols-12 gap-2 text-xs">
                <div className="col-span-2 text-gray-300">{event.round}</div>
                <div className="col-span-4 text-gray-300">{event.time}</div>
                <div className="col-span-6 flex items-center space-x-1">
                  <StatusDot color={event.color} />
                  <span className="text-gray-300">{event.event}</span>
                </div>
              </div>
            );
          })
        ) : (
          <div className="text-center text-gray-500 text-xs py-4">{t('table.no_events', 'No events yet')}</div>
        )}
      </div>
      {/* Go to Top Arrow */}
      <div className="absolute bottom-0 right-0">
        <Button variant="secondary" size="sm" onClick={scrollToTop}>↑</Button>
      </div>
    </div>
  );
};

export default EventTable; 
import React from 'react';

/**
 * Tab Icons for different sections
 */
export const TabIcons = {
  // OBS Icons
  websocket: (
    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0" />
    </svg>
  ),
  
  integration: (
    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
    </svg>
  ),
  
  // OVR Icons
  ovrIntegration: (
    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
    </svg>
  ),
  
  // Generic Integration icon (two arrows in opposite directions)
  integrationArrows: (
    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
    </svg>
  ),
  
  // Punch/Kick icon for PSS drawer
  punchKick: (
    <svg width="24" height="24" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      {/* Fist - simple clenched fist */}
      <rect x="6" y="8" width="8" height="8" rx="4" stroke="currentColor" strokeWidth="2" fill="none"/>
      <path d="M10 8v8" stroke="currentColor" strokeWidth="2"/>
      <path d="M12 8v8" stroke="currentColor" strokeWidth="2"/>
      <path d="M14 8v8" stroke="currentColor" strokeWidth="2"/>
      
      {/* Kick - leg/foot */}
      <path d="M16 8l2-2" stroke="currentColor" strokeWidth="2"/>
      <path d="M18 6l2-2" stroke="currentColor" strokeWidth="2"/>
      <path d="M20 4l2-2" stroke="currentColor" strokeWidth="2"/>
      <path d="M16 8v4" stroke="currentColor" strokeWidth="2"/>
      <path d="M18 6v4" stroke="currentColor" strokeWidth="2"/>
      <path d="M20 4v4" stroke="currentColor" strokeWidth="2"/>
      
      {/* Dynamic lines representing movement */}
      <path d="M14 10l2-2" stroke="currentColor" strokeWidth="1" opacity="0.6"/>
      <path d="M12 8l2-2" stroke="currentColor" strokeWidth="1" opacity="0.6"/>
    </svg>
  ),
  
  tournament: (
    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      {/* Medal icon */}
      <circle cx="12" cy="12" r="8" stroke="currentColor" strokeWidth="2" fill="none"/>
      <path d="M12 4v8" stroke="currentColor" strokeWidth="2"/>
      <path d="M8 8l4 4 4-4" stroke="currentColor" strokeWidth="2"/>
      <path d="M12 20v-4" stroke="currentColor" strokeWidth="2"/>
      <path d="M8 16l4 4 4-4" stroke="currentColor" strokeWidth="2"/>
    </svg>
  ),
  
  // PSS Icons
  udp: (
    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <rect x="6" y="3" width="12" height="6" rx="2" stroke="currentColor" strokeWidth="2"/>
      <path d="M6 9l6 12 6-12" stroke="currentColor" strokeWidth="2"/>
      <path d="M9 9v3l3 3 3-3V9" stroke="currentColor" strokeWidth="2"/>
    </svg>
  ),
  
  flags: (
    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  ),
  
  scoreboard: (
    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      {/* Olympic-inspired scoreboard design */}
      {/* Main scoreboard frame */}
      <rect x="3" y="4" width="18" height="16" rx="2" stroke="currentColor" strokeWidth="1.5" fill="none"/>
      
      {/* Header bar (like the dark blue bar in Olympic graphics) */}
      <rect x="3" y="4" width="18" height="4" fill="currentColor" opacity="0.3"/>
      
      {/* Olympic rings inspired element - simplified */}
      <circle cx="6" cy="6" r="1" stroke="currentColor" strokeWidth="1" fill="none"/>
      <circle cx="8" cy="6" r="1" stroke="currentColor" strokeWidth="1" fill="none"/>
      <circle cx="10" cy="6" r="1" stroke="currentColor" strokeWidth="1" fill="none"/>
      
      {/* Score display lines */}
      <line x1="5" y1="10" x2="19" y2="10" stroke="currentColor" strokeWidth="1"/>
      <line x1="5" y="12" x2="19" y2="12" stroke="currentColor" strokeWidth="1"/>
      <line x1="5" y1="14" x2="19" y2="14" stroke="currentColor" strokeWidth="1"/>
      
      {/* Score indicators (dots instead of text) */}
      <circle cx="7" cy="11" r="0.5" fill="currentColor"/>
      <circle cx="9" cy="11" r="0.5" fill="currentColor"/>
      <circle cx="7" cy="13" r="0.5" fill="currentColor"/>
      <circle cx="9" cy="13" r="0.5" fill="currentColor"/>
      <circle cx="7" cy="15" r="0.5" fill="currentColor"/>
      <circle cx="9" cy="15" r="0.5" fill="currentColor"/>
      
      {/* Result indicators (small squares for wins) */}
      <rect x="17" y="10.5" width="1" height="1" fill="currentColor"/>
      <rect x="17" y="12.5" width="1" height="1" fill="currentColor"/>
      <rect x="17" y="14.5" width="1" height="1" fill="currentColor"/>
    </svg>
  ),
  
  // Database Icons
  overview: (
    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
    </svg>
  ),
  
  events: (
    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
    </svg>
  ),
  
  database: (
    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
    </svg>
  ),
  
  cloud: (
    <svg width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
    </svg>
  ),
};

export default TabIcons; 
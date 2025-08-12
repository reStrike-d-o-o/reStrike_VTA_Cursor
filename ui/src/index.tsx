import React from 'react';
import ReactDOM from 'react-dom/client';
import './index.css';
import App from './App';
import { initLogLevelFromStorage, applyConsolePatch } from './utils/logger';

// Initialize logging level and patch console globally before rendering
initLogLevelFromStorage();
applyConsolePatch();

// Performance optimizations for development
if (process.env.NODE_ENV === 'development') {
  // Disable React.StrictMode in development for faster renders
  const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement);
  root.render(<App />);
} else {
  const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement);
  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
}
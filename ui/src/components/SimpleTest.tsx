import React from 'react';

const SimpleTest: React.FC = () => {
  return (
    <div style={{ 
      padding: '20px', 
      backgroundColor: '#1a1a1a', 
      color: 'white', 
      fontFamily: 'Arial, sans-serif',
      minHeight: '100vh',
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center'
    }}>
      <h1 style={{ fontSize: '2rem', marginBottom: '1rem' }}>✅ React App is Working!</h1>
      <p style={{ fontSize: '1.2rem', marginBottom: '1rem' }}>If you can see this, React is rendering correctly.</p>
      <div style={{ 
        backgroundColor: '#333', 
        padding: '1rem', 
        borderRadius: '8px',
        marginTop: '1rem'
      }}>
        <p>Current time: {new Date().toLocaleTimeString()}</p>
        <p>Environment: {typeof window !== 'undefined' ? 'Browser' : 'Server'}</p>
        <p>React version: {React.version}</p>
      </div>
    </div>
  );
};

export default SimpleTest; 
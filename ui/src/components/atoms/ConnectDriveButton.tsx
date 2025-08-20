/**
 * ConnectDriveButton atom
 * - Starts Google Drive OAuth flow using Tauri commands (with web fallback)
 */
import React from 'react';

const ConnectDriveButton: React.FC = () => {
  const invoke = (cmd: string, args?: any) =>
    (window as any).__TAURI__.core.invoke(cmd, args);

  const openExternal = (url: string) => {
    // Try Tauri shell first, fallback to window.open
    if ((window as any).__TAURI__?.shell?.open) {
      return (window as any).__TAURI__.shell.open(url);
    } else {
      // Fallback to window.open
      window.open(url, '_blank', 'noopener,noreferrer');
      return Promise.resolve();
    }
  };

  const saveCreds = async () => {
    const id = prompt('Google OAuth Client ID:');
    if (!id) return false;
    
    const secret = prompt('Google OAuth Client Secret:');
    if (!secret) return false;
    
    try {
      await invoke('drive_save_credentials', { id: id.trim(), secret: secret.trim() });
      return true;
    } catch (error) {
      alert('Failed to save credentials: ' + error);
      return false;
    }
  };

  const connect = async () => {
    try {
      const url: string = await invoke('drive_request_auth_url');
      
      // Show instructions first
      const instructions = `Google OAuth Instructions:
      
1. Click OK to open the Google OAuth page
2. Sign in with your Google account
3. Click "Continue" to grant permissions
4. You'll see a page with an authorization code
5. Copy the authorization code (it will be displayed on the page)
6. Paste it in the next prompt

Note: This uses the "out-of-band" OAuth flow for desktop applications.`;
      
      if (!confirm(instructions)) return;
      
      // Open the OAuth URL
      await openExternal(url);
      
      // Wait a moment then prompt for the code
      setTimeout(() => {
        const code = prompt('Paste the authorization code from the Google OAuth page:');
        if (code) {
          invoke('drive_complete_auth', { code })
            .then(() => {
              alert('Google Drive connected successfully!');
            })
            .catch((error: any) => {
              alert('Failed to complete authentication: ' + error);
            });
        }
      }, 2000);
      
    } catch (err: any) {
      const errorMsg = String(err);
      if (errorMsg.includes('credentials not found')) {
        const saved = await saveCreds();
        if (saved) {
          // Retry the connection
          setTimeout(() => connect(), 100);
        }
      } else {
        alert('Drive connect error: ' + errorMsg);
      }
    }
  };

  return (
    <button
      className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded"
      onClick={connect}
    >
      Connect Google Drive
    </button>
  );
};

export default ConnectDriveButton; 
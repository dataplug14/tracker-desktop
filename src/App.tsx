import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LoginView } from './views/LoginView';
import { DashboardView } from './views/DashboardView';
import { useAppStore } from './stores/appStore';

function App() {
  const { isAuthenticated, setAuth, setUser } = useAppStore();
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Check for existing auth on startup
    const checkAuth = async () => {
      try {
        const session = await invoke<{
          access_token: string;
          user_id: string;
          display_name: string;
        } | null>('get_stored_session');

        if (session) {
          setAuth(true, session.access_token);
          setUser({
            id: session.user_id,
            displayName: session.display_name,
          });
        }
      } catch (error) {
        console.error('Failed to check auth:', error);
      } finally {
        setIsLoading(false);
      }
    };

    checkAuth();
  }, [setAuth, setUser]);

  if (isLoading) {
    return (
      <div className="app-container loading">
        <div className="loader">
          <div className="loader-spinner" />
          <p>Loading VTC Tracker...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="app-container">
      {isAuthenticated ? <DashboardView /> : <LoginView />}
    </div>
  );
}

export default App;

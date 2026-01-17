import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { 
  Truck, 
  Route, 
  DollarSign, 
  Package,
  Gauge,
  MapPin,
  ArrowRight,
  Settings,
  LogOut,
  Minus,
  X,
} from 'lucide-react';
import { useAppStore } from '../stores/appStore';

export function DashboardView() {
  const { 
    user, 
    telemetry, 
    todayJobs, 
    todayDistance, 
    todayRevenue,
    isConnected,
    setTelemetry,
    setConnected,
    logout,
  } = useAppStore();

  useEffect(() => {
    // Start telemetry listener
    const startTelemetry = async () => {
      try {
        await invoke('start_telemetry');
      } catch (error) {
        console.error('Failed to start telemetry:', error);
      }
    };

    // Start heartbeat
    const startHeartbeat = async () => {
      try {
        const result = await invoke<{ success: boolean }>('send_heartbeat');
        setConnected(result.success);
      } catch (error) {
        console.error('Heartbeat failed:', error);
        setConnected(false);
      }
    };

    startTelemetry();
    startHeartbeat();

    // Heartbeat interval (30 seconds)
    const heartbeatInterval = setInterval(startHeartbeat, 30000);

    return () => {
      clearInterval(heartbeatInterval);
    };
  }, [setConnected]);

  // Listen for telemetry updates
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      unlisten = await listen('telemetry_update', (event: any) => {
        setTelemetry(event.payload);
      });
    };

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, [setTelemetry]);

  const handleLogout = async () => {
    try {
      await invoke('logout');
      logout();
    } catch (error) {
      console.error('Logout error:', error);
      logout();
    }
  };

  const handleMinimize = () => invoke('minimize_window');
  const handleClose = () => invoke('hide_to_tray');

  const formatDistance = (km: number) => {
    if (km >= 1000) return `${(km / 1000).toFixed(1)}k km`;
    return `${km} km`;
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'EUR',
      maximumFractionDigits: 0,
    }).format(amount);
  };

  return (
    <div className="dashboard">
      {/* Titlebar */}
      <div className="titlebar">
        <div className="titlebar-title">
          <div className="status-dot" style={{ 
            marginRight: 8,
            background: isConnected ? 'var(--success)' : 'var(--foreground-dim)',
            boxShadow: isConnected ? '0 0 8px var(--success)' : 'none',
          }} />
          VTC Tracker
        </div>
        <div className="titlebar-controls">
          <button className="titlebar-btn" onClick={handleMinimize}>
            <Minus size={14} />
          </button>
          <button className="titlebar-btn close" onClick={handleClose}>
            <X size={14} />
          </button>
        </div>
      </div>

      {/* User header */}
      <div className="dashboard-header">
        <div className="user-info">
          <div className="user-avatar">
            {user?.displayName?.charAt(0) || 'D'}
          </div>
          <div>
            <div className="user-name">{user?.displayName || 'Driver'}</div>
            <div className="user-status">
              {telemetry.connected ? (
                <span className="status-online">
                  <span className="status-dot online" />
                  {telemetry.game?.toUpperCase()} Connected
                </span>
              ) : (
                <span className="status-offline">Game not detected</span>
              )}
            </div>
          </div>
        </div>
        <div className="header-actions">
          <button className="icon-btn" title="Settings">
            <Settings size={18} />
          </button>
          <button className="icon-btn" onClick={handleLogout} title="Logout">
            <LogOut size={18} />
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="dashboard-content">
        {/* Stats */}
        <div className="stat-grid">
          <div className="stat-card">
            <div className="stat-value">{todayJobs}</div>
            <div className="stat-label">
              <Package size={12} style={{ marginRight: 4 }} />
              Jobs Today
            </div>
          </div>
          <div className="stat-card">
            <div className="stat-value">{formatDistance(todayDistance)}</div>
            <div className="stat-label">
              <Route size={12} style={{ marginRight: 4 }} />
              Distance
            </div>
          </div>
          <div className="stat-card" style={{ gridColumn: 'span 2' }}>
            <div className="stat-value">{formatCurrency(todayRevenue)}</div>
            <div className="stat-label">
              <DollarSign size={12} style={{ marginRight: 4 }} />
              Revenue Today
            </div>
          </div>
        </div>

        {/* Telemetry */}
        {telemetry.connected && (
          <div className="telemetry-panel">
            <div className="telemetry-status">
              <span className={`telemetry-game ${telemetry.game}`}>
                {telemetry.game}
              </span>
              <span style={{ color: 'var(--foreground-muted)', fontSize: 12 }}>
                Telemetry Active
              </span>
            </div>

            <div className="telemetry-data">
              <div className="telemetry-row">
                <span className="telemetry-label">
                  <Gauge size={14} style={{ marginRight: 6 }} />
                  Speed
                </span>
                <span className="telemetry-value">{telemetry.speed} km/h</span>
              </div>
              {telemetry.currentCity && (
                <div className="telemetry-row">
                  <span className="telemetry-label">
                    <MapPin size={14} style={{ marginRight: 6 }} />
                    Location
                  </span>
                  <span className="telemetry-value">{telemetry.currentCity}</span>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Active Job */}
        {telemetry.activeJob && (
          <div className="job-info">
            <div className="job-header">
              <Truck size={16} />
              <span>Active Delivery</span>
            </div>
            <div className="job-route">
              <span className="job-city">{telemetry.activeJob.sourceCity}</span>
              <ArrowRight size={16} className="job-arrow" />
              <span className="job-city">{telemetry.activeJob.destinationCity}</span>
            </div>
            <div className="job-details">
              <div className="job-detail">
                <span className="telemetry-label">Cargo</span>
                <span className="telemetry-value">{telemetry.activeJob.cargo}</span>
              </div>
              <div className="job-detail">
                <span className="telemetry-label">Distance</span>
                <span className="telemetry-value">
                  {formatDistance(telemetry.activeJob.distanceRemaining)} left
                </span>
              </div>
              <div className="job-detail">
                <span className="telemetry-label">Revenue</span>
                <span className="telemetry-value" style={{ color: 'var(--success)' }}>
                  {formatCurrency(telemetry.activeJob.revenue)}
                </span>
              </div>
            </div>
          </div>
        )}

        {/* No game detected */}
        {!telemetry.connected && (
          <div className="no-game">
            <Truck size={48} strokeWidth={1} />
            <h3>Waiting for Game</h3>
            <p>Start ETS2 or ATS to begin tracking your jobs automatically.</p>
          </div>
        )}
      </div>

      <style>{`
        .dashboard {
          display: flex;
          flex-direction: column;
          height: 100vh;
        }

        .dashboard-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 16px;
          background: var(--bg-secondary);
          border-bottom: 1px solid var(--border);
        }

        .user-info {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .user-avatar {
          width: 40px;
          height: 40px;
          background: linear-gradient(135deg, var(--ets2), var(--ats));
          border-radius: 10px;
          display: flex;
          align-items: center;
          justify-content: center;
          font-weight: 700;
          font-size: 18px;
        }

        .user-name {
          font-weight: 600;
          font-size: 14px;
        }

        .user-status {
          font-size: 12px;
          color: var(--foreground-muted);
        }

        .status-online {
          display: flex;
          align-items: center;
          gap: 6px;
          color: var(--success);
        }

        .status-offline {
          color: var(--foreground-dim);
        }

        .header-actions {
          display: flex;
          gap: 8px;
        }

        .icon-btn {
          width: 36px;
          height: 36px;
          border: none;
          background: var(--bg-tertiary);
          border-radius: 8px;
          color: var(--foreground-muted);
          cursor: pointer;
          display: flex;
          align-items: center;
          justify-content: center;
          transition: all 0.15s;
        }

        .icon-btn:hover {
          background: var(--border);
          color: var(--foreground);
        }

        .dashboard-content {
          flex: 1;
          padding: 16px;
          overflow-y: auto;
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        .job-header {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 12px;
          font-weight: 600;
          color: var(--ets2);
        }

        .no-game {
          flex: 1;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          text-align: center;
          color: var(--foreground-dim);
          gap: 12px;
        }

        .no-game h3 {
          color: var(--foreground-muted);
          font-size: 16px;
        }

        .no-game p {
          font-size: 13px;
          max-width: 240px;
        }
      `}</style>
    </div>
  );
}

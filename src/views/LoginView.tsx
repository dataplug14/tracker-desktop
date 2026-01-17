import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Truck, Monitor, ArrowRight, AlertCircle } from 'lucide-react';
import { useAppStore } from '../stores/appStore';

export function LoginView() {
  const [code, setCode] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { setAuth, setUser } = useAppStore();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (code.length !== 6) {
      setError('Please enter a 6-digit code');
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const result = await invoke<{
        success: boolean;
        access_token?: string;
        user_id?: string;
        display_name?: string;
        error?: string;
      }>('verify_device_code', { code });

      if (result.success && result.access_token) {
        setAuth(true, result.access_token);
        setUser({
          id: result.user_id!,
          displayName: result.display_name || 'Driver',
        });
      } else {
        setError(result.error || 'Invalid code');
      }
    } catch (err) {
      console.error('Login error:', err);
      setError('Failed to connect. Check your internet connection.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleCodeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value.replace(/\D/g, '').slice(0, 6);
    setCode(value);
    setError(null);
  };

  return (
    <div className="login-view">
      {/* Header */}
      <div className="titlebar">
        <div className="titlebar-title">VTC Tracker</div>
        <div className="titlebar-controls">
          <button 
            className="titlebar-btn close"
            onClick={() => invoke('close_window')}
          >
            ✕
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="login-content">
        {/* Logo */}
        <div className="login-logo">
          <div className="logo-icon">
            <Truck size={32} />
          </div>
          <h1>VTC Tracker</h1>
          <p>Desktop Companion</p>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="login-form">
          <div className="card">
            <div className="card-header">
              <div className="card-title">
                <Monitor size={16} style={{ marginRight: 8 }} />
                Link Your Account
              </div>
            </div>
            <div className="card-content">
              <p className="login-instructions">
                Enter the 6-digit code from the VTC Tracker website.
              </p>

              {error && (
                <div className="login-error">
                  <AlertCircle size={16} />
                  {error}
                </div>
              )}

              <input
                type="text"
                className="input input-code"
                placeholder="000000"
                value={code}
                onChange={handleCodeChange}
                maxLength={6}
                disabled={isLoading}
                autoFocus
              />

              <button
                type="submit"
                className="btn btn-primary"
                disabled={isLoading || code.length !== 6}
                style={{ width: '100%' }}
              >
                {isLoading ? (
                  'Verifying...'
                ) : (
                  <>
                    Connect
                    <ArrowRight size={16} />
                  </>
                )}
              </button>
            </div>
          </div>
        </form>

        {/* Help */}
        <div className="login-help">
          <p>
            Get your code at:<br />
            <span className="login-url">yoursite.com/settings/device</span>
          </p>
        </div>
      </div>

      <style>{`
        .login-view {
          display: flex;
          flex-direction: column;
          height: 100vh;
          background: linear-gradient(180deg, var(--bg-primary) 0%, #0d0d14 100%);
        }

        .login-content {
          flex: 1;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: 32px;
          gap: 32px;
        }

        .login-logo {
          text-align: center;
        }

        .logo-icon {
          width: 64px;
          height: 64px;
          background: linear-gradient(135deg, var(--ets2), var(--ats));
          border-radius: 16px;
          display: flex;
          align-items: center;
          justify-content: center;
          margin: 0 auto 16px;
          color: white;
        }

        .login-logo h1 {
          font-size: 24px;
          font-weight: 700;
          margin-bottom: 4px;
        }

        .login-logo p {
          color: var(--foreground-muted);
          font-size: 14px;
        }

        .login-form {
          width: 100%;
          max-width: 320px;
        }

        .login-instructions {
          color: var(--foreground-muted);
          font-size: 13px;
          text-align: center;
          margin-bottom: 16px;
        }

        .login-error {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 12px;
          background: rgba(239, 68, 68, 0.1);
          border: 1px solid rgba(239, 68, 68, 0.3);
          border-radius: 8px;
          color: var(--error);
          font-size: 13px;
          margin-bottom: 16px;
        }

        .card-content .input {
          margin-bottom: 16px;
        }

        .login-help {
          text-align: center;
          font-size: 12px;
          color: var(--foreground-dim);
        }

        .login-url {
          color: var(--ets2);
        }
      `}</style>
    </div>
  );
}

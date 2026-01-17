import { create } from 'zustand';

interface User {
  id: string;
  displayName: string;
  avatarUrl?: string;
}

interface TelemetryData {
  game: 'ets2' | 'ats' | null;
  connected: boolean;
  speed: number;
  currentCity: string | null;
  activeJob: ActiveJob | null;
}

interface ActiveJob {
  cargo: string;
  sourceCity: string;
  destinationCity: string;
  distanceKm: number;
  distanceRemaining: number;
  revenue: number;
}

interface AppState {
  // Auth
  isAuthenticated: boolean;
  accessToken: string | null;
  user: User | null;
  
  // Connection
  isConnected: boolean;
  lastHeartbeat: Date | null;
  
  // Telemetry
  telemetry: TelemetryData;
  
  // Stats
  todayJobs: number;
  todayDistance: number;
  todayRevenue: number;
  
  // Actions
  setAuth: (isAuthenticated: boolean, accessToken?: string) => void;
  setUser: (user: User | null) => void;
  setConnected: (connected: boolean) => void;
  setTelemetry: (data: Partial<TelemetryData>) => void;
  updateStats: (jobs: number, distance: number, revenue: number) => void;
  logout: () => void;
}

export const useAppStore = create<AppState>((set) => ({
  // Initial state
  isAuthenticated: false,
  accessToken: null,
  user: null,
  isConnected: false,
  lastHeartbeat: null,
  telemetry: {
    game: null,
    connected: false,
    speed: 0,
    currentCity: null,
    activeJob: null,
  },
  todayJobs: 0,
  todayDistance: 0,
  todayRevenue: 0,
  
  // Actions
  setAuth: (isAuthenticated, accessToken) => set({
    isAuthenticated,
    accessToken: accessToken || null,
  }),
  
  setUser: (user) => set({ user }),
  
  setConnected: (connected) => set({
    isConnected: connected,
    lastHeartbeat: connected ? new Date() : null,
  }),
  
  setTelemetry: (data) => set((state) => ({
    telemetry: { ...state.telemetry, ...data },
  })),
  
  updateStats: (jobs, distance, revenue) => set({
    todayJobs: jobs,
    todayDistance: distance,
    todayRevenue: revenue,
  }),
  
  logout: () => set({
    isAuthenticated: false,
    accessToken: null,
    user: null,
    isConnected: false,
    telemetry: {
      game: null,
      connected: false,
      speed: 0,
      currentCity: null,
      activeJob: null,
    },
  }),
}));

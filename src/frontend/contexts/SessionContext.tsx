import { useRouter } from 'expo-router';
import * as SecureStore from 'expo-secure-store';
import React, { createContext, useState, useEffect, useContext } from 'react';
import api from '~/lib/api';

interface SessionContextType {
  session: string | null;
  setSession: (session: string) => void;
  logout: () => void;
}

const SessionContext = createContext<SessionContextType | undefined>(undefined);

export const SessionProvider = ({ children }: { children: React.ReactNode }) => {
  const [session, setSession] = useState<string | null>(null);
  const router = useRouter();

  useEffect(() => {
    const loadSession = async () => {
      const storedSession = await SecureStore.getItemAsync('session');
      if (storedSession) {
        setSession(storedSession);
      } else {
        router.replace('/login');
      }
    };

    loadSession();
  });

  const logout = async () => {
    try {
      await api.getSecure('/auth/logout');
    } catch (e) {
      // We really don't care if this fails
      console.error('Failed to logout', e);
    }
    await SecureStore.deleteItemAsync('session');
    setSession(null);
    router.replace('/login');
  };

  return (
    <SessionContext.Provider value={{ session, setSession, logout }}>
      {children}
    </SessionContext.Provider>
  );
};

export const useSession = () => {
  const context = useContext(SessionContext);
  if (!context) {
    throw new Error('useSession must be used within a SessionProvider');
  }
  return context;
};

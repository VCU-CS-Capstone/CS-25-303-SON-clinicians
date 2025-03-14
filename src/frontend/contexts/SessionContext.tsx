import { useRouter } from 'expo-router';
import * as SecureStore from 'expo-secure-store';
import React, { createContext, useState, useEffect, useContext } from 'react';
import api from '~/lib/api';

interface SessionContextType {
  session: string | null;
  sessionExpiration: Date | null;
  setSession: (session: string) => void;
  setSessionExpiration: (expiration: Date) => void;
  logout: () => void;

  isValid(): boolean;
}

const SessionContext = createContext<SessionContextType | undefined>(undefined);

export const SessionProvider = ({ children }: { children: React.ReactNode }) => {
  const [session, setSession] = useState<string | null>(null);
  const [sessionExpiration, setSessionExpiration] = useState<Date | null>(null);
  const router = useRouter();

  useEffect(() => {
    const loadSession = async () => {
      const storedSession = await SecureStore.getItemAsync('session');
      const expiration = await SecureStore.getItemAsync('session-expiration');
      const expirationDate = expiration ? new Date(expiration) : null;

      if (storedSession) {
        if (expirationDate) {
          setSessionExpiration(expirationDate);
          if (expirationDate < new Date()) {
            await SecureStore.deleteItemAsync('session');
            await SecureStore.deleteItemAsync('session-expiration');
            setSessionExpiration(null);
            setSession(null);
            router.replace('/(login)/LoginScreen');
            return;
          }
        }
        setSession(storedSession);
      } else {
        router.replace('/(login)/LoginScreen');
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
    setSessionExpiration(null);
  };
  const isValid = () =>
    session !== null && sessionExpiration !== null && sessionExpiration > new Date();

  return (
    <SessionContext.Provider
      value={{
        session,
        setSession,
        logout,
        sessionExpiration,
        setSessionExpiration,
        isValid,
      }}
    >
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

import { useRouter } from 'expo-router';
import * as SecureStore from 'expo-secure-store';
import React, { createContext, useState, useEffect, useContext } from 'react';
import api from '~/lib/api';
import { UserSession, UserSessionData } from '~/lib/types/user';

interface SessionContextType {
  session: UserSession | null;
  getSessionKey: () => string | null;
  setSession: (session: UserSessionData) => Promise<void>;
  logout: () => void;
  isValid(): boolean;
}

const SessionContext = createContext<SessionContextType | undefined>(undefined);

export const SessionProvider = ({ children }: { children: React.ReactNode }) => {
  const [session, setSessionValue] = useState<UserSession | null>(null);
  const router = useRouter();

  useEffect(() => {
    const loadSession = async () => {
      const storedSession = await SecureStore.getItemAsync('session');

      if (storedSession) {
        let sessionValue = JSON.parse(storedSession) as UserSessionData;
        if (!session) {
          console.trace('Restored Session', sessionValue);
        }
        setSessionValue(new UserSession(sessionValue));
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
    setSessionValue(null);
  };
  const isValid = () => {
    if (!session) {
      return false;
    }

    return !session.hasExpired();
  };
  const getSessionKey = () => {
    if (!session) {
      return null;
    }
    return session.session_key;
  };
  const setSession = async (session: UserSessionData) => {
    setSessionValue(new UserSession(session));
    await SecureStore.setItemAsync('session', JSON.stringify(session));
  };
  return (
    <SessionContext.Provider
      value={{
        session,
        getSessionKey,
        setSession,
        logout,
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

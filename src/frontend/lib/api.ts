import { SessionProvider, useSession } from '~/contexts/SessionContext';
import { PaginatedResponse } from './RequestTypes';
import {
  Participant,
  ParticipantLookupRequest,
  ParticipantLookupResponse,
  RecentVisit,
} from './types/participant';
import * as SecureStore from 'expo-secure-store';
import { Location } from './types/locations';

const API_URL = process.env.EXPO_PUBLIC_API_URL;

const api = {
  get: async (endpoint: string) => {
    const url = appendEndpoint(endpoint);
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
      credentials: 'include',
    });
    console.debug('Get Response', response);
    if (!response.ok) {
      throw new Error(`Failed to fetch ${endpoint}, Error: ${response.status}`);
    }
    return await response.json();
  },
  getSecure: async (endpoint: string) => {
    const session = await SecureStore.getItemAsync('session');
    const authHeader = session ? `Session ${session}` : undefined;
    const url = appendEndpoint(endpoint);
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        ...(authHeader && { Authorization: authHeader }),
      },
      credentials: 'include',
    });
    console.debug('Secure Get Response', response);
    if (!response.ok) {
      throw new Error(`Failed to fetch ${endpoint}, Error: ${response.status}`);
    }
    return await response.json();
  },
  post: async (endpoint: string, data: any) => {
    const url = appendEndpoint(endpoint);
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
      credentials: 'include',
    });
    console.debug('Post Response', response);
    if (!response.ok) {
      throw new Error(`Failed to post ${endpoint}, Error: ${response.status}`);
    }
    return await response.json();
  },
  postSecure: async (endpoint: string, data: any) => {
    const session = await SecureStore.getItemAsync('session');
    const authHeader = session ? `Session ${session}` : undefined;

    const url = appendEndpoint(endpoint);
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(authHeader && { Authorization: authHeader }),
      },
      body: JSON.stringify(data),
      credentials: 'include',
    });
    console.debug('Secure Post Response', response);
    if (!response.ok) {
      throw new Error(`Failed to post ${endpoint}, Error: ${response.status}`);
    }
    return await response.json();
  },
  participants: {
    fetchById: async (id: number) => {
      const response = await api.getSecure(`/participant/get/${id}`);
      console.log({ response });
      return response as Participant;
    },
    getRecentVisits: async (id: number) => {
      const response = await api.getSecure(`/participant/case_notes/${id}/list/all`);
      return response as RecentVisit[];
    },
    lookup: async (
      lookup: ParticipantLookupRequest
    ): Promise<PaginatedResponse<ParticipantLookupResponse>> => {
      const response = await api.postSecure('/participant/lookup', lookup);
      return response as PaginatedResponse<ParticipantLookupResponse>;
    },
  },

  locations: {
    fetchAll: async () => {
      const response = await api.getSecure('/location/all');
      return response as Location[];
    },
    fetchById: async (id: number) => {
      const response = await api.getSecure(`/location/${id}`);
      return response as Location;
    },
  },
  login: async (username: string, password: string) => {
    const response = await api.post('/auth/login/password', { username, password });
    console.log({ response });
    return response;
  },
};

function appendEndpoint(endpoint: string) {
  return `${API_URL}${endpoint}`;
}

export default api;

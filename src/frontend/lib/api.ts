import { BloodPressureStats, WeightEntry } from './types/stats';
import { PaginatedResponse } from './RequestTypes';
import {
  Participant,
  ParticipantDemographics,
  ParticipantHealthOverview,
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
    return response;
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
    return response;
  },
  participants: {
    fetchById: async (id: number) => {
      const response = await api.getSecure(`/participant/get/${id}`);
      if (response.ok) {
        return (await response.json()) as Participant;
      } else if (response.status === 404) {
        return undefined;
      } else {
        throw new Error(`Failed to fetch participant with id ${id}, Error: ${response.status}`);
      }
    },
    fetchDemographic: async (id: number) => {
      const response = await api.getSecure(`/participant/get/${id}/demographics`);
      if (response.ok) {
        return (await response.json()) as ParticipantDemographics;
      } else if (response.status === 404) {
        return undefined;
      } else {
        throw new Error(`Failed to fetch participant with id ${id}, Error: ${response.status}`);
      }
    },
    fetchHealthOverview: async (id: number) => {
      const response = await api.getSecure(`/participant/get/${id}/health_overview`);
      if (response.ok) {
        return (await response.json()) as ParticipantHealthOverview;
      } else if (response.status === 404) {
        return undefined;
      } else {
        throw new Error(`Failed to fetch participant with id ${id}, Error: ${response.status}`);
      }
    },
    getRecentVisits: async (id: number) => {
      const response = await api.getSecure(`/participant/case_notes/${id}/list/all`);
      if (response.ok) {
        return (await response.json()) as RecentVisit[];
      } else if (response.status === 404) {
        return undefined;
      } else {
        throw new Error(`Failed to fetch participant with id ${id}, Error: ${response.status}`);
      }
    },
    fetchBpHistory: async (id: number, page_size?: number, page?: number) => {
      const pageNumber = page || 1;
      const pageSize = page_size || 10;
      const response = await api.getSecure(
        `/participant/stats/bp/history/${id}?page_size=${pageSize}&page=${pageNumber}`
      );
      if (response.ok) {
        return (await response.json()) as PaginatedResponse<BloodPressureStats>;
      } else if (response.status === 404) {
        return undefined;
      } else {
        throw new Error(`Failed to fetch participant with id ${id}, Error: ${response.status}`);
      }
    },
    fetchWeightHistory: async (id: number, page_size?: number, page?: number) => {
      const pageNumber = page || 1;
      const pageSize = page_size || 10;
      const response = await api.getSecure(
        `/participant/stats/weight/history/${id}?page_size=${pageSize}&page=${pageNumber}`
      );
      if (response.ok) {
        return (await response.json()) as PaginatedResponse<WeightEntry>;
      } else if (response.status === 404) {
        return undefined;
      } else {
        throw new Error(`Failed to fetch participant with id ${id}, Error: ${response.status}`);
      }
    },
    lookup: async (
      lookup: ParticipantLookupRequest
    ): Promise<PaginatedResponse<ParticipantLookupResponse>> => {
      const response = await api.postSecure('/participant/lookup', lookup);
      if (!response.ok) {
        throw new Error(`Failed to fetch participant lookup, Error: ${response.status}`);
      }
      return (await response.json()) as PaginatedResponse<ParticipantLookupResponse>;
    },
  },

  locations: {
    fetchAll: async () => {
      const response = await api.getSecure('/location/all');
      return (await response.json()) as Location[];
    },
    fetchById: async (id: number) => {
      const response = await api.getSecure(`/location/${id}`);
      return (await response.json()) as Location;
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

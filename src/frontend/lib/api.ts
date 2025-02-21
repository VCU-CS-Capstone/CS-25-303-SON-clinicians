import { BloodPressureStats, GlucoseEntry, WeightEntry } from './types/stats';
import { PaginatedResponse, SiteInfo } from './RequestTypes';
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
import { MedicationEntry } from './types/medications';

export const API_URL = process.env.EXPO_PUBLIC_API_URL || 'https://cs-25-303.wyatt-herkamp.dev/api';

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
    try {
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
      // Get "x-request-id" header from response
      const requestId = response.headers.get('x-request-id');
      if (requestId) {
        console.debug('Request ID:', requestId);
      } else {
        console.warn('No Request ID');
      }
      console.debug('Secure Get Response', response);
      return response;
    } catch (e) {
      throw e;
    }
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
    try {
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
      // Get "x-request-id" header from response
      const requestId = response.headers.get('x-request-id');
      if (requestId) {
        console.debug('Request ID:', requestId);
      } else {
        console.warn('No Request ID');
      }
      console.debug('Secure Get Response', response);
      return response;
    } catch (e) {
      throw e;
    }
  },
  // Fetch site info
  // Scalar: https://cs-25-303.wyatt-herkamp.dev/scalar#tag/api/GET/api/info
  siteInfo: async () => {
    const response = await api.get('/info');
    return response as SiteInfo;
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
    fetchWeightHistory: async (id: number, pageSize: number = 15, pageNumber: number = 1) => {
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
    fetchGlucoseHistory: async (id: number, pageSize: number = 15, pageNumber: number = 1) => {
      const response = await api.getSecure(
        `/participant/stats/glucose/history/${id}?page_size=${pageSize}&page=${pageNumber}`
      );
      if (response.ok) {
        return (await response.json()) as PaginatedResponse<GlucoseEntry>;
      } else if (response.status === 404) {
        return undefined;
      } else {
        throw new Error(`Failed to fetch participant with id ${id}, Error: ${response.status}`);
      }
    },
    fetchParticipantMedications: async (id: number, page_size: number = 10, page: number = 1) => {
      const response = await api.getSecure(
        `/participant/medications/${id}/search?page_size=${page_size}&page=${page}`
      );
      if (response.ok) {
        return (await response.json()) as PaginatedResponse<MedicationEntry>;
      } else if (response.status === 404) {
        return undefined;
      } else {
        throw new Error(`Failed to fetch participant with id ${id}, Error: ${response.status}`);
      }
    },
    lookup: async (
      lookup: ParticipantLookupRequest,
      page_size: number = 15,
      page: number = 1
    ): Promise<PaginatedResponse<ParticipantLookupResponse>> => {
      const response = await api.postSecure(
        `/participant/lookup?page_size=${page_size}&page=${page}`,
        lookup
      );
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

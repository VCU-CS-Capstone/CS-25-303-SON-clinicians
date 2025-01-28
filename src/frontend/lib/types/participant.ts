export interface Participant {
  id: number;
  first_name: string;
  last_name: string;
  email: string;
  phone_number_one: string;
  date_of_birth: string;
}

export interface RecentVisit {
  id: number;
  participant_id: number;
  date_of_visit: string;
  visit_type: string;
  location: number;
}

export interface ParticipantLookupRequest {
  program?: string | null;
  first_name?: string;
  last_name?: string;
}

export interface ParticipantLookupResponse {
  id: number;
  first_name: string;
  last_name: string;
  phone_number_one?: string;
  phone_number_two?: string;
  program: string;
  location: number;
}

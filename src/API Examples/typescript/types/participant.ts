import { Program } from "./locations.ts";

export enum ParticipantStatus {
  Active = "Active",
  Inactive = "Inactive",
  NoValidContactStatus = "NoValidContactStatus",
  Deceases = "Deceases",
  Withdrew = "Withdrew",
}
export interface Participant {
  id: number;
  first_name: string;
  last_name: string;
  email: string;
  phone_number_one: string;
  date_of_birth: string;
  status: ParticipantStatus;
  program: Program;
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

export interface ParticipantDemographics {
  participant_id: number;
  age?: number;
  ethnicity?: string;
  is_veteran?: boolean;
  language?: string;
  health_insurance?: HealthInsurance[];
  highest_education_level?: string;
}
export interface ParticipantHealthOverview {
  participant_id: number;
  alergies?: string;
  reported_health_conditions?: string;
  takes_more_than_5_medications?: boolean;
  has_blood_pressure_cuff?: boolean;
  height?: number;
}

export enum HealthInsurance {
  Medicaid = "Medicaid",
  Medicare = "Medicare",
  Private = "Private",
  VA = "VA",
  None = "None",
}

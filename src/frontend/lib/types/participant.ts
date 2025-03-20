import { Program } from './locations';

export enum ParticipantStatus {
  Active = 'Active',
  Inactive = 'Inactive',
  NoValidContactStatus = 'NoValidContactStatus',
  Deceases = 'Deceases',
  Withdrew = 'Withdrew',
}
export namespace ParticipantStatus {
  export function title(status: ParticipantStatus): string {
    switch (status) {
      case ParticipantStatus.Active:
        return 'Active';
      case ParticipantStatus.Inactive:
        return 'Inactive';
      case ParticipantStatus.NoValidContactStatus:
        return 'No Valid Contact Status';
      case ParticipantStatus.Deceases:
        return 'Deceased';
      case ParticipantStatus.Withdrew:
        return 'Withdrew';
    }
  }
}
export enum VCUHSPatientStatus {
  Yes = 'Yes',
  No = 'No',
  Unsure = 'Unsure',
  DidNotAsk = 'DidNotAsk',
}
export interface Participant {
  id: number;
  first_name: string;
  last_name: string;
  email: string;
  phone_number_one: string;
  phone_number_two: string;
  other_contact: string;
  date_of_birth: string;
  status: ParticipantStatus;
  vcuhs_patient_status?: VCUHSPatientStatus;
  program: Program;

  signed_up_on: string;
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
export interface ParticipantRelatedDataNotFound {
  participant_exists: boolean;
}
export interface ParticipantRelatedData<T> {
  participant_exists: boolean;
  data: T | undefined;
}
export enum HealthInsurance {
  Medicaid = 'Medicaid',
  Medicare = 'Medicare',
  Private = 'Private',
  VA = 'VA',
  None = 'None',
}

export namespace HealthInsurance {
  export function fullName(insurance: HealthInsurance): string {
    switch (insurance) {
      case HealthInsurance.Medicaid:
        return 'Medicaid';
      case HealthInsurance.Medicare:
        return 'Medicare';
      case HealthInsurance.Private:
        return 'Private';
      case HealthInsurance.VA:
        return 'Veterans Affairs';
      case HealthInsurance.None:
        return 'None';
    }
  }
}

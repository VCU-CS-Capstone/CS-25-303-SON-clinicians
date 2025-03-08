import { HealthInsurance, ParticipantStatus } from "./participant.ts";
import { Program } from "./locations.ts";

export interface ResearcherParticipant {
  participant_id: number;
  red_cap_id: number | null;
  first_name: string;
  last_name: string;
  phone_number_one: string | null;
  phone_number_two: null | null;
  other_contact: string | null;
  visit_history: Date[] | undefined;
  last_visited: Date | undefined;
}
// Acceptable values for
// Any number.
// A string in the format of ">25" or "<25" or "25" or ">=25" or "<=25" or "25..30"
export type NumberQuery = number | string;
export interface ResearcherQuery {
  location: number | number[];
  program?: Program;
  vcuhs_patient_status?: "Yes" | "No" | "Unsure" | "DidNotAsk";

  status?: ParticipantStatus;
  gender?: "male" | "female" | "Transgender" | "NonBinary" | string;
  highest_level_of_education?: string;
  race?: string;
  language?: string;
  health_insurance?: HealthInsurance;

  age?: NumberQuery;

  get_visit_history?: boolean;
  get_last_visited?: boolean;

  bmi: NumberQuery;
  blood_pressure?: ResearcherBloodPressure;
  glucose?: ResearcherBloodGlucose;
}
export interface ResearcherBloodPressure {
  reading_type?: "sit" | "stand" | "personal";
  systolic?: NumberQuery;
  diastolic?: NumberQuery;
}

export interface ResearcherBloodGlucose {
  fasted_atleast_2_hours?: boolean;
  glucose: NumberQuery;
}

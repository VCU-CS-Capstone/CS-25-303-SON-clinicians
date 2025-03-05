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

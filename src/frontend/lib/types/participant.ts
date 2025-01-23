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

export interface Location {
  id: number;
  name: string;
  program: string;
  parent_location?: number;
}

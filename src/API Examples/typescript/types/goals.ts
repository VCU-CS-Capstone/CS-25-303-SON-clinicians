export interface Goal {
  id: number;
  is_active?: boolean;
  participant_id: number;
  goal: string;
}

export interface GoalStep {
  id: number;
  goal_id?: number;
  participant_id: number;

  step: string;
  confidence_level?: number;
  is_complete?: boolean;
  date_set?: string;
  date_to_be_completed?: string;
}

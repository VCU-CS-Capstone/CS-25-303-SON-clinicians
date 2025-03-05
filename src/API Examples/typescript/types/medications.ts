export interface MedicationEntry {
  id: number;
  name: string;
  dosage: string;
  frequency: string;
  date_prescribed?: string;
  date_entered_into_system: string;
  is_current?: boolean;
  comments?: string;
}

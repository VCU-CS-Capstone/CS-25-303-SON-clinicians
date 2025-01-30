export interface BloodPressureStats {
  case_note_id: number;
  date_of_visit: string;
  blood_pressure: {
    Sit?: BloodPressureReading;
    Stand?: BloodPressureReading;
  };
}

export interface BloodPressureReading {
  systolic: number;
  diastolic: number;
}
export interface BloodPressureStatsOneReading {
  case_note_id: number;
  date_of_visit: Date;
  blood_pressure: BloodPressureReading;
}
export interface WeightEntry {
  case_note_id: number;
  date_of_visit: string;
  weight: number;
}

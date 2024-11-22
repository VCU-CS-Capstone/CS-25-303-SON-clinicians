use chrono::NaiveDate;

use crate::{
    database::red_cap::participants::{NewMedication, ParticipantMedications},
    red_cap::{MedStatus, RedCapDataSet, RedCapMedicationFrequency, RedCapType},
};

#[derive(Debug, Clone, PartialEq)]
pub struct RedCapMedication {
    pub name: String,
    pub dosage: Option<String>,
    pub frequency: Option<RedCapMedicationFrequency>,
    pub date_prescribed: Option<NaiveDate>,
    pub date_entered_into_system: Option<NaiveDate>,
    pub status: Option<MedStatus>,
    pub date_discontinued: Option<NaiveDate>,
    pub comments: Option<String>,
    pub red_cap_index: Option<i32>,
}
impl From<ParticipantMedications> for RedCapMedication {
    fn from(value: ParticipantMedications) -> Self {
        let ParticipantMedications {
            name,
            dosage,
            frequency,
            date_prescribed,
            date_entered_into_system,
            is_current,
            date_discontinued,
            comments,
            red_cap_index,
            ..
        } = value;

        Self {
            name,
            dosage,
            frequency: frequency.map(Into::into),
            date_prescribed,
            date_entered_into_system,
            status: is_current.map(Into::into),
            date_discontinued,
            comments,
            red_cap_index,
        }
    }
}
impl From<RedCapMedication> for NewMedication {
    fn from(value: RedCapMedication) -> Self {
        let RedCapMedication {
            name,
            dosage,
            frequency,
            date_prescribed,
            date_entered_into_system,
            status,
            date_discontinued,
            comments,
            red_cap_index,
        } = value;

        Self {
            name,
            dosage,
            frequency: frequency.map(|x| x.into()),
            date_prescribed,
            date_entered_into_system,
            is_current: status.map(|x| x.into()),
            date_discontinued,
            comments,
            red_cap_index,
        }
    }
}
impl RedCapMedication {
    pub fn read_index<D: RedCapDataSet>(data: &D, index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        let name = data.get_string(format!("med{}", index).as_str())?;
        let dosage = data.get_string(format!("dosage{}", index).as_str());
        let frequency = RedCapMedicationFrequency::read_with_index(data, index);
        let date_prescribed = data.get_date(format!("med_date{}", index).as_str());
        let date_entered_into_system = data.get_date(format!("med_red_{}", index).as_str());
        let status = data.get_enum::<MedStatus>(format!("med_status{}", index).as_str());
        let date_discontinued = data.get_date(format!("med_dis{}", index).as_str());
        let comments = data.get_string(format!("med_other{}", index).as_str());

        Some(Self {
            name,
            dosage,
            frequency,
            date_prescribed,
            date_entered_into_system,
            status,
            date_discontinued,
            comments,
            red_cap_index: Some(index as i32),
        })
    }

    pub fn read<D: RedCapDataSet>(data: &D) -> Vec<Self>
    where
        Self: Sized,
    {
        (1..=40).filter_map(|x| Self::read_index(data, x)).collect()
    }

    pub fn write<D: RedCapDataSet>(&self, data: &mut D)
    where
        Self: Sized,
    {
        let index = self.red_cap_index.unwrap_or(1) as usize;
        data.insert(format!("med{}", index), self.name.clone().into());
        if let Some(dosage) = &self.dosage {
            data.insert(format!("dosage{}", index), dosage.clone().into());
        }
        if let Some(frequency) = &self.frequency {
            frequency.write_with_index(data, index);
        }
        if let Some(date_prescribed) = self.date_prescribed {
            data.insert(format!("med_date{}", index), date_prescribed.into());
        }
        if let Some(date_entered_into_system) = self.date_entered_into_system {
            data.insert(
                format!("med_red_{}", index),
                date_entered_into_system.into(),
            );
        }
        if let Some(status) = &self.status {
            data.insert(format!("med_status{}", index), status.clone().into());
        }
        if let Some(date_discontinued) = self.date_discontinued {
            data.insert(format!("med_dis{}", index), date_discontinued.into());
        }
        if let Some(comments) = &self.comments {
            data.insert(format!("med_other{}", index), comments.clone().into());
        }
    }
}

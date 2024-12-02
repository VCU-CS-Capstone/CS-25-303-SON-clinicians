use chrono::NaiveDate;

use crate::{
    database::red_cap::participants::{NewMedication, ParticipantMedications},
    red_cap::{MedStatus, RedCapDataSet, RedCapMedicationFrequency, RedCapType},
};
/// Medications are a form with a set of fields that repeat 40 times.
#[derive(Debug, Clone, PartialEq)]
pub struct RedCapMedication {
    /// The name of the medication
    ///
    /// Redcap Field: `med{index}`
    pub name: String,
    /// The dosage of the medication
    ///
    /// Redcap Field: `dosage{index}`
    pub dosage: Option<String>,
    /// The frequency of the medication
    pub frequency: Option<RedCapMedicationFrequency>,
    /// The date the medication was prescribed
    ///
    /// Redcap Field: `med_date{index}`
    pub date_prescribed: Option<NaiveDate>,
    /// The date the medication was entered into the system
    ///
    /// Redcap Field: `med_red_{index}`
    pub date_entered_into_system: Option<NaiveDate>,
    /// The status of the medication
    ///
    /// Redcap Field: `med_status{index}`
    pub status: Option<MedStatus>,
    /// The date the medication was discontinued
    pub date_discontinued: Option<NaiveDate>,
    /// Comments about the medication
    pub comments: Option<String>,
    /// The index of the medication in red cap\
    /// 1-40
    ///
    /// This is used for syncing purposes
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
    /// Reads a medication from a red cap data set
    ///
    /// # Note
    /// if name is null. This will return None
    ///
    /// Otherwise it accepts null values for all other fields
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
    /// Reads all medications from a red cap data set
    ///
    /// Just a loop of 40 times.
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

use crate::utils::InvalidVariant;
use cs25_303_macros::RedCapEnum;
use serde::Serialize;
use strum::EnumIter;
mod multi_select;
pub use multi_select::*;
use utoipa::ToSchema;

use crate::red_cap::{utils::is_all_none, RedCapDataSet, RedCapEnum, RedCapType};

/// The two Program Types that are available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, RedCapEnum, ToSchema)]
pub enum Programs {
    #[default]
    #[red_cap(enum_index = 1)]
    /// Richmond Health And Wellness Program
    RHWP,
    /// Mobile Health And Wellness Program
    #[red_cap(enum_index = 2)]
    MHWP,
}
/// Participant Status
#[derive(Debug, Clone, PartialEq, Eq, RedCapEnum, ToSchema)]
pub enum Status {
    #[red_cap(enum_index = 1)]
    Active,
    #[red_cap(enum_index = 0)]
    Inactive,
    #[red_cap(enum_index = 3)]
    NoValidContactStatus,
    #[red_cap(enum_index = 4)]
    Deceases,
    #[red_cap(enum_index = 5)]
    Withdrew,
}

/// Have they been seen at VCUHS
#[derive(Debug, Clone, PartialEq, Eq, RedCapEnum, ToSchema)]
pub enum SeenAtVCUHS {
    #[red_cap(enum_index = 1)]
    Yes,
    #[red_cap(enum_index = 0)]
    No,
    #[red_cap(enum_index = 2)]
    Unsure,
    #[red_cap(enum_index = 3)]
    DidNotAsk,
}
/// Handles reading the data from the redcap gender and gender_self fields from the redcap data
#[derive(Debug, Clone, Serialize)]
pub struct RedCapGender {
    /// Gender Enum Value
    ///
    /// ## Notes
    /// - Radio
    pub gender: Option<Gender>,
    pub gender_self: Option<String>,
}

impl RedCapType for RedCapGender {
    fn read<D: RedCapDataSet>(data: &D) -> Option<Self>
    where
        Self: Sized,
    {
        let gender = data.get_enum("gender");
        let gender_self = data.get_string("gender_self");
        is_all_none!(gender, gender_self);
        Some(Self {
            gender,
            gender_self,
        })
    }

    fn write<D: RedCapDataSet>(&self, data: &mut D) {
        data.insert("gender", self.gender.clone().into());
        data.insert("gender_self", self.gender_self.clone().into());
    }
}
impl From<Gender> for RedCapGender {
    fn from(value: Gender) -> Self {
        let gender_self = match &value {
            Gender::PreferToSelfDescribe(value) => Some(value.clone()),
            _ => None,
        };
        Self {
            gender: Some(value),
            gender_self,
        }
    }
}
impl From<RedCapGender> for Option<Gender> {
    fn from(value: RedCapGender) -> Self {
        let RedCapGender {
            gender,
            gender_self,
        } = value;
        if let Some(string) = gender_self {
            if string.is_empty() {
                return gender;
            }
        }
        gender
    }
}
/// Gender Enum
#[derive(Debug, Clone, PartialEq, Eq, RedCapEnum, ToSchema)]
pub enum Gender {
    #[red_cap(name = "female", enum_index = 2)]
    Female,
    #[red_cap(name = "male", enum_index = 1)]
    Male,
    #[red_cap(enum_index = 3)]
    Transgender,
    #[red_cap(enum_index = 4)]
    NonBinary,
    #[red_cap(enum_index = 6)]
    PreferNotToAnswer,
    #[red_cap(other, enum_index = 5)]
    PreferToSelfDescribe(String),
}

#[derive(Debug, Clone, PartialEq, Eq, RedCapEnum, ToSchema, EnumIter)]
pub enum Ethnicity {
    #[red_cap(enum_index = 1)]
    HispanicOrLatino,
    #[red_cap(enum_index = 0)]
    NotHispanicOrLatino,
}

#[derive(Debug, Clone, PartialEq, Eq, RedCapEnum, ToSchema)]
pub enum PreferredLanguage {
    #[red_cap(enum_index = 1)]
    EnUs,
    #[red_cap(enum_index = 2)]
    Spanish,
    #[red_cap(enum_index = 3)]
    Asl,
    #[red_cap(other, enum_index = 4)]
    Other(String),
}
#[derive(Debug, Clone, Serialize)]
pub struct RedCapLanguage {
    pub language: Option<PreferredLanguage>,
    pub language_other: Option<String>,
}
impl From<PreferredLanguage> for RedCapLanguage {
    fn from(value: PreferredLanguage) -> Self {
        let language_other = match &value {
            PreferredLanguage::Other(value) => Some(value.clone()),
            _ => None,
        };
        Self {
            language: Some(value),
            language_other,
        }
    }
}
impl From<RedCapLanguage> for Option<PreferredLanguage> {
    fn from(value: RedCapLanguage) -> Self {
        let RedCapLanguage {
            language,
            language_other,
        } = value;
        if let Some(string) = language_other {
            if string.is_empty() {
                return language;
            }
        }
        language
    }
}
impl RedCapType for RedCapLanguage {
    fn read<D: RedCapDataSet>(data: &D) -> Option<Self>
    where
        Self: Sized,
    {
        let language = data.get_enum("language");
        let language_other = data.get_string("language_other");
        is_all_none!(language, language_other);
        Some(Self {
            language,
            language_other,
        })
    }

    fn write<D: RedCapDataSet>(&self, data: &mut D) {
        data.insert("language", self.language.clone().into());
        data.insert("language_other", self.language_other.clone().into());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, RedCapEnum, ToSchema)]
pub enum EducationLevel {
    #[red_cap(enum_index = 1)]
    None,
    #[red_cap(enum_index = 2)]
    Nursery,
    #[red_cap(enum_index = 3)]
    SomeHighSchool,
    #[red_cap(enum_index = 4)]
    HighschoolOrGED,
    #[red_cap(enum_index = 5)]
    SomeCollege,
    #[red_cap(enum_index = 6)]
    Trade,
    #[red_cap(enum_index = 7)]
    Associates,
    #[red_cap(enum_index = 8)]
    Bachelors,
    #[red_cap(enum_index = 9)]
    Graduates,
}

#[derive(Debug, Clone, PartialEq, Eq, RedCapEnum, ToSchema, EnumIter)]
pub enum MedicationFrequency {
    #[red_cap(name = "Daily", enum_index = 1)]
    Daily,
    #[red_cap(name = "TwiceADay", enum_index = 2)]
    TwiceADay,
    #[red_cap(enum_index = 3)]
    ThriceADay,
    #[red_cap(enum_index = 4)]
    FourTimesADay,
    #[red_cap(enum_index = 5)]
    AsNeeded,
    #[red_cap(other, enum_index = 6)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedCapMedicationFrequency {
    /// The frequency of the medication
    ///
    /// Redcap Field: `med_freq{index}`
    pub frequency: Option<MedicationFrequency>,
    /// The other value for the frequency
    ///
    /// Redcap Field: `other_med{index}`
    pub frequency_other: Option<String>,
}
impl From<MedicationFrequency> for RedCapMedicationFrequency {
    fn from(value: MedicationFrequency) -> Self {
        let frequency_other = match &value {
            MedicationFrequency::Other(value) => Some(value.clone()),
            _ => None,
        };
        Self {
            frequency: Some(value),
            frequency_other,
        }
    }
}
impl RedCapType for RedCapMedicationFrequency {
    fn read_with_index<D: RedCapDataSet>(data: &D, index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        let frequency = data.get_enum(format!("frequency{}", index).as_str());
        let frequency_other = data.get_string(format!("other_med{}", index).as_str());
        is_all_none!(frequency, frequency_other);
        Some(Self {
            frequency,
            frequency_other,
        })
    }

    fn write_with_index<D: RedCapDataSet>(&self, data: &mut D, index: usize)
    where
        Self: Sized,
    {
        data.insert(format!("frequency{}", index), self.frequency.clone().into());
        data.insert(
            format!("other_med{}", index),
            self.frequency_other.clone().into(),
        );
    }
    fn read<D: RedCapDataSet>(_: &D) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }

    fn write<D: RedCapDataSet>(&self, _: &mut D) {}
}
impl From<RedCapMedicationFrequency> for MedicationFrequency {
    fn from(value: RedCapMedicationFrequency) -> Self {
        match value.frequency {
            Some(MedicationFrequency::Other(value)) => MedicationFrequency::Other(value),
            Some(value) => value,
            None => panic!("Frequency should not be none"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, RedCapEnum)]
pub enum MedStatus {
    #[red_cap(enum_index = 1)]
    Current,
    #[red_cap(enum_index = 2)]
    Discontinued,
}
impl From<MedStatus> for bool {
    fn from(value: MedStatus) -> Self {
        match value {
            MedStatus::Current => true,
            MedStatus::Discontinued => false,
        }
    }
}
impl From<bool> for MedStatus {
    fn from(value: bool) -> Self {
        if value {
            MedStatus::Current
        } else {
            MedStatus::Discontinued
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, RedCapEnum, ToSchema)]
pub enum VisitType {
    #[red_cap(enum_index = 1)]
    Onsite,
    #[red_cap(enum_index = 2)]
    HomeVisit,
    #[red_cap(enum_index = 3)]
    OnsiteAndHome,
    #[red_cap(enum_index = 6)]
    Telephone,
    #[red_cap(enum_index = 8)]
    RBHIAndRHWP,
    #[red_cap(enum_index = 9)]
    PPPAndRHWP,
}

use chrono::{Local, NaiveDate};
use serde::Serialize;
use tracing::info;

use crate::{
    database::red_cap::{
        locations::{RedCapLocationConnectionRules, RedCapLocationRules},
        participants::{
            health_overview::HealthOverview, NewDemographics, NewHealthOverview, NewParticipant,
            ParticipantDemograhics, Participants,
        },
    },
    red_cap::{
        DegreeLevel, Ethnicity, HealthInsurance, MobilityDevice, MultiSelectType, Programs,
        RedCapDataSet, RedCapEnum, RedCapGender, RedCapLanguage, RedCapRace, RedCapType, Status,
    },
};

use super::{RedCapConverter, RedCapConverterError};

#[derive(Debug, Default, Serialize)]
pub struct RedCapParticipant {
    pub red_cap_id: Option<i32>,
    /// Redcap: first_name
    pub first_name: String,
    /// Red Cap last_name
    pub last_name: String,
    /// RedCap: phone1
    pub phone_number_one: Option<String>,
    /// RedCap: phone2
    pub phone_number_two: Option<String>,
    /// RedCap: other_info
    pub other_contact: Option<String>,
    pub program: Programs,
    /// Redcap: rhwp_location
    /// Relates to [super::Locations]
    pub location: Option<i32>,
    /// Red Cap: pt_status
    pub status: Option<Status>,
    /// Red Cap: behav_health_risk
    pub behavioral_risks_identified: Option<String>,
    /// Red Cap: consent_cc
    pub date_care_coordination_consent_signed: Option<NaiveDate>,
    /// Red Cap: consent_home
    pub date_home_visit_consent_signed: Option<NaiveDate>,
    /// Red Cap: date_intake
    pub signed_up_on: NaiveDate,
}
impl From<RedCapParticipant> for NewParticipant {
    fn from(value: RedCapParticipant) -> Self {
        let RedCapParticipant {
            red_cap_id,
            first_name,
            last_name,
            phone_number_one,
            phone_number_two,
            other_contact,
            program,
            location,
            status,
            behavioral_risks_identified,
            date_care_coordination_consent_signed,
            date_home_visit_consent_signed,
            signed_up_on,
        } = value;
        NewParticipant {
            red_cap_id,
            first_name,
            last_name,
            phone_number_one,
            phone_number_two,
            other_contact,
            program,
            location,
            status,
            behavioral_risks_identified,
            date_care_coordination_consent_signed,
            date_home_visit_consent_signed,
            signed_up_on,
            last_synced_with_redcap: Some(Local::now().into()),
        }
    }
}
impl From<Participants> for RedCapParticipant {
    fn from(participant: Participants) -> Self {
        Self {
            red_cap_id: participant.red_cap_id,
            first_name: participant.first_name,
            last_name: participant.last_name,
            program: participant.program,
            phone_number_one: participant.phone_number_one,
            phone_number_two: participant.phone_number_two,
            other_contact: participant.other_contact,
            location: participant.location,
            status: participant.status,
            behavioral_risks_identified: participant.behavioral_risks_identified,
            date_care_coordination_consent_signed: participant
                .date_care_coordination_consent_signed,
            date_home_visit_consent_signed: participant.date_home_visit_consent_signed,
            signed_up_on: participant.signed_up_on,
        }
    }
}
impl RedCapParticipant {
    pub async fn write_to_data_set<D: RedCapDataSet>(
        &self,
        data: &mut D,
        converter: &mut RedCapConverter,
    ) -> Result<(), RedCapConverterError> {
        let Self {
            red_cap_id,
            first_name,
            last_name,
            phone_number_one,
            phone_number_two,
            other_contact,
            program,
            location,
            status,
            behavioral_risks_identified,
            date_care_coordination_consent_signed,
            date_home_visit_consent_signed,
            signed_up_on,
        } = self;
        if let Some(red_cap_id) = red_cap_id {
            data.insert("record_id".to_string(), (*red_cap_id as usize).into());
        }

        data.insert("first_name".to_string(), first_name.clone().into());
        data.insert("last_name".to_string(), last_name.clone().into());
        data.insert("program".to_string(), program.to_usize().into());
        if phone_number_one.is_some() || phone_number_two.is_some() {
            data.insert("phone", true.into());
            data.insert("phone1".to_string(), phone_number_one.clone().into());
            data.insert("phone2".to_string(), phone_number_two.clone().into());
        }
        data.insert("other_info".to_string(), other_contact.clone().into());
        data.insert("pt_status".to_string(), status.clone().into());
        data.insert(
            "behav_health_risk".to_string(),
            behavioral_risks_identified.clone().into(),
        );
        data.insert(
            "consent_cc".to_string(),
            (*date_care_coordination_consent_signed).into(),
        );
        data.insert(
            "consent_home".to_string(),
            (*date_home_visit_consent_signed).into(),
        );
        data.insert("date_intake".to_string(), (*signed_up_on).into());

        if let Some(location) = location {
            let location = converter.locations.iter().find(|x| x.id == *location);
            if let Some(location) = location {
                location
                    .red_cap_connection_rules
                    .participant_rules()
                    .write(data);
            }
        }

        Ok(())
    }
    pub async fn read_participant<D: RedCapDataSet>(
        data: &D,
        converter: &mut RedCapConverter,
    ) -> Result<Self, RedCapConverterError> {
        let first_name = data
            .get("first_name")
            .and_then(|x| x.to_string())
            .ok_or(RedCapConverterError::RequiredFieldMissing("first_name"))?;

        let last_name = data
            .get("last_name")
            .and_then(|x| x.to_string())
            .ok_or(RedCapConverterError::RequiredFieldMissing("last_name"))?;
        let program = data
            .get_enum("program")
            .ok_or(RedCapConverterError::RequiredFieldMissing("program"))?;
        let red_cap_id = data
            .get_number("record_id")
            .ok_or(RedCapConverterError::RequiredFieldMissing("record_id"))?;

        let location = if let Some(location) = RedCapLocationRules::read(data) {
            let location = converter.find_location_from_connection_rules(&location);
            info!("Location: {:?}", location);
            location.map(|x| x.id)
        } else {
            None
        };

        let result = Self {
            red_cap_id: Some(red_cap_id as i32),
            first_name,
            last_name,
            program,
            phone_number_one: data.get("phone1").and_then(|x| x.to_string()),
            phone_number_two: data.get("phone2").and_then(|x| x.to_string()),
            other_contact: data.get("other_info").and_then(|x| x.to_string()),
            location,
            status: data.get("pt_status").and_then(|x| x.to_enum::<Status>()),
            behavioral_risks_identified: data.get("behav_health_risk").and_then(|x| x.to_string()),
            date_care_coordination_consent_signed: data.get("consent_cc").and_then(|x| x.to_date()),
            date_home_visit_consent_signed: data.get("consent_home").and_then(|x| x.to_date()),
            signed_up_on: data
                .get("date_intake")
                .and_then(|x| x.to_date())
                .unwrap_or_else(|| Local::now().date_naive()),
        };

        Ok(result)
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RedCapParticipantDemographics {
    /// Redcap: age
    pub age: Option<i16>,
    /// Redcap Gender
    pub gender: Option<RedCapGender>,
    /// Redcap: Race
    pub race: Option<RedCapRace>,
    /// Red Cap: ethnicity
    pub ethnicity: Option<Ethnicity>,
    pub language: Option<RedCapLanguage>,
    /// Red Cap: veteran
    /// Yes Or No
    pub is_veteran: Option<bool>,
    /// Red Cap: insurance
    pub health_insurance: Vec<HealthInsurance>,
    /// Red Cap: education
    pub highest_education_level: Option<DegreeLevel>,
}
impl From<RedCapParticipantDemographics> for Option<NewDemographics> {
    fn from(demographics: RedCapParticipantDemographics) -> Self {
        let RedCapParticipantDemographics {
            age,
            gender,
            race,
            ethnicity,
            language,
            is_veteran,
            health_insurance,
            highest_education_level,
        } = demographics;
        if age.is_none()
            && gender.is_none()
            && race.is_none()
            && ethnicity.is_none()
            && language.is_none()
            && is_veteran.is_none()
            && health_insurance.is_empty()
            && highest_education_level.is_none()
        {
            return None;
        }
        let RedCapRace {
            race,
            race_multiracial_other,
            race_other,
        } = race.unwrap_or(RedCapRace::default());

        let result = NewDemographics {
            age,
            gender: gender.and_then(Into::into),
            race,
            race_multiple: race_multiracial_other,
            race_other,
            ethnicity,
            health_insurance,
            highest_education_level,
            language: language.and_then(Into::into),
            is_veteran,
        };

        Some(result)
    }
}
impl From<ParticipantDemograhics> for RedCapParticipantDemographics {
    fn from(demographics: ParticipantDemograhics) -> Self {
        let race = RedCapRace {
            race: demographics.race,
            race_multiracial_other: demographics.race_multiple,
            race_other: demographics.race_other,
        };

        Self {
            age: demographics.age,
            gender: demographics.gender.map(|x| x.into()),
            race: Some(race),
            ethnicity: demographics.ethnicity,
            language: demographics.language.map(|x| x.into()),
            is_veteran: demographics.is_veteran,
            health_insurance: demographics.health_insurance,
            highest_education_level: demographics.highest_education_level,
        }
    }
}
impl RedCapParticipantDemographics {
    pub async fn read<D: RedCapDataSet>(data: &D) -> Result<Self, RedCapConverterError> {
        let result = Self {
            age: data.get_number("age").map(|x| x as i16),
            gender: RedCapGender::read(data),
            is_veteran: data.get_bool("is_veteran"),
            race: RedCapRace::read(data),
            ethnicity: data.get_enum("ethnicity"),
            health_insurance: data.get_enum_multi_select("insurance").unwrap_or_default(),
            highest_education_level: data.get_enum("education"),
            language: RedCapLanguage::read(data),
        };

        Ok(result)
    }
    pub fn write<D: RedCapDataSet>(&self, data: &mut D) {
        let Self {
            age,
            gender,
            race,
            ethnicity,
            language,
            is_veteran,
            health_insurance,
            highest_education_level,
        } = self;

        data.insert("age".to_string(), (*age).into());
        if let Some(gender) = &gender {
            gender.write(data);
        }
        if let Some(race) = &race {
            race.write(data);
        }
        data.insert("ethnicity", ethnicity.clone().into());
        if let Some(language) = &language {
            language.write(data);
        }
        data.insert("veteran".to_string(), (*is_veteran).into());
        data.insert(
            "insurance".to_string(),
            HealthInsurance::create_multiselect("insurance", health_insurance).into(),
        );
        data.insert(
            "education".to_string(),
            highest_education_level.clone().into(),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RedCapHealthOverview {
    /// Red Cap: height
    pub height: Option<i32>,
    /// Red Cap: health_conditions
    pub reported_health_conditions: Option<String>,
    /// Red Cap: allergies
    pub allergies: Option<String>,
    /// Red Cap: personal_cuff
    pub has_blood_pressure_cuff: Option<bool>,
    /// Red Cap: num_meds
    pub takes_more_than_5_medications: Option<bool>,
    /// Red Cap: mobility_devices
    pub mobility_devices: Option<Vec<MobilityDevice>>,
}
impl From<RedCapHealthOverview> for NewHealthOverview {
    fn from(value: RedCapHealthOverview) -> Self {
        let RedCapHealthOverview {
            height,
            reported_health_conditions,
            allergies,
            has_blood_pressure_cuff,
            takes_more_than_5_medications,
            mobility_devices,
        } = value;

        NewHealthOverview {
            height,
            reported_health_conditions,
            allergies,
            has_blood_pressure_cuff,
            takes_more_than_5_medications,
            mobility_devices,
        }
    }
}

impl From<HealthOverview> for RedCapHealthOverview {
    fn from(overview: HealthOverview) -> Self {
        Self {
            height: overview.height,
            reported_health_conditions: overview.reported_health_conditions,
            allergies: overview.allergies,
            has_blood_pressure_cuff: overview.has_blood_pressure_cuff,
            takes_more_than_5_medications: overview.takes_more_than_5_medications,
            mobility_devices: overview.mobility_devices,
        }
    }
}
impl RedCapHealthOverview {
    pub fn write<D: RedCapDataSet>(&self, data: &mut D) {
        let Self {
            height,
            reported_health_conditions,
            allergies,
            has_blood_pressure_cuff,
            takes_more_than_5_medications,
            mobility_devices,
        } = self;

        data.insert("height".to_string(), (*height).into());
        data.insert(
            "health_conditions".to_string(),
            reported_health_conditions.clone().into(),
        );
        data.insert("allergies".to_string(), allergies.clone().into());
        data.insert(
            "personal_cuff".to_string(),
            (*has_blood_pressure_cuff).into(),
        );
        data.insert(
            "num_meds".to_string(),
            (*takes_more_than_5_medications).into(),
        );
        if let Some(mobility_devices) = mobility_devices {
            data.insert_multi_select("mobility_devices", mobility_devices);
        }
    }
}

impl RedCapHealthOverview {
    pub async fn read<D: RedCapDataSet>(data: &D) -> Result<Self, RedCapConverterError> {
        let result = Self {
            height: data.get_number("height").map(|x| x as i32),
            reported_health_conditions: data.get_string("health_conditions"),
            allergies: data.get_string("allergies"),
            has_blood_pressure_cuff: data.get_bool("personal_cuff"),
            takes_more_than_5_medications: data.get_bool("num_meds"),
            mobility_devices: data.get_enum_multi_select("mobility_devices"),
        };

        Ok(result)
    }
}

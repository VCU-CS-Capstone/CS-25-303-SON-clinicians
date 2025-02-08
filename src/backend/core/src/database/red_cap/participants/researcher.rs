use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    database::queries::{ItemOrArray, NumberQuery},
    red_cap::{
        EducationLevel, Gender, HealthInsurance, PreferredLanguage, Programs, Race, SeenAtVCUHS,
        Status,
    },
};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ResearcherQueryResult {
    pub participant_id: i32,
    pub first_name: String,
    pub last_name: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(default)]
pub struct ResearcherQuery {
    /// Location to filter by
    pub location: Option<ItemOrArray<i32>>,
    /// Program to filter by
    pub program: Option<Programs>,
    pub vcuhs_patient_status: Option<SeenAtVCUHS>,
    /// Status to filter by
    ///
    /// By default it only returns active participants
    #[schema(default = "Active")]
    pub status: Option<Status>,

    pub gender: Option<Gender>,
    pub highest_level_of_education: Option<EducationLevel>,
    pub race: Option<Race>,
    pub language: Option<PreferredLanguage>,
    pub health_insurance: Option<HealthInsurance>,
    pub age: Option<NumberQuery<i16>>,
}
impl Default for ResearcherQuery {
    fn default() -> Self {
        Self {
            location: None,
            program: None,
            vcuhs_patient_status: None,
            status: Some(Status::Active),
            gender: None,
            highest_level_of_education: None,
            race: None,
            language: None,
            health_insurance: None,
            age: None,
        }
    }
}
impl ResearcherQuery {}

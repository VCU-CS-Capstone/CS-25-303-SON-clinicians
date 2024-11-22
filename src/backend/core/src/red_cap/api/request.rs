use std::fmt::Display;

use serde::Serialize;
use strum::{AsRefStr, Display, EnumString};

#[derive(Debug, EnumString, Display)]
pub enum Forms {
    #[strum(serialize = "participant_information")]
    ParticipantInformation,
    #[strum(serialize = "health_overview")]
    HealthOverview,
    #[strum(serialize = "medications")]
    Medications,
    #[strum(serialize = "wellness_goals")]
    WellnessGoals,
    #[strum(serialize = "case_note")]
    CaseNotes,
}
#[derive(Debug, EnumString, Display, Serialize, AsRefStr)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Json,
    Csv,
    Xml,
}
#[derive(Debug, EnumString, Display, Serialize, AsRefStr)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum FormatType {
    Flat,
    EAV,
}
#[derive(Debug)]
pub struct ConcatVec<T>(pub Vec<T>);
impl<T> From<Vec<T>> for ConcatVec<T> {
    fn from(vec: Vec<T>) -> Self {
        Self(vec)
    }
}
impl<T> Default for ConcatVec<T> {
    fn default() -> Self {
        Self(vec![])
    }
}
impl<T> Display for ConcatVec<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ConcatVec(vec) = self;
        write!(
            f,
            "{}",
            vec.iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}
#[derive(Debug, Default)]
pub struct ExportOptions {
    pub forms: Option<ConcatVec<Forms>>,
    pub records: Option<ConcatVec<usize>>,
    pub fields: Option<ConcatVec<String>>,
}

impl ExportOptions {
    pub fn with_forms(&mut self, forms: Vec<Forms>) {
        self.forms = Some(ConcatVec(forms));
    }
    pub fn with_records(&mut self, records: Vec<usize>) {
        self.records = Some(ConcatVec(records));
    }
    fn get_forms_mut(&mut self) -> &mut ConcatVec<Forms> {
        self.forms.get_or_insert_with(Default::default)
    }
    fn get_records_mut(&mut self) -> &mut ConcatVec<usize> {
        self.records.get_or_insert_with(Default::default)
    }

    pub fn add_form(&mut self, form: Forms) -> &mut Self {
        self.get_forms_mut().0.push(form);
        self
    }
    pub fn add_record(&mut self, record: usize) -> &mut Self {
        self.get_records_mut().0.push(record);
        self
    }
}

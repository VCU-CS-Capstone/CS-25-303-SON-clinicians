use crate::{database::prelude::*, red_cap::VisitType};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{CaseNote, CaseNoteColumn, CaseNoteType};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, FromRow)]
pub struct CaseNoteIDAndDate {
    /// Case Note ID
    pub id: i32,
    /// Date of the visit
    pub date_of_visit: NaiveDate,
}
impl TableQuery for CaseNoteIDAndDate {
    type Table = super::CaseNote;

    fn columns() -> Vec<super::CaseNoteColumn> {
        vec![
            super::CaseNoteColumn::Id,
            super::CaseNoteColumn::DateOfVisit,
        ]
    }
}

impl CaseNoteType for CaseNoteIDAndDate {
    fn get_id(&self) -> i32 {
        self.id
    }
}
/// A small struct to represent a case note for listing visits
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, FromRow)]
pub struct CaseNoteListItem {
    /// Case Note ID
    pub id: i32,
    /// Participant ID
    pub participant_id: i32,
    /// Location of the visit
    pub location: Option<i32>,
    /// Visit Type
    pub visit_type: Option<VisitType>,
    /// Date of the visit
    pub date_of_visit: NaiveDate,
}
impl TableQuery for CaseNoteListItem {
    type Table = CaseNote;
    fn columns() -> Vec<CaseNoteColumn> {
        vec![
            CaseNoteColumn::Id,
            CaseNoteColumn::ParticipantId,
            CaseNoteColumn::Location,
            CaseNoteColumn::VisitType,
            CaseNoteColumn::DateOfVisit,
        ]
    }
}

impl CaseNoteType for CaseNoteListItem {
    fn get_id(&self) -> i32 {
        self.id
    }
}

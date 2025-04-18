use axum::{
    extract::{Path, State},
    response::Response,
    routing::get,
};
use cs25_303_core::database::red_cap::{
    case_notes::{CaseNoteType, queries::CaseNoteListItem},
    participants::Participants,
};
use tracing::instrument;
use utoipa::OpenApi;

use crate::utils::response::ResponseBuilder;
use crate::{
    app::{SiteState, authentication::Authentication, error::InternalError},
    utils::ErrorReason,
};
#[derive(OpenApi)]
#[openapi(
    paths(get_all_case_notes_for_participant),
    components(schemas(CaseNoteListItem))
)]
pub struct CaseNoteAPI;

pub fn case_note_routes() -> axum::Router<SiteState> {
    axum::Router::new().route(
        "/{participant_id}/list/all",
        get(get_all_case_notes_for_participant),
    )
}
/// Returns a list of all case notes for a participant
#[utoipa::path(
    get,
    path = "/{participant_id}/list/all",
    params(
        ("participant_id" = i32, Path, description = "Participant ID")
    ),
    responses(
        (status = 200, description = "Participants Found", body = Vec<CaseNoteListItem>, content_type = "application/json"),
        (status = 404, description = "Participant Not Found"),
    ),
    security(
        ("session" = []),
    )
)]
#[instrument]
pub async fn get_all_case_notes_for_participant(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let case_notes = CaseNoteListItem::get_all_by_participant_id(id, &site.database).await?;
    // If the participant does not exist, return a 404
    if case_notes.is_empty() && !Participants::does_participant_id_exist(id, &site.database).await?
    {
        return Ok(ResponseBuilder::not_found()
            .extension(ErrorReason::from("Participant Not Found"))
            .empty());
    }
    Ok(ResponseBuilder::ok().json(&case_notes))
}

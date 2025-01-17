use axum::{
    extract::{Path, State},
    response::Response,
    routing::get,
};
use cs25_303_core::database::red_cap::participants::{ParticipantMedications, Participants};
use tracing::instrument;
use utoipa::OpenApi;

use crate::{
    app::{authentication::Authentication, error::InternalError, SiteState},
    utils::{not_found_response, ok_json_response},
};

#[derive(OpenApi)]
#[openapi(
    paths(get_participants_medications),
    components(schemas(ParticipantMedications))
)]
pub struct ParticipantMedicationsAPI;

pub fn participant_medications() -> axum::Router<SiteState> {
    axum::Router::new().route("/{participant_id}/all", get(get_participants_medications))
}
/// Returns all medications for a participant
#[utoipa::path(
    get,
    path = "/{participant_id}/all",
    params(
        ("participant_id", Path,  description = "Participant ID"),
    ),
    responses(
        (status = 200, description = "medications for participant", body = Vec<ParticipantMedications>),
        (status = 404, description = "Participant Not Found")
    ),
    security(
        ("session" = []),
        ("api_token" = []),
    )
)]
#[instrument(name = "api::participant::medications::get_all")]
pub async fn get_participants_medications(
    State(site): State<SiteState>,
    Path(participant_id): Path<i32>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let goals =
        ParticipantMedications::get_all_participant_medications(participant_id, &site.database)
            .await?;

    if goals.is_empty()
        && !Participants::does_participant_id_exist(participant_id, &site.database).await?
    {
        return not_found_response();
    }

    ok_json_response(goals)
}

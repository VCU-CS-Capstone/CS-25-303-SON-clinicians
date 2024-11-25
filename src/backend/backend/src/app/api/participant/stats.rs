use axum::{
    extract::{Path, State},
    response::Response,
    routing::get,
};
use cs25_303_core::database::red_cap::{
    case_notes::queries::{BloodPressureHistory, WeightHistory},
    participants::Participants,
};
use tracing::instrument;
use utoipa::OpenApi;

use crate::{
    app::{authentication::Authentication, error::InternalError, SiteState},
    utils::{not_found_response, ok_json_response},
};

#[derive(OpenApi)]
#[openapi(
    paths(participant_weight_history, bp_history),
    components(schemas(WeightHistory, BloodPressureHistory))
)]

pub struct ParticipantStatAPI;

pub fn stat_routes() -> axum::Router<SiteState> {
    axum::Router::new()
        .route("/weight/history/:id", get(participant_weight_history))
        .route("/bp/history/:id", get(bp_history))
}
#[utoipa::path(
    get,
    path = "/weight/history/{id}",
    params(
        ("id" = i32, Path, description = "Participant ID")
    ),
    responses(
        (status = 200, description = "Participant Weight History", body = Vec<WeightHistory>),
        (status = 404, description = "Participant Not Found"),
    ),
    security(
        ("session" = []),
        ("api_token" = []),
    )
)]
#[instrument]
pub async fn participant_weight_history(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let weights = WeightHistory::find_all_for_participant(id, &site.database).await?;
    // If the participant does not exist, return a 404
    if weights.is_empty() && !Participants::does_participant_id_exist(id, &site.database).await? {
        return not_found_response();
    }
    ok_json_response(weights)
}

#[utoipa::path(
    get,
    path = "/bp/history/{id}",
    params(
        ("id" = i32, Path, description = "Participant ID")
    ),
    responses(
        (status = 200, description = "Blood Pressure History", body = Vec<BloodPressureHistory>),
        (status = 404, description = "Participant Not Found"),
    ),
    security(
        ("session" = []),
        ("api_token" = []),
    )
)]
#[instrument]
pub async fn bp_history(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let readings = BloodPressureHistory::find_all_for_participant(id, &site.database).await?;
    // If the participant does not exist, return a 404
    if readings.is_empty() && !Participants::does_participant_id_exist(id, &site.database).await? {
        return not_found_response();
    }
    ok_json_response(readings)
}

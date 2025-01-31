use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::get,
};
use cs25_303_core::database::{
    red_cap::{
        case_notes::queries::{BloodPressureHistory, BloodPressureReadings, WeightHistory},
        participants::Participants,
    },
    tools::{PageParams, PaginatedResponse},
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
    components(schemas(WeightHistory, BloodPressureHistory, BloodPressureReadings))
)]
pub struct ParticipantStatAPI;

pub fn stat_routes() -> axum::Router<SiteState> {
    axum::Router::new()
        .route(
            "/weight/history/{participant_id}",
            get(participant_weight_history),
        )
        .route("/bp/history/{participant_id}", get(bp_history))
}
#[utoipa::path(
    get,
    path = "/weight/history/{participant_id}",
    summary= "Fetch the weight history for a participant",

    params(
        ("participant_id" = i32, Path, description = "Participant ID"),
        PageParams,
    ),
    responses(
        (status = 200, description = "Participant Weight History", body = PaginatedResponse<WeightHistory>),
        (status = 404, description = "Participant Not Found"),
    ),
    security(
        ("session" = []),
    )
)]
#[instrument]
pub async fn participant_weight_history(
    State(site): State<SiteState>,
    Path(participant_id): Path<i32>,
    Query(page): Query<PageParams>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let weights =
        WeightHistory::find_all_for_participant(participant_id, page, &site.database).await?;
    // If the participant does not exist, return a 404
    if weights.is_empty()
        && !Participants::does_participant_id_exist(participant_id, &site.database).await?
    {
        return not_found_response();
    }
    ok_json_response(weights)
}

#[utoipa::path(
    get,
    path = "/bp/history/{participant_id}",
    params(
        ("participant_id" = i32, Path, description = "Participant ID"),
        PageParams,
    ),
    summary="Fetch the blood pressure history for a participant",
    responses(
        (status = 200, description = "Blood Pressure History", body = PaginatedResponse<BloodPressureHistory>),
        (status = 404, description = "Participant Not Found"),
    ),
    security(
        ("session" = []),
    )
)]
#[instrument]
pub async fn bp_history(
    State(site): State<SiteState>,
    Path(participant_id): Path<i32>,
    Query(page): Query<PageParams>,

    auth: Authentication,
) -> Result<Response, InternalError> {
    let readings =
        BloodPressureHistory::find_all_for_participant(participant_id, page, &site.database)
            .await?;
    // If the participant does not exist, return a 404
    if readings.is_empty()
        && !Participants::does_participant_id_exist(participant_id, &site.database).await?
    {
        return not_found_response();
    }
    ok_json_response(readings)
}

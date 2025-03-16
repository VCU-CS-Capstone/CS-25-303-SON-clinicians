use crate::utils::{ErrorReason, response::ResponseBuilder};
use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::get,
};
use cs25_303_core::database::{
    CSPageParams, PaginatedResponse,
    red_cap::{
        participants::Participants,
        participants::stats::{
            BloodGlucoseHistory, BloodPressureHistory, BloodPressureReadings, WeightHistory,
        },
    },
};
use serde::Deserialize;
use tracing::instrument;
use utoipa::{IntoParams, OpenApi};

use crate::app::{SiteState, authentication::Authentication, error::InternalError};

#[derive(OpenApi)]
#[openapi(
    paths(participant_weight_history, bp_history, glucose_history),
    components(schemas(
        WeightHistory, BloodPressureHistory, BloodPressureReadings,
        PaginatedResponse<WeightHistory>, PaginatedResponse<BloodPressureHistory>, PaginatedResponse<BloodGlucoseHistory>))
)]
pub struct ParticipantStatAPI;

pub fn stat_routes() -> axum::Router<SiteState> {
    axum::Router::new()
        .route(
            "/weight/history/{participant_id}",
            get(participant_weight_history),
        )
        .route("/bp/history/{participant_id}", get(bp_history))
        .route("/glucose/history/{participant_id}", get(glucose_history))
}
#[derive(IntoParams, Deserialize)]
#[into_params(parameter_in = Query)]
pub struct CalculateBMI {
    /// Rather or not you want the calculated BMI
    #[serde(default)]
    #[into_params(default = false)]
    pub calculate_bmi: bool,
}
#[utoipa::path(
    get,
    path = "/weight/history/{participant_id}",
    summary= "Fetch the weight history for a participant",

    params(
        ("participant_id" = i32, Path, description = "Participant ID"),
        CalculateBMI,
        CSPageParams,
    ),
    responses(
        (status = 200, description = "Participant Weight History", body = PaginatedResponse<WeightHistory>, content_type = "application/json"),
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
    Query(page): Query<CSPageParams>,
    Query(CalculateBMI { calculate_bmi }): Query<CalculateBMI>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let weights = WeightHistory::find_all_for_participant(
        participant_id,
        calculate_bmi,
        page,
        &site.database,
    )
    .await?;
    // If the participant does not exist, return a 404
    if weights.is_empty()
        && !Participants::does_participant_id_exist(participant_id, &site.database).await?
    {
        return Ok(ResponseBuilder::not_found()
            .extension(ErrorReason::from("Participant Not Found"))
            .empty());
    }
    Ok(ResponseBuilder::ok().json(&weights))
}

#[utoipa::path(
    get,
    path = "/bp/history/{participant_id}",
    params(
        ("participant_id" = i32, Path, description = "Participant ID"),
        CSPageParams,
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
    Query(page): Query<CSPageParams>,

    auth: Authentication,
) -> Result<Response, InternalError> {
    let readings =
        BloodPressureHistory::find_all_for_participant(participant_id, page, &site.database)
            .await?;
    // If the participant does not exist, return a 404
    if readings.is_empty()
        && !Participants::does_participant_id_exist(participant_id, &site.database).await?
    {
        return Ok(ResponseBuilder::not_found()
            .extension(ErrorReason::from("Participant Not Found"))
            .empty());
    }
    Ok(ResponseBuilder::ok().json(&readings))
}
#[utoipa::path(
    get,
    path = "/glucose/history/{participant_id}",
    params(
        ("participant_id" = i32, Path, description = "Participant ID"),
        CSPageParams,
    ),
    summary="Fetch the blood glucose history for a participant",
    responses(
        (status = 200, description = "Blood glucose History", body = PaginatedResponse<BloodGlucoseHistory>),
        (status = 404, description = "Participant Not Found"),
    ),
    security(
        ("session" = []),
    )
)]
#[instrument]
pub async fn glucose_history(
    State(site): State<SiteState>,
    Path(participant_id): Path<i32>,
    Query(page): Query<CSPageParams>,

    auth: Authentication,
) -> Result<Response, InternalError> {
    let readings =
        BloodGlucoseHistory::find_all_for_participant(participant_id, page, &site.database).await?;
    // If the participant does not exist, return a 404
    if readings.is_empty()
        && !Participants::does_participant_id_exist(participant_id, &site.database).await?
    {
        return Ok(ResponseBuilder::not_found()
            .extension(ErrorReason::from("Participant Not Found"))
            .empty());
    }
    Ok(ResponseBuilder::ok().json(&readings))
}

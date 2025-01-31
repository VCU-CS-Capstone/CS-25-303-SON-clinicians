use axum::{
    extract::{Path, State},
    response::Response,
    routing::get,
};
use cs25_303_core::database::red_cap::participants::{
    goals::{ParticipantGoals, ParticipantGoalsSteps},
    Participants,
};
use tracing::instrument;
use utoipa::OpenApi;

use crate::app::{
    authentication::Authentication, error::InternalError,
    utils::response::builder::ResponseBuilder, SiteState,
};

#[derive(OpenApi)]
#[openapi(
    paths(get_participants_goals, get_steps_for_goal, get_steps_without_goal),
    components(schemas(ParticipantGoals))
)]
pub struct ParticipantGoalsAPI;

pub fn participant_goals() -> axum::Router<SiteState> {
    axum::Router::new()
        .route("/{participant_id}/all", get(get_participants_goals))
        .route("/{goal_id}/steps", get(get_steps_for_goal))
        .route(
            "/{participant_id}/steps/without_goal",
            get(get_steps_without_goal),
        )
}
/// Returns all goals for a participant
#[utoipa::path(
    get,
    path = "/{participant_id}/all",
    params(
        ("participant_id", Path,  description = "Participant ID"),
    ),
    responses(
        (status = 200, description = "goals for participant", body = Vec<ParticipantGoals>),
        (status = 404, description = "Participant Not Found")
    ),
    security(
        ("session" = []),

    )
)]
#[instrument]
pub async fn get_participants_goals(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let goals = ParticipantGoals::get_all_participant_goals(id, &site.database).await?;

    if goals.is_empty() && !Participants::does_participant_id_exist(id, &site.database).await? {
        return Ok(ResponseBuilder::not_found().empty());
    }

    Ok(ResponseBuilder::ok().json(&goals))
}

/// Returns all steps for a goal
#[utoipa::path(
    get,
    path = "/{goal_id}/steps",
    params(
        ("goal_id", Path,  description = "Goal ID"),
    ),
    responses(
        (status = 200, description = "Steps for Goal", body = Vec<ParticipantGoalsSteps>),
        (status = 404, description = "Goal Not Found")
    ),
    security(
        ("session" = []),

    )
)]
#[instrument]
pub async fn get_steps_for_goal(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let goals = ParticipantGoalsSteps::get_all_steps_for_goal(id, &site.database).await?;

    if goals.is_empty() && !Participants::does_participant_id_exist(id, &site.database).await? {
        return Ok(ResponseBuilder::not_found().empty());
    }

    Ok(ResponseBuilder::ok().json(&goals))
}
/// Returns Steps that do not have a goal
///
/// These should be flagged for review
#[utoipa::path(
    get,
    path = "/{id}/steps/without_goal",
    params(
        ("id", Path,  description = "Participant ID"),
    ),
    responses(
        (status = 200, description = "Steps without goal", body = Vec<ParticipantGoalsSteps>),
        (status = 404, description = "Participant Not Found")
    ),
    security(
        ("session" = []),

    )
)]
#[instrument]
pub async fn get_steps_without_goal(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let goals =
        ParticipantGoalsSteps::get_goaless_steps_for_participant(id, &site.database).await?;

    if goals.is_empty() && !Participants::does_participant_id_exist(id, &site.database).await? {
        return Ok(ResponseBuilder::not_found().empty());
    }

    Ok(ResponseBuilder::ok().json(&goals))
}

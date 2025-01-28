use axum::{
    extract::{Path, State},
    response::Response,
    routing::get,
};
use cs25_303_core::{
    database::red_cap::{Locations, RedCapLocationConnectionRules},
    red_cap::Programs,
};
use tracing::instrument;
use utoipa::OpenApi;

use crate::{
    app::{
        authentication::Authentication, error::InternalError,
        utils::response::builder::ResponseBuilder, SiteState,
    },
    utils::ok_json_response,
};

#[derive(OpenApi)]
#[openapi(
    paths(all_locations, get_location_by_id, get_locations_by_program),
    components(schemas(Locations, RedCapLocationConnectionRules))
)]
pub struct LocationsAPI;

pub fn location_routes() -> axum::Router<SiteState> {
    axum::Router::new()
        .route("/all", get(all_locations))
        .route("/{id}", get(get_location_by_id))
        .route("/program/{program}", get(get_locations_by_program))
}
#[utoipa::path(
    get,
    path = "/all",
    responses(
        (status = 200, description = "All locations in the system", body = Vec<Locations>)
    ),
    security(
        ("session" = []),
        ("api_token" = []),
    )
)]
#[instrument]
pub async fn all_locations(
    State(site): State<SiteState>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let locations = Locations::get_all(&site.database).await?;
    ok_json_response(locations)
}
#[utoipa::path(
    get,
    path = "/{id}",
    params(
        ("id" = i32, description = "The ID of the location to retrieve")
    ),
    responses(
        (status = 200, description = "The location that was requested", body =Locations),
        (status = 404, description = "The location was not found")
    ),
    security(
        ("session" = []),
        ("api_token" = []),
    )
)]
#[instrument]
pub async fn get_location_by_id(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let Some(location) = Locations::find_by_id(id, &site.database).await? else {
        return Ok(ResponseBuilder::not_found().empty());
    };

    Ok(ResponseBuilder::ok().json(&location))
}

#[utoipa::path(
    get,
    path = "/program/{program}",
    params(
        ("program" = Programs, description = "The program you want to get locations for")
    ),
    responses(
        (status = 200, description = "All locations in the requested program", body =Vec<Locations>)
    ),
    security(
        ("session" = []),
        ("api_token" = []),
    )
)]
#[instrument]
pub async fn get_locations_by_program(
    State(site): State<SiteState>,
    Path(program): Path<Programs>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let locations = Locations::find_all_in_program(program, &site.database).await?;

    Ok(ResponseBuilder::ok().json(&locations))
}

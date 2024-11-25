use axum::{extract::State, response::Response, routing::get};
use cs25_303_core::database::red_cap::{Locations, RedCapLocationConnectionRules};
use tracing::instrument;
use utoipa::OpenApi;

use crate::{
    app::{authentication::Authentication, error::InternalError, SiteState},
    utils::ok_json_response,
};

#[derive(OpenApi)]
#[openapi(
    paths(all_locations),
    components(schemas(Locations, RedCapLocationConnectionRules))
)]
pub struct LocationsAPI;

pub fn location_routes() -> axum::Router<SiteState> {
    axum::Router::new().route("/all", get(all_locations))
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

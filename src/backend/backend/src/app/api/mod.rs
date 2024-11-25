use axum::{extract::State, Json};
use serde::Serialize;
use tracing::instrument;
use utoipa::ToSchema;
pub mod admin;
pub mod location;
pub mod participant;
pub mod questions;
pub mod user;
use super::SiteState;
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Instance {
    pub version: &'static str,
}
impl Instance {
    pub fn new(_state: SiteState) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}
pub fn api_routes() -> axum::Router<SiteState> {
    axum::Router::new()
        .route("/info", axum::routing::get(info))
        .nest("/user", user::user_routes())
        .nest("/participant", participant::participant_routes())
        .nest("/location", location::location_routes())
        .nest("/admin", admin::admin_routes())
}
#[utoipa::path(
    get,
    path = "/api/info",
    responses(
        (status = 200, description = "information about the Site", body = Instance)
    ),
    security(
        (),
        ("session" = []),
        ("api_token" = []),
    )
)]
#[instrument]
pub async fn info(State(site): State<SiteState>) -> Json<Instance> {
    Json(Instance::new(site))
}

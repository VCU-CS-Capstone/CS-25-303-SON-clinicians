use axum::{extract::State, Json};
use serde::Serialize;
use tracing::instrument;
use utoipa::ToSchema;
pub mod user;
use super::{SiteState, WrappedSiteState};
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
        .merge(user::user_routes())
}
#[utoipa::path(
    get,
    path = "/api/info",
    responses(
        (status = 200, description = "information about the Site", body = Instance)
    )
)]
#[instrument]
pub async fn info(State(site): WrappedSiteState) -> Json<Instance> {
    Json(Instance::new(site))
}

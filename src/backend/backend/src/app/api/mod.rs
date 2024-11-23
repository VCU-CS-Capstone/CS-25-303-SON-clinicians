use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use utoipa::ToSchema;
pub mod location;
pub mod participant;
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
        .nest("/participant", participant::participant_routes())
        .nest("/location", location::location_routes())
}
#[utoipa::path(
    get,
    path = "/api/info",
    responses(
        (status = 200, description = "information about the Site", body = Instance)
    )
)]
#[instrument]
pub async fn info(State(site): State<SiteState>) -> Json<Instance> {
    Json(Instance::new(site))
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    /// The current page number
    pub page: i32,
    /// The number of items per page
    pub page_size: i32,
    /// The total number of items
    pub total: i32,
    /// The data for the current page
    pub data: Vec<T>,
}

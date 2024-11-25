use utoipa::OpenApi;

use crate::app::SiteState;

pub mod user;
#[derive(OpenApi)]
#[openapi(paths(), components(schemas()),
nest(
    (path = "/user", api = user::AdminUserAPI, tags=["UserAdmin"]),
))]
pub struct AdminAPI;

pub fn admin_routes() -> axum::Router<SiteState> {
    axum::Router::new().nest("/user", user::admin_user_routes())
}

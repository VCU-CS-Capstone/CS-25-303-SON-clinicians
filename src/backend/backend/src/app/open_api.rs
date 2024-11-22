use super::api::{self, user::UserAPI};
use axum::{
    response::{IntoResponse, Response},
    Json, Router,
};

use utoipa::OpenApi;
#[derive(OpenApi)]
#[openapi(
    modifiers(),
    nest(
        (path = "/api/user", api = UserAPI, tags=["user"]),
    ),
    paths(api::info),
    components(schemas(api::Instance)),
    tags()
)]
pub struct ApiDoc;

#[cfg(feature = "utoipa-scalar")]
pub fn build_router<S>() -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    use utoipa_scalar::{Scalar, Servable};

    Router::new()
        .route("/open-api-doc-raw", get(api_docs))
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
}
#[cfg(not(feature = "utoipa-scalar"))]
pub fn build_router<S>() -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    use axum::routing::get;

    Router::new().route("/open-api-doc-raw", get(api_docs))
}
async fn api_docs() -> Response {
    Json(ApiDoc::openapi()).into_response()
}

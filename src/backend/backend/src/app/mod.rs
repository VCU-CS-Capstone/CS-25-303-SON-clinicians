use std::sync::Arc;
pub mod error;
use anyhow::Context;
use authentication::session::SessionManager;
use axum::routing::Router;

mod state;
use http::HeaderName;
use sqlx::postgres::PgConnectOptions;
pub use state::*;
pub mod authentication;
use tower_http::request_id::PropagateRequestIdLayer;
mod api;
mod open_api;
mod web;
use crate::config::FullConfig;
const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

pub(super) async fn start_web_server(config: FullConfig) -> anyhow::Result<()> {
    let FullConfig {
        web_server,
        database,
        tls,
        mode,
        log,
        auth,
    } = config;
    // Start the logger
    crate::logging::init(log, mode)?;
    // Connect to database
    let pg_options: PgConnectOptions = database.try_into()?;
    let database = cs25_303_core::database::connect(pg_options, true).await?;
    let session = SessionManager::new(None, mode)?;
    // Create the website state
    let inner = SiteStateInner::new(auth, session);
    let website = SiteState {
        inner: Arc::new(inner),
        database,
    };
    website.start().await;
    let mut router = Router::new()
        .nest("/api", api::api_routes())
        .with_state(website.clone());
    if web_server.open_api_routes {
        router = router.merge(open_api::build_router())
    }
    router = router
        .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER))
        .layer(authentication::api_middleware::AuthenticationLayer(
            website.clone(),
        ));
    // Start the web server
    let tls = tls
        .map(|tls| {
            web::rustls_server_config(tls.private_key, tls.certificate_chain)
                .context("Failed to create TLS configuration")
        })
        .transpose()?;
    if let Some(_tls) = tls {
        todo!("Start the web server with TLS");
    } else {
        web::start(web_server.bind_address, web_server.port, router, website).await?;
    }
    Ok(())
}

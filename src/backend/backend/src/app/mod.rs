use std::sync::Arc;
pub mod error;
use anyhow::Context;
use authentication::{api_middleware::AuthenticationLayer, session::SessionManager};
use axum::routing::Router;
pub mod request_logging;
mod state;
pub mod utils;
use request_logging::AppTracingLayer;
use sqlx::postgres::PgConnectOptions;
pub use state::*;
pub mod authentication;

use tracing::info;
mod api;
mod open_api;
mod web;
use crate::config::FullConfig;

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
    crate::logging::init(log)?;
    info!("Starting web server");

    // Connect to database
    let pg_options: PgConnectOptions = database.try_into()?;
    let database = cs25_303_core::database::connect(pg_options, true).await?;
    info!("Connected to database");
    let session = SessionManager::new(None, mode)?;
    // Create the website state
    let inner = SiteStateInner::new(auth, session);
    let website = SiteState {
        inner: Arc::new(inner),
        database,
    };
    website.start().await;
    info!("Website Configured");
    let router = Router::new()
        .nest("/api", api::api_routes())
        .merge(open_api::open_api_router(
            web_server.open_api_routes,
            web_server.scalar,
        ))
        .layer(AuthenticationLayer(website.clone()))
        .layer(AppTracingLayer(website.clone()))
        .with_state(website.clone());

    info!("Router Configured");
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
        info!("Starting web server without TLS");
        web::start(web_server.bind_address, web_server.port, router, website).await?;
    }
    Ok(())
}

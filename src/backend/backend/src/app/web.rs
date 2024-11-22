use std::{fs::File, io::BufReader, net::SocketAddr, path::Path, sync::Arc};

use anyhow::Context;
use axum::Router;
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio::{net::TcpListener, signal};
use tracing::info;

use super::SiteState;

pub fn rustls_server_config(
    key: impl AsRef<Path>,
    cert: impl AsRef<Path>,
) -> anyhow::Result<Arc<ServerConfig>> {
    let mut key_reader = BufReader::new(File::open(key).context("Failed to open key file")?);
    let mut cert_reader = BufReader::new(File::open(cert).context("Failed to open cert file")?);

    let cert_chain = certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;
    let mut keys = pkcs8_private_keys(&mut key_reader).collect::<Result<Vec<_>, _>>()?;

    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            cert_chain,
            rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0)),
        )?;

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(Arc::new(config))
}

pub async fn start(
    bind_address: String,
    port: u16,
    router: Router,
    site: SiteState,
) -> anyhow::Result<()> {
    let bind = format!("{}:{}", bind_address, port);
    let listener = TcpListener::bind(bind).await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal(site))
    .await?;
    Ok(())
}

async fn shutdown_signal(website: SiteState) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("Shutting down");
    website.close().await;
}

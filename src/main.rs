mod domain;
mod http;
mod hub;

use domain::AppConfig;
use http::{AppState, router};
use hub::HubHandle;
use std::error::Error;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = AppConfig::from_env()?;
    let (hub, actor) = HubHandle::new(256);
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

    tokio::spawn(actor.run());

    tracing_subscriber::registry()
        .with(EnvFilter::new("info,tower_http=info"))
        .with(fmt::layer().with_writer(non_blocking))
        .init();

    let app = router(AppState { hub });

    let listener = tokio::net::TcpListener::bind(config.bind_addr()?).await?;

    tracing::info!(local_addr = %listener.local_addr()?, "http server bound");

    axum::serve(listener, app).await?;

    Ok(())
}

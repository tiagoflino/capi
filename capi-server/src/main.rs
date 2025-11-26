use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = capi_core::Config::load()?;
    let db = Arc::new(capi_core::Database::open(config.database_path())?);

    let registry = Arc::new(capi_core::Registry::new(db.clone()));

    let state = capi_core::AppState {
        registry,
    };

    let app = capi_core::create_router(state);
    let bind_addr = config.bind_address();
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    println!("Server running on {}", config.server_url());

    axum::serve(listener, app).await?;

    Ok(())
}

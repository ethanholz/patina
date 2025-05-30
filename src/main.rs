use axum::Router;
use log::info;
use models::state::AppState;
use std::{env, fs::File, path::Path};
use tokio::signal;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

mod api;
mod db;
mod models;

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("byos_rust=info,tower_http=warn"))
                .unwrap(),
        )
        .init();

    let db_path = Path::new("./database.db");
    // Create a new DB if it doesn't exist
    if !db_path.exists() {
        File::create(db_path)?;
    }
    let pool = db::initialize(db_path.to_str().unwrap()).await;
    info!("DB created");
    let pool = match pool {
        Ok(pool) => pool,
        Err(err) => panic!("{}", err),
    };
    let port = env::var("PORT").unwrap_or("3000".to_string());
    let base_url = env::var("BASE_URL").unwrap_or(format!("http://localhost:{}", port));
    let bind_url = format!("0.0.0.0:{}", port);

    let state = AppState {
        db: pool,
        base_url: base_url.clone(),
    };

    let app = Router::new()
        .nest("/api", api::router())
        .nest_service("/storage/images", ServeDir::new("assets"))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(&bind_url).await?;
    info!("🚀 TRMNL BYOS Server running on {}", &base_url);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    info!("byos-rust Shutdown!");
    Ok(())
}

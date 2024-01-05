use std::{error::Error, net::SocketAddr};

use axum::Router;
use clap::Parser;
use sqlx::{PgPool, Pool, Postgres};
use tokio::signal;

mod config;
mod handlers;

fn app(pool: Pool<Postgres>, geocoding_api_key: String) -> Router {
    handlers::router(pool, geocoding_api_key)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load environment variable from .env file
    dotenv::dotenv().ok();

    // initialize the tracing
    tracing_subscriber::fmt().init();

    // load the env variable into config struct
    let config = config::Config::parse();
    tracing::info!("config .env {:?}", config);

    // initialize the database pool
    let pool = PgPool::connect(&config.database_url).await.unwrap();

    // bind an address from the env port
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    // start the server
    // run our app with hyper, listening globally on port PORT
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", addr);
    axum::serve(
        listener,
        app(pool, config.geocoding_api_key).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
    Ok(())
}
async fn shutdown_signal() {
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
}

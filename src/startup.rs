use axum::{
    extract::{connect_info::IntoMakeServiceWithConnectInfo, ConnectInfo},
    middleware::AddExtension,
    serve::Serve,
    Router,
};
use sqlx::{PgPool, Pool, Postgres};
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::{config, handlers};

fn app(pool: Pool<Postgres>, geocoding_api_key: String) -> Router {
    handlers::router(pool, geocoding_api_key)
}

type Server = Serve<
    IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
    AddExtension<Router, ConnectInfo<SocketAddr>>,
>;

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    config: config::Config,
) -> Result<Server, std::io::Error> {
    let server = axum::serve(
        listener,
        app(db_pool, config.geocoding_api_key).into_make_service_with_connect_info::<SocketAddr>(),
    );
    Ok(server)
}

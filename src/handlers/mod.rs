use sqlx::{Pool, Postgres};

pub mod day0;
pub mod day1;
pub mod day11;
pub mod day12;
pub mod day13;
pub mod day14;
pub mod day15;
pub mod day18;
pub mod day19;
pub mod day20;
pub mod day21;
pub mod day22;
pub mod day4;
pub mod day5;
pub mod day6;
pub mod day7;
pub mod day8;

pub fn router(pool: Pool<Postgres>, geocoding_api_key: String) -> axum::Router {
    axum::Router::new()
        .nest("/", day0::router())
        .nest("/", day1::router())
        .nest("/", day4::router())
        .nest("/", day5::router())
        .nest("/", day6::router())
        .nest("/", day7::router())
        .nest("/", day8::router())
        .nest("/", day11::router())
        .nest("/", day12::router())
        .nest("/", day13::router(pool.clone()))
        .nest("/", day14::router())
        .nest("/", day15::router())
        .nest("/", day18::router(pool))
        .nest("/", day19::router())
        .nest("/", day20::router())
        .nest("/", day21::router(geocoding_api_key))
        .nest("/", day22::router())
}

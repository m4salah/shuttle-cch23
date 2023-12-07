mod day0;
mod day1;
mod day4;
mod day6;

pub use day4::{ContestReindeer, ContestResult};
pub use day6::ElfOnShelfResult;

pub fn router() -> axum::Router {
    axum::Router::new()
        .nest("/", day0::router())
        .nest("/", day1::router())
        // Days 2 and 3 are omitted due to being weekends
        .nest("/", day4::router())
        .nest("/", day6::router())
}

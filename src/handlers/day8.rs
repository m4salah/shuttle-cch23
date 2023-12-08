use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct PokeWeight {
    weight: u32,
}

impl PokeWeight {
    // Extract the weight from the pokemone in kg
    fn extract_weight_kg(&self) -> f64 {
        self.weight as f64 / 10.0
    }
}

async fn poke_weight(Path(pokedex): Path<u32>) -> impl IntoResponse {
    let poke_weight = reqwest::get(format!("https://pokeapi.co/api/v2/pokemon/{pokedex}"))
        .await
        .unwrap()
        .json::<PokeWeight>()
        .await
        .unwrap();

    format!("{}", poke_weight.extract_weight_kg())
}

async fn poke_drop(Path(pokedex): Path<u32>) -> impl IntoResponse {
    let poke_weight = reqwest::get(format!("https://pokeapi.co/api/v2/pokemon/{pokedex}"))
        .await
        .unwrap()
        .json::<PokeWeight>()
        .await
        .unwrap();

    println!(
        "{}",
        poke_weight.extract_weight_kg() * (9.825f64 * 10.0 * 2.0).sqrt()
    );
    format!(
        "{}",
        poke_weight.extract_weight_kg() * (9.825f64 * 10.0 * 2.0).sqrt()
    )
}

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/8", get(|| async { StatusCode::OK }))
        .route("/8/weight/:pokedex", get(poke_weight))
        .route("/8/drop/:pokedex", get(poke_drop))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;

    #[tokio::test]
    async fn day8_health() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/8").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn day8_poke_weight() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/8/weight/25").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await, "6");
    }

    #[tokio::test]
    async fn day8_poke_drop() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/8/drop/25").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert!((res.text().await.parse::<f64>().unwrap() - 84.10707461325713).abs() <= 0.001);
    }
}

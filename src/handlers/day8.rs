use std::error::Error;

use axum::{extract::Path, http::StatusCode, routing::get};
use serde::Deserialize;

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/8/weight/:pokedex", get(poke_weight))
        .route("/8/drop/:pokedex", get(poke_drop))
        .route("/8/health", get(|| async { StatusCode::OK }))
}

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

async fn fetch_poke(poke_id: u32) -> Result<PokeWeight, Box<dyn Error>> {
    Ok(
        reqwest::get(format!("https://pokeapi.co/api/v2/pokemon/{poke_id}"))
            .await?
            .json::<PokeWeight>()
            .await?,
    )
}

async fn poke_weight(Path(pokedex): Path<u32>) -> Result<String, StatusCode> {
    let poke_weight = fetch_poke(pokedex).await.map_err(|e| {
        eprintln!("ERR: error while fetch poke {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(format!("{}", poke_weight.extract_weight_kg()))
}

async fn poke_drop(Path(pokedex): Path<u32>) -> Result<String, StatusCode> {
    let poke_weight = fetch_poke(pokedex).await.map_err(|e| {
        eprintln!("ERR: error while fetch poke {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(format!(
        "{}",
        poke_weight.extract_weight_kg() * (9.825f64 * 10.0 * 2.0).sqrt()
    ))
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
        let res = client.get("/8/health").send().await;
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

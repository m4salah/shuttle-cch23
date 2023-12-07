use axum::Router;

mod handlers;
#[allow(dead_code)]
fn app() -> Router {
    handlers::router()
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    Ok(app().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;
    use serde_json::json;

    #[tokio::test]
    async fn hello_world() {
        let app = app();

        let client = TestClient::new(app);
        let res = client.get("/").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await, "Hello, World!");
    }

    #[tokio::test]
    async fn internal_server_error() {
        let app = app();

        let client = TestClient::new(app);
        let res = client.get("/-1/error").send().await;
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(res.text().await, "");
    }

    #[tokio::test]
    async fn num1_xor_num2_pow_3() {
        let app = app();

        let client = TestClient::new(app);
        let res = client.get("/1/3/5").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = ((3 ^ 5) as i32).pow(3);
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn one_packet_ids() {
        let app = app();

        let client = TestClient::new(app);
        let res = client.get("/1/10").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = 1000;
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn multi_packet_ids() {
        let app = app();

        let client = TestClient::new(app);
        let res = client.get("/1/4/5/8/10").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = 27;
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn multi_packet_ids_more_than_20_ids() {
        let app = app();

        let client = TestClient::new(app);
        let res = client
            .get("/1/1/2/3/4/5/6/7/8/9/10/11/12/13/14/15/16/17/18/19/20/21")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let expected = "packet ids must be between 1 and 20 inclusive packets in a sled";
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn sum_of_strength() {
        let app = app();

        let client = TestClient::new(app);
        let res = client
            .post("/4/strength")
            .body(
                json!([
                  { "name": "Dasher", "strength": 5 },
                  { "name": "Dancer", "strength": 6 },
                  { "name": "Prancer", "strength": 4 },
                  { "name": "Vixen", "strength": 7 }
                ])
                .to_string(),
            )
            .header("Content-Type", "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = "22";
        assert_eq!(res.text().await, format!("{expected}"));
    }

    #[tokio::test]
    async fn valid_contest() {
        let app = app();

        let client = TestClient::new(app);
        let res = client
            .post("/4/contest")
            .body(
                json!([
                {
                      "name": "Dasher",
                      "strength": 5,
                      "speed": 50.4,
                      "height": 80,
                      "antler_width": 36,
                      "snow_magic_power": 9001,
                      "favorite_food": "hay",
                      "cAnD13s_3ATeN-yesT3rdAy": 2
                    },
                    {
                      "name": "Dancer",
                      "strength": 6,
                      "speed": 48.2,
                      "height": 65,
                      "antler_width": 37,
                      "snow_magic_power": 4004,
                      "favorite_food": "grass",
                      "cAnD13s_3ATeN-yesT3rdAy": 5
                    }
                                ])
                .to_string(),
            )
            .header("Content-Type", "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = handlers::ContestResult {
            fastest: "Speeding past the finish line with a strength of 6 is Dancer".to_string(),
            tallest: "Dasher is standing tall with his 36 cm wide antlers".to_string(),
            magician: "Dasher could blast you away with a snow magic power of 9001".to_string(),
            consumer: "Dancer ate lots of candies, but also some grass".to_string(),
        };
        assert_eq!(res.json::<handlers::ContestResult>().await, expected);
    }

    #[tokio::test]
    async fn invalid_contest() {
        let app = app();

        let client = TestClient::new(app);
        let res = client
            .post("/4/contest")
            .body(json!([]).to_string())
            .header("Content-Type", "application/json")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn elf() {
        let app = app();

        let client = TestClient::new(app);
        let res = client
            .post("/6")
            .body(
                "The mischievous elf peeked out from behind the toy workshop,
      and another elf joined in the festive dance.
      Look, there is also an elf on that shelf!",
            )
            .header("Content-Type", "text/plain")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = handlers::ElfOnShelfResult {
            elf: 4,
            ..Default::default()
        };
        assert_eq!(
            res.json::<handlers::ElfOnShelfResult>().await.elf,
            expected.elf
        );
    }
    #[tokio::test]
    async fn elf_on_shelf() {
        let app = app();

        let client = TestClient::new(app);
        let res = client
            .post("/6")
            .body("there is an elf on a shelf on an elf. there is also another shelf in Belfast.")
            .header("Content-Type", "text/plain")
            .send()
            .await;
        assert_eq!(res.status(), StatusCode::OK);
        let expected = handlers::ElfOnShelfResult {
            elf: 5,
            shelf_with_no_elf: 1,
            elf_on_shelf: 1,
        };
        assert_eq!(res.json::<handlers::ElfOnShelfResult>().await, expected);
    }
}

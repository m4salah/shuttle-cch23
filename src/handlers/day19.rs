use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use futures::{stream::StreamExt, SinkExt};
use serde_json::json;
use tokio::sync::broadcast;

// Our shared state
struct AppState {
    /// Keys are the name of the channel
    views: Arc<AtomicUsize>,
    rooms: Arc<RwLock<HashMap<usize, RoomState>>>,
}

#[derive(Clone)]
struct RoomState {
    /// Previously stored in AppState
    user_set: HashSet<String>,
    /// Previously created in main.
    tx: broadcast::Sender<Tweet>,
}

impl RoomState {
    fn new() -> Self {
        Self {
            // Track usernames per room
            user_set: HashSet::new(),
            // Create a new channel for every room
            tx: broadcast::channel(1000).0,
        }
    }
    fn insert_user(&mut self, username: String) {
        self.user_set.insert(username);
    }
}

#[derive(Clone, Debug)]
struct Tweet {
    message: TweetInput,
    user: String,
}

impl Display for Tweet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            json!({"user": self.user, "message": self.message.message})
                .to_string()
                .as_str(),
        )
    }
}

impl Tweet {
    fn new(user: String, message: TweetInput) -> Self {
        Self { message, user }
    }
}

impl AppState {
    fn new() -> Self {
        Self {
            views: Arc::new(AtomicUsize::new(0)),
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    fn inc_views(&self) {
        self.views.fetch_add(1, Ordering::Relaxed);
    }

    fn get_views(&self) -> usize {
        self.views.load(Ordering::Relaxed)
    }

    fn reset_views(&self) {
        self.views.store(0, Ordering::Relaxed);
    }
}

pub fn router() -> Router {
    let app_state = Arc::new(AppState::new());
    Router::new()
        .route("/19/health", get(|| async { StatusCode::OK }))
        .route("/19/ws/ping", get(ping_ws))
        .route("/19/reset", post(reset_views))
        .route("/19/views", get(view_count))
        .route("/19/ws/room/:room_id/user/:username", get(connect_to_room))
        .with_state(app_state)
}

async fn ping_ws(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    tracing::info!("client connected to ping ws at address {addr}");
    ws.on_upgrade(handle_ping_socket)
}

async fn handle_ping_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };
        if let Ok(message_text) = msg.to_text() {
            if message_text == "serve" {
                while let Some(Ok(msg)) = socket.recv().await {
                    if let Ok(message_text) = msg.to_text() {
                        if message_text == "ping" {
                            let _ = socket.send(Message::Text("pong".to_string())).await;
                        }
                    }
                }
            }
        }
    }
}

async fn view_count(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    app_state.get_views().to_string()
}

async fn reset_views(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    app_state.reset_views();
}

async fn connect_to_room(
    ws: WebSocketUpgrade,
    Path((room_id, username)): Path<(usize, String)>,
    State(app_state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    tracing::info!("user: {username} connected to room {room_id}, with address {addr}");
    ws.on_upgrade(move |socket| connect_to_room_handler(socket, app_state, room_id, username))
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TweetInput {
    message: String,
}

async fn connect_to_room_handler(
    stream: WebSocket,
    state: Arc<AppState>,
    room_id: usize,
    username: String,
) {
    // By splitting we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();

    // Get the room channel to send the message to it
    let room_sender = {
        // If the room already exists in the app state
        // get the sender from it.
        // else:
        // we insert the new room, and add the user to it.
        let mut rooms = state.rooms.write().unwrap();
        let room_state = rooms.entry(room_id).or_insert_with(RoomState::new);
        room_state.insert_user(username.clone());
        room_state.tx.clone()
    };

    // create room receiver to subscribe to any new message sent to the room channel
    let mut room_receiver = room_sender.subscribe();

    // spawn new task listening to any message sent from the current connected client
    // if there is new message sent from the current client, i send this message to
    // the room channel.
    let mut send_task = {
        let username = username.clone();
        tokio::spawn(async move {
            let room_sender = room_sender.clone();
            while let Some(Ok(Message::Text(msg))) = receiver.next().await {
                if let Ok(tweet_input) = serde_json::from_str::<TweetInput>(msg.as_str()) {
                    tracing::info!(
                        r#"user: "{}" sent message: "{}" to room "{}""#,
                        username,
                        tweet_input.message,
                        room_id
                    );
                    if tweet_input.message.len() <= 128 {
                        room_sender
                            .send(Tweet::new(username.clone(), tweet_input))
                            .unwrap();
                    }
                }
            }
        })
    };

    // spawn new task listening to any message sent to the room channel,
    // if there is new message sent to the room channel,
    // i use the current sender to send this message to current connected client.
    let mut recv_task = {
        // This task will receive messages from client and send them to broadcast subscribers.
        let state = state.clone();
        tokio::spawn(async move {
            while let Ok(msg) = room_receiver.recv().await {
                state.inc_views();
                // Add username before message.
                let _ = sender.send(Message::Text(format!("{}", msg))).await;
            }
        })
    };

    // select on the recv and send task
    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // TODO: if we reach here, means that the user disconnected, so we need to remove the user from the room.
    // TODO: if there is no one left in the room we remove the entire room.
}

#[cfg(test)]
mod tests {

    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;

    #[tokio::test]
    async fn day19_health() {
        let app = router();

        let client = TestClient::new(app);
        let res = client.get("/19/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }
}

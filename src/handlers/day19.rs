#[allow(dead_code, unused_variables)]
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use futures::{stream::StreamExt, SinkExt};
use tokio::sync::broadcast;

struct RoomState {
    rooms: Mutex<HashMap<u64, Room>>,
}

#[allow(dead_code)]
struct Room {
    id: u64,
    users: Mutex<HashSet<String>>,
    tx: broadcast::Sender<String>,
}

#[allow(dead_code)]
struct RoomMessage {
    total_view: u64,
    username: String,
    message: String,
}

pub fn router() -> Router {
    let room_messages_state = Arc::new(RoomState {
        rooms: Mutex::new(HashMap::new()),
    });
    Router::new()
        .route("/19/health", get(|| async { StatusCode::OK }))
        .route("/19/ws/ping", get(ping_ws))
        .route("/19/reset", post(ping_ws))
        .route("/19/views", get(ping_ws))
        .route("/19/ws/room/:room_id/user/:username", get(connect_to_room))
        .with_state(room_messages_state)
}

async fn ping_ws(ws: WebSocketUpgrade) -> impl IntoResponse {
    tracing::info!("client connected");
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };
        tracing::info!("client send msg: {:?}", msg.clone());
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

#[axum::debug_handler]
async fn connect_to_room(
    ws: WebSocketUpgrade,
    Path((room_id, username)): Path<(u64, String)>,
    State(room_state): State<Arc<RoomState>>,
) -> impl IntoResponse {
    tracing::info!("user: {} connected to room {room_id}", username.clone());
    ws.on_upgrade(move |socket| connect_to_room_handler(socket, room_state, room_id, username))
}

async fn connect_to_room_handler(
    stream: WebSocket,
    room_state: Arc<RoomState>,
    room_id: u64,
    username: String,
) {
    let (mut sender, mut receiver) = stream.split();

    let mut state_room = room_state.rooms.lock().unwrap();
    let (tx, _rx) = broadcast::channel(100);
    let room = if let Some(r) = state_room.get_mut(&room_id) {
        r.users.lock().unwrap().insert(username.clone());
        r
    } else {
        let mut new_hashset = HashSet::new();
        new_hashset.insert(username.clone());
        let new_room = Room {
            id: room_id,
            users: Mutex::new(new_hashset),
            tx,
        };
        state_room.insert(room_id, new_room);
        state_room.get(&room_id).unwrap()
    };
    let mut rx = room.tx.subscribe();

    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    #[allow(dead_code, unused_variables)]
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Clone things we want to pass (move) to the receiving task.
    let tx = room.tx.clone();
    let name = username.clone();

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    #[allow(dead_code, unused_variables)]
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Add username before message.
            let _ = tx.send(format!("{name}: {text}"));
        }
    });

    // If any one of the tasks run to completion, we abort the other.
    // tokio::select! {
    //     _ = (&mut send_task) => recv_task.abort(),
    //     _ = (&mut recv_task) => send_task.abort(),
    // };

    // // Send "user left" message (similar to "joined" above).
    // let msg = format!("{username} left.");
    // tracing::debug!("{msg}");
    // let _ = room.tx.send(msg);

    // // Remove username from map so new clients can take it again.
    // room.users.lock().unwrap().remove(&username);
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

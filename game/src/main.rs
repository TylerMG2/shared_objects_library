use axum::{extract::{Query, State, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use shared::{ClientGameEvent, Room, ServerGameEvent};
use tokio::net::TcpListener;
use websocket_rooms::core::{ClientEvent, PlayerFields, RoomJoinQuery, Rooms, ServerRoom};

const MAX_PLAYERS: usize = 8;

#[tokio::main]
async fn main() {
    let state = Rooms::<Room, MAX_PLAYERS>::new(event_handler);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("localhost:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn ws_handler(ws: WebSocketUpgrade, query: Query<RoomJoinQuery>, State(state): State<Rooms<Room, MAX_PLAYERS>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| state.handle_socket(socket, query.0))
}

fn event_handler(room: &mut ServerRoom<Room,  MAX_PLAYERS>, player_index: usize, event: &ClientEvent<ClientGameEvent>) {
    room.room.host = player_index as u8;
    room.update_all(&ServerGameEvent::Test);

    let test = room.room.players[0].unwrap().set_name(&[0; 20]);
}
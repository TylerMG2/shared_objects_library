use axum::{extract::{Query, State, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use websocket_rooms::{core::{ClientEvent, Networked, PlayerFields, RoomJoinQuery, RoomLogic, Rooms, ServerRoom}, proc_macros::{Networked, PlayerFields, RoomFields}};

#[derive(Clone, Networked, PlayerFields, Copy, Serialize, Deserialize, Default, Debug)]
struct Player {
    #[name]
    test: [u8; 20],

    #[disconnected]
    disconnected: bool,

    #[private] // This field should only be sent to the owner of the player, the macro should also enforce this is an Option since
    // only the owner should be able to see this field
    cards: u8,
}

#[derive(Clone, RoomFields, Copy, Serialize, Deserialize, Networked, Default, Debug)]
struct Room {
    #[players]
    players: [Option<Player>; 8],

    #[host]
    host: u8,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
enum ClientGameEvent {
    Test,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
enum ServerGameEvent {
    Test,
}

impl RoomLogic for Room {
    type ClientGameEvent = ClientGameEvent;
    type ServerGameEvent = ServerGameEvent;

    fn validate_event(&self, player_index: usize, action: &ClientGameEvent) -> bool {
        true
    }
}

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
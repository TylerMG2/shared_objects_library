use serde::{Deserialize, Serialize};
use websocket_rooms::{core::{RoomLogic, Networked, PlayerFields, RoomFields}, proc_macros::{Networked, PlayerFields, RoomFields}};

#[derive(Clone, Networked, PlayerFields, Copy, Serialize, Deserialize, Default, Debug)]
pub struct Player {
    #[name]
    pub test: [u8; 20],

    #[disconnected]
    pub  disconnected: bool,

    #[private] // This field should only be sent to the owner of the player, the macro should also enforce this is an Option since
    // only the owner should be able to see this field
    pub  cards: u8,
}

#[derive(Clone, RoomFields, Copy, Serialize, Deserialize, Networked, Default, Debug)]
pub struct Room {
    #[players]
    pub players: [Option<Player>; 8],

    #[host]
    pub host: u8,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum ClientGameEvent {
    Test,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum ServerGameEvent {
    Test,
}

impl RoomLogic for Room {
    type ClientGameEvent = ClientGameEvent;
    type ServerGameEvent = ServerGameEvent;

    fn validate_event(&self, player_index: usize, action: &ClientGameEvent) -> bool {
        true
    }
}
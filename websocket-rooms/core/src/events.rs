use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{Networked, RoomFields};

#[derive(Serialize, Deserialize, Default, Clone)]
pub enum ServerEvent<T> {
    RoomJoined,
    PlayerJoined,
    PlayerLeft,
    PlayerDisconnected,
    PlayerReconnected,
    HostChanged,
    #[default]
    Unknown,
    GameEvent(T),
}

#[derive(Serialize, Deserialize)]
pub struct ServerMessage<GameEvent, Room: Networked + RoomFields + DeserializeOwned> {
    pub event: ServerEvent<GameEvent>,
    pub room: Option<Room::Optional>
}

#[derive(Serialize, Deserialize, Default)]
pub enum ClientEvent<GameEvent: Serialize> {
    JoinRoom { name: [u8; 20] },
    LeaveRoom,
    #[default]
    Unknown,
    GameEvent(GameEvent),
}
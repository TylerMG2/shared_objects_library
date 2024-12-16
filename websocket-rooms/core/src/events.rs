use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{Networked, RoomFields, RoomLogic};

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
pub struct ServerMessage<T: RoomLogic + RoomLogic + Networked> {
    pub event: ServerEvent<T::ServerGameEvent>,
    pub room: Option<T::Optional>
}

#[derive(Serialize, Deserialize, Default)]
pub enum ClientEvent<GameEvent: Serialize> {
    JoinRoom { name: [u8; 20] },
    LeaveRoom,
    #[default]
    Unknown,
    GameEvent(GameEvent),
}
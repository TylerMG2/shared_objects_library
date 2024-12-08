use events::ClientEvent;
use serde::{de::DeserializeOwned, Serialize};
mod networked;
mod events;
mod server;

pub use networked::Networked;
pub use events::ServerEvent;

pub trait PlayerFields {
    type Name: Serialize + DeserializeOwned + Copy + Default;

    fn name(&self) -> Self::Name;
    fn set_name(&mut self, name: Self::Name);
    fn disconnected(&self) -> bool;
    fn set_disconnected(&mut self, disconnected: bool);
}

pub trait RoomFields {
    type Player: PlayerFields;

    fn players(&self) -> &[Option<Self::Player>];
    fn players_mut(&mut self) -> &mut [Option<Self::Player>];
    fn host(&self) -> u8;
    fn set_host(&mut self, host: u8);
}

pub trait RoomLogic 
where 
    Self::Room: RoomFields + Networked + Serialize + DeserializeOwned + Copy,
    Self::ServerGameEvent: Serialize + DeserializeOwned + Clone + Default,
    Self::ClientGameEvent: Serialize + DeserializeOwned + Clone + Default
{
    type Room;
    type ServerGameEvent;
    type ClientGameEvent;

    fn validate_action(&self, player_index: usize, action: &ClientEvent<Self::ClientGameEvent>) -> bool;
}
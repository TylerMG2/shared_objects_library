use serde::{de::DeserializeOwned, Serialize};
mod networked;
mod events;
mod server;
mod client;

pub use networked::Networked;
pub use events::{ClientEvent, ServerEvent};
pub use server::{ServerRoom, Rooms, RoomJoinQuery};

pub trait PlayerFields {
    fn name(&self) -> &[u8];
    fn set_name(&mut self, name: &[u8]);
    fn disconnected(&self) -> bool;
    fn set_disconnected(&mut self, disconnected: bool);
}

pub trait RoomFields {
    type Player: PlayerFields + Default;

    fn players(&self) -> &[Option<Self::Player>];
    fn players_mut(&mut self) -> &mut [Option<Self::Player>];
    fn host(&self) -> u8;
    fn set_host(&mut self, host: u8);
}

pub trait RoomLogic 
where 
    Self::ServerGameEvent: Serialize + DeserializeOwned + Clone + Send,
    Self::ClientGameEvent: Serialize + DeserializeOwned + Clone + Send,
{
    type ServerGameEvent;
    type ClientGameEvent;

    // Validate in almost every game should be shared between the client and server to allow for instant updates
    // This is because the client should be able to predict the outcome of an action before the server sends the update
    fn validate_event(&self, player_index: usize, action: &Self::ClientGameEvent) -> bool;

    // Ideally in the future theres some shared update function here that can be used by the client and server
    // so the client can be given instant feedback on their actions thanks in part to the validate_action function
    // fn handle_event(&mut self, player_index: usize, event: &ClientEvent<Self::ClientGameEvent>);
}
use serde::{de::DeserializeOwned, Serialize};
mod networked;
mod events;

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
    fn players(&self) -> &[Option<impl PlayerFields>];
    fn players_mut(&mut self) -> &mut [Option<impl PlayerFields>];
    fn host(&self) -> u8;
    fn set_host(&mut self, host: u8);
}
pub trait PlayerFields {
    fn name(&self) -> &[u8];
    fn set_name(&mut self, name: &[u8]);
    fn disconnected(&self) -> bool;
    fn set_disconnected(&mut self, disconnected: bool);
}

pub trait RoomFields {
    fn players(&self) -> &[impl PlayerFields];
    fn players_mut(&mut self) -> &mut [impl PlayerFields];
    fn host(&self) -> u8;
    fn set_host(&mut self, host: u8);
}

pub trait Networked {
    fn serialize(&self) -> Vec<u8>;
    fn update_from(&mut self, data: &[u8]);
    fn is_different(&self) -> bool;
}
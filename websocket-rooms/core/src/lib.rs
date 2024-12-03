pub trait PlayerFields {
    fn name(&self) -> &[u8];
    fn set_name(&mut self, name: &[u8]);
    fn disconnected(&self) -> bool;
    fn set_disconnected(&mut self, disconnected: bool);
}
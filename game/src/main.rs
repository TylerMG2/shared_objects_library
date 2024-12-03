use websocket_rooms::{core::PlayerFields, proc_macros::PlayerFields};

fn main() {
    let mut player = Player {
        test: [0; 20],
        disconnected: false,
    };

    test(&mut player);
}

#[derive(PlayerFields, Clone, Copy)]
struct Player {
    #[name] 
    test: [u8; 20],

    #[disconnected] 
    disconnected: bool,
}

fn test(player: &mut impl PlayerFields) {
    println!("{:?}", player.name());
    println!("{:?}", player.disconnected());

    player.set_name(b"helloaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    println!("{:?}", player.name());

    player.set_name(b"t");
    println!("{:?}", player.name());
}
use serde::{Deserialize, Serialize};
use websocket_rooms::{core::{PlayerFields, RoomFields}, proc_macros::{Networked, PlayerFields, RoomFields}};

fn main() {
    let mut player = Player {
        test: [0; 20],
        disconnected: false,
        cards: 0,
    };

    test(&mut player);

    let mut room = Room {
        players: [player; 8],
        host: 0,
    };

    println!("{:?}", room.players[0].name());

    room.players_mut()[0].set_name(b"hello");

    println!("{:?}", room.players[0].name());
}

#[derive(PlayerFields, Clone, Copy, Networked, Serialize, Deserialize)]
struct Player {
    #[name]
    test: [u8; 20],

    #[disconnected]
    disconnected: bool,

    #[private] // This field should only be sent to the owner of the player, the macro should also enforce this is an Option since some clients may not have this field
    cards: u8,
}

#[derive(Clone, Copy, Networked, RoomFields, Serialize, Deserialize)]
struct Room {
    #[players]
    players: [Player; 8],

    #[host]
    host: u8,
}

fn test(player: &mut impl PlayerFields) {
    println!("{:?}", player.name());
    println!("{:?}", player.disconnected());

    player.set_name(b"helloaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    println!("{:?}", player.name());

    player.set_name(b"t");
    println!("{:?}", player.name());
}
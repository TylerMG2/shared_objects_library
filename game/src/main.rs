use serde::{Deserialize, Serialize};
use websocket_rooms::{core::{PlayerFields, RoomFields, Networked}, proc_macros::{PlayerFields, RoomFields}};

fn main() {
    let mut player = Player {
        test: [0; 20],
        disconnected: false,
        cards: 0,
    };

    let test = u8::from_optional(6);
    println!("{:?}", test);

    let diff = 8.differences_with(&8);
    println!("{:?}", diff);

    let mut test2 = [None; 8];
    let mut test2_diff = [Some(6); 8];
    test2_diff[0] = None;

    let diff = test2.differences_with(&test2_diff);
    println!("{:?}", diff);

    let bytes = bincode::serialize(&diff).unwrap();
    println!("Number of bytes: {}", bytes.len());

    let test2_update = test2.update_from_optional(diff.unwrap());
    println!("{:?}", test2);
}

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Player {
    test: [u8; 20],

    disconnected: bool,

    //#[private] // This field should only be sent to the owner of the player, the macro should also enforce this is an Option since some clients may not have this field
    cards: u8,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Room {
    players: [Player; 8],

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
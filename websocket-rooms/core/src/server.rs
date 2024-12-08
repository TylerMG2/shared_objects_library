use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::Message;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::{mpsc::UnboundedSender, RwLock};

use crate::{events::ServerMessage, RoomLogic, Networked, RoomFields, ServerEvent, ClientEvent};

type RoomMap<T, const MAX_PLAYERS: usize> = Arc<RwLock<HashMap<String, ServerRoom<T, MAX_PLAYERS>>>>;

pub struct Connection {
    pub id: String,
    pub sender: Option<UnboundedSender<Message>>,
}

pub struct ServerRoom<T, const MAX_PLAYERS: usize> 
where 
    T: RoomLogic + RoomFields + Networked + Copy + Serialize + DeserializeOwned,
{
    pub room: T,
    pub previous_room: T,
    pub connections: [Option<Connection>; MAX_PLAYERS],
}

impl<T, const MAX_PLAYERS: usize> ServerRoom<T, MAX_PLAYERS> 
where 
    T: RoomLogic + RoomFields + Networked + Copy + Serialize + DeserializeOwned,
{
    pub fn new(room: T) -> Self {
        Self {
            room,
            previous_room: room,
            connections: [const { None }; MAX_PLAYERS],
        }
    }

    pub fn get_connection_index(&self, id: &str) -> Option<usize> {
        self.connections.iter().position(|connection| {
            if let Some(connection) = connection {
                connection.id == id
            } else {
                false
            }
        })
    }

    pub fn handle_event(&mut self, index: usize, event: &ClientEvent<T::ClientGameEvent>) {
        if self.room.validate_action(index, event) {
            //TODO: Call a provided function to update the room state
            // Also ideally we should pass just the clients defined events to this function
            // i.e ClientGameEvent and not the full ClientEvent since those should be handled
            // already
        }
    }

    pub fn update_all(&mut self, event: &ServerEvent<T::ServerGameEvent>) {
        let changes = self.room.differences_with(&self.previous_room);

        for (i, _) in self.connections.iter().enumerate() {
            self.send_message(i, event, changes);
        }

        self.previous_room = self.room;
    }

    // Sends the room changes to all clients except the one at the given index
    pub fn update_except(&mut self, index: usize, event: &ServerEvent<T::ServerGameEvent>) {
        let changes = self.room.differences_with(&self.previous_room);

        for (i, _) in self.connections.iter().enumerate() {
            if i != index {
                self.send_message(i, event, changes);
            }
        }

        self.previous_room = self.room;
    }

    // Sends the room changes to just one client
    // Should only be used for private events and if just private fields have changed
    pub fn update_private(&mut self, index: usize, event: &ServerEvent<T::ServerGameEvent>) {
        let changes = self.room.differences_with(&self.previous_room);
        self.send_message(index, event, changes); // To stop desync issues, we should only send the changes to private fields for the player at this index
        self.previous_room = self.room;
    }

    fn send_message(&self, index: usize, event: &ServerEvent<T::ServerGameEvent>, changes: Option<T::Optional>) {
        if let Some(connection) = &self.connections[index] {
            if let Some(sender) = &connection.sender {

                //TODO: use the index to call a .privatised() method on the message room optional
                // which will remove all fields that are marked as private and not owned by current player

                let message = ServerMessage::<T::ServerGameEvent, T> {
                    event: event.clone(),
                    room: changes,
                };

                sender.send(Message::Binary(bincode::serialize(&message).unwrap())).unwrap();
            }
        }
    }
}

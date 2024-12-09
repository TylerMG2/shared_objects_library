use std::{collections::HashMap, sync::Arc, time::Duration};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::{SplitSink, SplitStream, StreamExt}, SinkExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::{sync::{mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}, RwLock}, time::timeout};

use crate::{events::ServerMessage, ClientEvent, Networked, PlayerFields, RoomFields, RoomLogic, ServerEvent};

pub type HandleEventFn<T, const MAX_PLAYERS: usize> = fn(&mut ServerRoom<T, MAX_PLAYERS>, usize, &ClientEvent<<T as RoomLogic>::ClientGameEvent>);
type RoomMap<T, const MAX_PLAYERS: usize> = Arc<RwLock<HashMap<String, ServerRoom<T, MAX_PLAYERS>>>>;

pub struct Connection {
    pub id: String,
    pub sender: Option<UnboundedSender<Message>>,
}

pub struct ServerRoom<T, const MAX_PLAYERS: usize> 
where 
    T: RoomLogic + RoomFields + Networked + Copy + Serialize + DeserializeOwned + Default,
{
    pub room: T,
    previous_room: T,
    connections: [Option<Connection>; MAX_PLAYERS],
    handle_event: HandleEventFn<T, MAX_PLAYERS>,
}

impl<T, const MAX_PLAYERS: usize> ServerRoom<T, MAX_PLAYERS> 
where 
    T: RoomLogic + RoomFields + Networked + Copy + Serialize + DeserializeOwned + Default,
{
    pub fn new(handle_event: HandleEventFn<T, MAX_PLAYERS>) -> Self {
        let mut room = T::default();
        room.set_host(0);

        Self {
            room,
            previous_room: room,
            connections: [const { None }; MAX_PLAYERS],
            handle_event,
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
        if !self.room.validate_event(index, event) {
            return;
        }

        (self.handle_event)(self, index, event);
    }

    pub fn update_all_server_event(&mut self, event: &ServerEvent<T::ServerGameEvent>) {
        let changes = self.room.differences_with(&self.previous_room);

        for (i, _) in self.connections.iter().enumerate() {
            self.send_message(i, event, changes);
        }

        self.previous_room = self.room;
    }

    // Sends the room changes to all clients except the one at the given index
    fn update_except_server_event(&mut self, index: usize, event: &ServerEvent<T::ServerGameEvent>) {
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
    pub fn update_one_server_event(&mut self, index: usize, event: &ServerEvent<T::ServerGameEvent>) {
        let changes = self.room.differences_with(&self.previous_room);
        self.send_message(index, event, changes); // To stop desync issues, we should only send the changes to private fields for the player at this index
        self.previous_room = self.room;
    }

    fn send_message(&self, index: usize, event: &ServerEvent<T::ServerGameEvent>, changes: Option<T::Optional>) {
        if let Some(connection) = &self.connections[index] {
            if let Some(sender) = &connection.sender {

                // TODO: use the index to call a .privatised() method on the message room optional
                // which will remove all fields that are marked as private and not owned by current player (set to None)
                // Ideally this should also set whole things to None if this means there aren't any changes.
                // For example lets say the server updates a players private field, but it attempts to send the changes to all players
                // Only the player who owns the private field should receive the changes, the other players should receive None for that
                // whole array index.

                let message = ServerMessage::<T::ServerGameEvent, T> {
                    event: event.clone(),
                    room: changes,
                };

                sender.send(Message::Binary(bincode::serialize(&message).unwrap())).unwrap();
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoomJoinQuery {
    pub id: String,
    pub code: String,
}

#[derive(Clone)]
pub struct Rooms<T, const MAX_PLAYERS: usize> 
where 
    T: RoomLogic + RoomFields + Networked + Copy + Serialize + DeserializeOwned + Default,
{
    rooms: RoomMap<T, MAX_PLAYERS>,
    handle_event: HandleEventFn<T, MAX_PLAYERS>,
}

impl <T, const MAX_PLAYERS: usize> Rooms<T, MAX_PLAYERS> 
where 
    T: RoomLogic + RoomFields + Networked + Copy + Serialize + DeserializeOwned + Default
{
    pub fn new(handle_event: HandleEventFn<T, MAX_PLAYERS>) -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            handle_event,
        }
    }

    pub async fn handle_socket(self, socket: WebSocket, query: RoomJoinQuery) {
        if query.id.len() != 36 || query.code.len() != 6 { return; }

        let (tx, rx) = unbounded_channel::<Message>();
        let (sender, mut receiver) = socket.split();
        println!("{} attemping to connect to {}", query.id, query.code);

        let player_index = {
            let result = self.handle_connect(&query.code, &query.id, tx, &mut receiver).await;
            match result {
                Ok(player_index) => {
                    let rooms = self.rooms.read().await;
                    let room = rooms.get(&query.code).expect("Room should of been created in handle_connect");
                    room.update_one_server_event(player_index, &ServerEvent::RoomJoined);
                    player_index
                },
                Err(e) => {
                    println!("{} failed to connect to {}: {}", query.id, query.code, e);
                    return;
                }
            }
        };
        println!("{} connected to {}", query.id, query.code);
    
        let recv_state = self.rooms.clone();
        let recv_query = query.clone();

        let mut send_task = tokio::spawn(send_task(sender, rx));
        let mut recv_task = tokio::spawn(receive_task(recv_state, recv_query, player_index, receiver)); // TODO: Fix this with a static lifetime maybe?

        tokio::select! {
            _ = &mut send_task => recv_task.abort(),
            _ = &mut recv_task => send_task.abort(),
        };
    
        // Send a disconnect event to the room (if the player hasn't already left)
        let mut rooms = self.rooms.write().await;
        let room = rooms.get_mut(&query.code).unwrap();
        if let Some(Some(player)) = room.room.players_mut().get_mut(player_index) {
            if !player.disconnected() {
                player.set_disconnected(true);
                room.update_all_server_event(&ServerEvent::PlayerDisconnected);
            }
        }
        println!("{} left room {}", query.id, query.code);

        // If all players are disconnected, remove the room
        if room.room.players().iter().all(|player| {
            if let Some(player) = player {
                player.disconnected()
            } else {
                true
            }
        }) {
            rooms.remove(&query.code);
            println!("Room {} closed", query.code);
        }
    }

    async fn handle_connect(&self, code: &String, player_id: &String, tx: UnboundedSender<Message>, receiver: &mut SplitStream<WebSocket>) -> Result<usize, String> {
        if let Some(player_index) = self.handle_reconnect(code, player_id, tx.clone()).await {
            return Ok(player_index);
        }

        // Wait 10 seconds for the player to provide a name and code
        let name = match timeout(Duration::from_secs(300), self.wait_for_name(receiver)).await {
            Ok(Ok(data)) => data,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err("Connection timeout: No name and code provided.".to_string()),
        };

        // Now that we have the name, we can lock the rooms map
        let mut rooms = self.rooms.write().await;
        let room = rooms.entry(code.clone()).or_insert(ServerRoom::new(self.handle_event));
        let player_index = room.room.players().iter().position(|player| player.is_none());

        if let Some(player_index) = player_index {
            let player = T::Player::default();
            //player.set_name(name); TODO: Fix trait
            let connection = Connection { id: player_id.clone(), sender: Some(tx) };
            room.connections[player_index] = Some(connection);
            room.room.players_mut()[player_index] = Some(player);

            room.update_except_server_event(player_index, &ServerEvent::PlayerJoined);
            return Ok(player_index);
        }

        // If we reach this point, the room is full
        return Err("Room is full".to_string());
    }

    async fn handle_reconnect(&self, code: &str, player_id: &String, tx: UnboundedSender<Message>) -> Option<usize> {
        let mut rooms = self.rooms.write().await;
        let room = rooms.get_mut(code)?;
    
        let player_index = room.get_connection_index(&player_id)?;
        let player = room.room.players_mut().get_mut(player_index)?.as_mut()?;
        player.set_disconnected(false);
        room.connections[player_index] = Some(Connection { id: player_id.clone(), sender: Some(tx) });
    
        room.update_except_server_event(player_index, &ServerEvent::PlayerReconnected);
        return Some(player_index);
    }

    async fn wait_for_name(&self, receiver: &mut SplitStream<WebSocket>) -> Result<[u8; 20], String> {
        while let Some(msg) = receiver.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(_) => break,
            };
    
            match msg {
                Message::Binary(data) => {
                    let event = bincode::deserialize::<ClientEvent<T::ClientGameEvent>>(data.as_ref()).unwrap_or_default();
                    if let ClientEvent::JoinRoom { name } = event {
                        return Ok(name);
                    }
                }
                _ => {}
            }
        }
        return Err("No name and code provided".into());
    }
}

async fn send_task(mut sender: SplitSink<WebSocket, Message>, mut rx: UnboundedReceiver<Message>) {
    while let Some(msg) = rx.recv().await {
        if sender.send(msg).await.is_err() {
            break;
        }
    }
}

async fn receive_task<T, const MAX_PLAYERS: usize>(
    recv_state: RoomMap<T, MAX_PLAYERS>,
    recv_query: RoomJoinQuery,
    player_index: usize,
    mut receiver: SplitStream<WebSocket>,
) 
where
    T: RoomLogic + RoomFields + Networked + Copy + Serialize + DeserializeOwned + Default + 'static,
{
    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => break, // Close the connection if receiving fails
        };

        match msg {
            Message::Binary(data) => {
                let event = bincode::deserialize::<ClientEvent<T::ClientGameEvent>>(&data).unwrap_or_default();
                let mut rooms = recv_state.write().await;
                let room = rooms.get_mut(&recv_query.code).expect("Room should only be removed if all players are disconnected");

                if let ClientEvent::LeaveRoom = event {
                    room.connections[player_index] = None;
                    room.update_all_server_event(&ServerEvent::PlayerLeft);
                    break;
                }

                // Ensure the player exists before continuing
                if room.room.players().get(player_index).is_none() {
                    break;
                }

                room.handle_event(player_index, &event);
            }
            _ => {}
        }
    }
}
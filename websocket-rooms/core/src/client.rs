use leptos::{logging::log, prelude::{signal, ReadSignal, Set, Write, WriteSignal}};
use serde::de::DeserializeOwned;
use web_sys::{wasm_bindgen::{prelude::Closure, JsCast, JsValue}, ErrorEvent, MessageEvent, WebSocket};

use crate::{events::ServerMessage, Networked, RoomFields, RoomLogic};

type HandleEventFn<T> = fn(ServerMessage<T>) -> ();

pub enum ConnectionStatus {
    Connected,
    Disconnected,
}

pub struct RoomContext<T: RoomFields + Networked + RoomLogic> {
    ws: WebSocket,
    pub connection_status: ReadSignal<ConnectionStatus>,
    pub room: ReadSignal<T>,
    pub set_room: WriteSignal<T>,
}

impl <T: RoomFields + Networked + RoomLogic> RoomContext<T> {
    pub fn send(&self, event: T::ClientGameEvent) -> Result<(), JsValue> {
        let event = bincode::serialize(&event).unwrap();
        self.ws.send_with_u8_array(&event)?;
        Ok(())
    }
}

pub fn create_room_context<T>(websocket_url: &str, handle_event: HandleEventFn<T>) -> Result<RoomContext<T>, JsValue>
where
    T: RoomFields + RoomLogic + Networked + DeserializeOwned + Default + Send + Sync + 'static,
{
    let (room, set_room) = signal(T::default());
    let (connection_status, set_connection_status) = signal(ConnectionStatus::Disconnected);
    let ws = WebSocket::new(websocket_url)?;
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(data) = e.data().dyn_into::<web_sys::js_sys::ArrayBuffer>() {
            let array = web_sys::js_sys::Uint8Array::new(&data);
            let vec = array.to_vec();
            let event = bincode::deserialize::<ServerMessage<T>>(&vec).unwrap();
            set_room.write().update_from_optional(event.room);

            (handle_event)(event);
        }
    });
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        log!("WebSocket error: {:?}", e);
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        set_connection_status.set(ConnectionStatus::Connected);
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    let onclose_callback = Closure::<dyn FnMut()>::new(move || {
        set_connection_status.set(ConnectionStatus::Disconnected);
    });
    ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
    onclose_callback.forget();

    let room = RoomContext { ws, connection_status, room, set_room };
    Ok(room)
}
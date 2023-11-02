use std::sync::Arc;

use serde_derive::{Deserialize, Serialize};
use tokio::{
    net::TcpStream,
    sync::{oneshot::Sender, Mutex},
};

use ipc_client::client::message::{CallObjectRequest, Event, ListObjects, RegisterObject, Success};

#[derive(Debug)]
pub enum Message {
    ProcessInput(Session, Sender<Vec<u8>>),
    RemoveRegistered(Session),
}

#[derive(Debug)]
pub struct SocketHolder {
    pub name: String,
    pub socket: Arc<Mutex<TcpStream>>,
}

#[derive(Debug)]
pub struct Session {
    pub msg: IpcMessage,
    pub socket_holder: SocketHolder,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum IpcMessage {
    None,
    Register(RegisterObject),
    Call(CallObjectRequest),
    Success(Success),
    WaitForObjects(ListObjects),
    Events(Event),
}

impl Session {
    pub fn new(msg: IpcMessage, ipaddress: String, socket: Arc<Mutex<TcpStream>>) -> Self {
        Self {
            msg,
            socket_holder: SocketHolder {
                name: ipaddress,
                socket,
            },
        }
    }
}

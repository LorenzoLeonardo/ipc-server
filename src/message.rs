use std::sync::Arc;

use serde_derive::{Deserialize, Serialize};
use tokio::{
    net::TcpStream,
    sync::{oneshot::Sender, Mutex},
};

pub enum Message {
    ProcessInput(Session, Sender<Vec<u8>>),
    RemoveRegistered(Session),
}

pub struct SocketHolder {
    pub name: String,
    pub socket: Arc<Mutex<TcpStream>>,
}
pub struct Session {
    pub msg: IpcMessage,
    pub socket_holder: SocketHolder,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterObject {
    pub reg_object: String,
}

#[derive(Serialize, Deserialize)]
pub struct CallObject {
    pub object: String,
    pub method: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum IpcMessage {
    None,
    Register(RegisterObject),
    Call(CallObject),
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

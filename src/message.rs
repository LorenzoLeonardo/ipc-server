use std::sync::Arc;

use serde_derive::{Deserialize, Serialize};
use tokio::{
    net::TcpStream,
    sync::{oneshot::Sender, Mutex},
};

use ipc_client::client::message::{CallObjectRequest, ListObjects};

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
pub struct RegisterObject {
    pub reg_object: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Success {
    pub success: String,
}

impl Success {
    pub fn new(success: &str) -> Self {
        Self {
            success: success.to_string(),
        }
    }

    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum IpcMessage {
    None,
    Register(RegisterObject),
    Call(CallObjectRequest),
    Success(Success),
    WaitForObjects(ListObjects),
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

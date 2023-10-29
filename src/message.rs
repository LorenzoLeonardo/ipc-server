use std::{collections::HashMap, sync::Arc};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<HashMap<String, String>>,
}

impl CallObject {
    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
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

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum IpcMessage {
    None,
    Register(RegisterObject),
    Call(CallObject),
    Success(Success),
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

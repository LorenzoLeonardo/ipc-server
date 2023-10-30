use std::{collections::HashMap, sync::Arc};

use serde_derive::{Deserialize, Serialize};
use strum::AsRefStr;
use strum_macros::Display;
use strum_macros::EnumString;
use tokio::{
    net::TcpStream,
    sync::{oneshot::Sender, Mutex},
};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct ListObjects {
    pub list: Vec<String>,
}

impl ListObjects {
    pub fn new(list: Vec<String>) -> Self {
        Self { list }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum IpcMessage {
    None,
    Register(RegisterObject),
    Call(CallObject),
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

#[derive(Debug, EnumString, Display, AsRefStr)]
pub enum StaticReplies {
    #[strum(serialize = "OK")]
    Ok,
    #[strum(serialize = "Object not found")]
    ObjectNotFound,
    #[strum(serialize = "client connection error")]
    ClientConnectionError,
    #[strum(serialize = "server connection error")]
    ServerConnectionError,
    #[strum(serialize = "serde parsing error")]
    SerdeParseError,
}

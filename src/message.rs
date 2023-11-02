use std::sync::Arc;

use serde_derive::{Deserialize, Serialize};
use tokio::{
    net::TcpStream,
    sync::{oneshot::Sender, Mutex},
};

use ipc_client::client::message::{
    CallObjectRequest, Event, ListObjects, RegisterObject, SubscribeToEvent, Success,
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
#[serde(untagged)]
pub enum IpcMessage {
    None,
    Register(RegisterObject),
    Call(CallObjectRequest),
    Success(Success),
    WaitForObjects(ListObjects),
    AddToEventList(SubscribeToEvent),
    BroadCastEvent(Event),
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use ipc_client::client::message::JsonValue;

    use crate::message::IpcMessage;

    #[test]
    fn test_ipc_message() {
        let msg = r#"{"event":"your event here","result":{}}"#;
        let ipc_message: IpcMessage = serde_json::from_str(msg).unwrap();
        if let IpcMessage::BroadCastEvent(event) = ipc_message {
            assert_eq!(event.event, "your event here");
            assert_eq!(event.result, JsonValue::HashMap(HashMap::new()));
        } else {
            panic!("This must be IpcMessage::BroadCastEvent");
        }

        let msg = r#"{"event_name":"your event here"}"#;
        let ipc_message: IpcMessage = serde_json::from_str(msg).unwrap();
        if let IpcMessage::AddToEventList(event) = ipc_message {
            assert_eq!(event.event_name, "your event here");
        } else {
            panic!("This must be IpcMessage::AddToEventList");
        }

        let msg = r#"{"reg_object":"object name"}"#;
        let ipc_message: IpcMessage = serde_json::from_str(msg).unwrap();
        if let IpcMessage::Register(obj) = ipc_message {
            assert_eq!(obj.reg_object, "object name");
        } else {
            panic!("This must be IpcMessage::Register");
        }

        let msg = r#"{"object":"object name","method":"method name","param":{"key":"parameter"}}"#;
        let ipc_message: IpcMessage = serde_json::from_str(msg).unwrap();
        if let IpcMessage::Call(obj) = ipc_message {
            let mut hash = HashMap::new();

            hash.insert("key".into(), JsonValue::String("parameter".into()));
            assert_eq!(obj.object, "object name");
            assert_eq!(obj.method, "method name");
            assert_eq!(obj.param, Some(JsonValue::HashMap(hash)));
        } else {
            panic!("This must be IpcMessage::Call");
        }

        let msg = r#"{"list":["mango","apple","orange"]}"#;
        let ipc_message: IpcMessage = serde_json::from_str(msg).unwrap();
        if let IpcMessage::WaitForObjects(list) = ipc_message {
            assert_eq!(
                list.list,
                vec![
                    "mango".to_string(),
                    "apple".to_string(),
                    "orange".to_string()
                ]
            );
        } else {
            panic!("This must be IpcMessage::WaitForObjects");
        }

        let msg = r#"{"success":"OK"}"#;
        let ipc_message: IpcMessage = serde_json::from_str(msg).unwrap();
        if let IpcMessage::Success(success) = ipc_message {
            assert_eq!(success.success, "OK".to_string());
        } else {
            panic!("This must be IpcMessage::WaitForObjects");
        }
    }
}

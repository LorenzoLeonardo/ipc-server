use std::collections::HashMap;

use ipc_client::client::connector::Connector;
use ipc_client::client::message::{IncomingMessage, JsonValue};
use ipc_client::client::shared_object::{ObjectDispatcher, SharedObject};
use ipc_client::client::wait_for_objects;

use log::LevelFilter;
use tokio::sync::mpsc::unbounded_channel;

use async_trait::async_trait;

use crate::manager::TaskManager;
use crate::{setup_logger, Server};

struct Mango;
struct Apple;
struct Orange;

#[async_trait]
impl SharedObject for Mango {
    async fn remote_call(&self, method: &str, param: Option<JsonValue>) -> JsonValue {
        log::trace!("[Mango] Method: {} Param: {:?}", method, param);

        JsonValue::String("This is my response from mango".into())
    }
}

#[async_trait]
impl SharedObject for Apple {
    async fn remote_call(&self, method: &str, param: Option<JsonValue>) -> JsonValue {
        log::trace!("[Apple] Method: {} Param: {:?}", method, param);

        JsonValue::String("This is my response from apple".into())
    }
}

#[async_trait]
impl SharedObject for Orange {
    async fn remote_call(&self, method: &str, param: Option<JsonValue>) -> JsonValue {
        log::trace!("[Orange] Method: {} Param: {:?}", method, param);

        JsonValue::String("This is my response from orange".into())
    }
}

#[tokio::test]
async fn test_server() {
    setup_logger(LevelFilter::Trace);
    let (tx, rx) = unbounded_channel();

    // The server
    let _server = tokio::spawn(async move {
        TaskManager::spawn(rx).await;
        Server::spawn(tx).await;
    });

    // The process that shares objects
    let _process1 = tokio::spawn(async move {
        let mut shared = ObjectDispatcher::new().await.unwrap();

        shared
            .register_object("mango", Box::new(Mango))
            .await
            .unwrap();
        shared
            .register_object("apple", Box::new(Apple))
            .await
            .unwrap();
        shared
            .register_object("orange", Box::new(Orange))
            .await
            .unwrap();
        let _r = shared.spawn().await;
    });

    // The processes that calls remote objects
    let process2 = tokio::spawn(async move {
        // Wait for objects before connecting.
        let list = vec![
            "mango".to_string(),
            "apple".to_string(),
            "orange".to_string(),
        ];
        wait_for_objects::wait_for_objects(list).await;

        let proxy = Connector::connect().await.unwrap();

        let mut param = HashMap::new();
        param.insert(
            "provider".to_string(),
            JsonValue::String("microsoft".to_string()),
        );

        let result = proxy
            .remote_call("mango", "login", Some(JsonValue::HashMap(param)))
            .await
            .unwrap();

        let result: IncomingMessage = serde_json::from_slice(result.as_slice()).unwrap();

        if let IncomingMessage::CallResponse(e) = result {
            log::trace!("[Process2] Response: {:?}", e);
            assert_eq!(
                e.response,
                JsonValue::String(String::from("This is my response from mango"))
            );
        } else {
            panic!("There is must be a valid response now!");
        }
    });

    let process3 = tokio::spawn(async move {
        // Wait for objects before connecting.
        let list = vec![
            "mango".to_string(),
            "apple".to_string(),
            "orange".to_string(),
        ];
        wait_for_objects::wait_for_objects(list).await;

        let proxy = Connector::connect().await.unwrap();

        let result = proxy.remote_call("apple", "login", None).await.unwrap();

        let result: IncomingMessage = serde_json::from_slice(result.as_slice()).unwrap();

        if let IncomingMessage::CallResponse(e) = result {
            log::trace!("[Process3] Response: {:?}", e);
            assert_eq!(
                e.response,
                JsonValue::String(String::from("This is my response from apple"))
            );
        } else {
            panic!("There is must be a valid response now!");
        }
    });

    let process4 = tokio::spawn(async move {
        // Wait for objects before connecting.
        let list = vec![
            "mango".to_string(),
            "apple".to_string(),
            "orange".to_string(),
        ];
        wait_for_objects::wait_for_objects(list).await;

        let proxy = Connector::connect().await.unwrap();

        let result = proxy.remote_call("orange", "login", None).await.unwrap();

        let result: IncomingMessage = serde_json::from_slice(result.as_slice()).unwrap();

        if let IncomingMessage::CallResponse(e) = result {
            log::trace!("[Process4] Response: {:?}", e);
            assert_eq!(
                e.response,
                JsonValue::String(String::from("This is my response from orange"))
            );
        } else {
            panic!("There is must be a valid response now!");
        }
    });

    let _ = tokio::join!(process2, process3, process4);
}

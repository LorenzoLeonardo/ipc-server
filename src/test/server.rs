use core::panic;
use std::collections::HashMap;

use ipc_client::client::connector::Connector;
use ipc_client::client::message::{CallObjectResponse, IncomingMessage, OutgoingMessage};
use ipc_client::client::shared_object::{ObjectDispatcher, SharedObject};

use ipc_client::client::wait_for_objects;
use tokio::sync::mpsc::unbounded_channel;

use async_trait::async_trait;

use crate::manager::TaskManager;
use crate::{setup_logger, Server};

struct Sample;

#[async_trait]
impl SharedObject for Sample {
    async fn remote_call(
        &self,
        method: &str,
        param: Option<HashMap<String, String>>,
    ) -> OutgoingMessage {
        log::trace!("Method: {} Param: {:?}", method, param);

        OutgoingMessage::CallResponse(CallObjectResponse::new("This is my response"))
    }
}

#[tokio::test]
async fn test_server() {
    setup_logger().unwrap();
    let (tx, rx) = unbounded_channel();

    let server = tokio::spawn(async move {
        TaskManager::spawn(rx).await;
        Server::spawn(tx).await;
    });

    let process1 = tokio::spawn(async move {
        let mut shared = ObjectDispatcher::new().await.unwrap();
        let shared_object = Sample;
        shared
            .register_object("object.name", Box::new(shared_object))
            .await
            .unwrap();
        let _r = shared.spawn();
    });

    let process2 = tokio::spawn(async move {
        // Wait for objects before connecting.
        let list = vec!["object.name".to_string()];
        wait_for_objects::wait_for_objects(list).await;

        let proxy = Connector::connect().await.unwrap();

        let result = proxy
            .remote_call("object.name", "login", None)
            .await
            .unwrap();

        let result: IncomingMessage = serde_json::from_slice(result.as_slice()).unwrap();

        if let IncomingMessage::CallResponse(e) = result {
            log::trace!("{:?}", e);
        } else {
            panic!("There is must be a valid response now!");
        }
    });

    let _ = tokio::join!(process1, process2);
    server.abort();
}

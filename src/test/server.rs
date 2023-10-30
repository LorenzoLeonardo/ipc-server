use std::collections::HashMap;
use std::time::Duration;

use ipc_client::client::connector::Connector;
use ipc_client::client::message::{CallObjectResponse, IncomingMessage, OutgoingMessage};
use ipc_client::client::shared_object::{ObjectDispatcher, SharedObject};

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
            .register_object("applications.oauth2", Box::new(shared_object))
            .await
            .unwrap();
        shared.spawn().await;
    });

    let process2 = tokio::spawn(async move {
        let proxy = Connector::connect().await.unwrap();

        for _n in 0..2 {
            let result = proxy
                .remote_call("applications.oauth2", "login", None)
                .await
                .unwrap();

            let result: IncomingMessage = serde_json::from_slice(result.as_slice()).unwrap();
            log::trace!("Result: {:?}", result);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    process2.await.unwrap();
    process1.abort();
    server.abort();
}

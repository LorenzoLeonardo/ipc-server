use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use ipc_client::client::connector::Connector;
use ipc_client::client::error::Error;
use ipc_client::client::shared_object::{ObjectDispatcher, SharedObject};
use ipc_client::client::wait_for_objects;
use ipc_client::{ENV_LOGGER, ENV_SERVER_ADDRESS};

use async_trait::async_trait;
use json_elem::jsonelem::JsonElem;
use tokio::runtime::Builder;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex;
use tokio::task::LocalSet;

use crate::manager::TaskManager;
use crate::{setup_logger, Server};

struct Mango;
struct Apple;
struct Orange;

#[async_trait]
impl SharedObject for Mango {
    async fn remote_call(&self, method: &str, param: Option<JsonElem>) -> Result<JsonElem, Error> {
        log::trace!("[Mango] Method: {} Param: {:?}", method, param);

        Ok(JsonElem::String("This is my response from mango".into()))
    }
}

#[async_trait]
impl SharedObject for Apple {
    async fn remote_call(&self, method: &str, param: Option<JsonElem>) -> Result<JsonElem, Error> {
        log::trace!("[Apple] Method: {} Param: {:?}", method, param);

        Ok(JsonElem::String("This is my response from apple".into()))
    }
}

#[async_trait]
impl SharedObject for Orange {
    async fn remote_call(&self, method: &str, param: Option<JsonElem>) -> Result<JsonElem, Error> {
        log::trace!("[Orange] Method: {} Param: {:?}", method, param);

        Err(Error::new(JsonElem::String(
            "exception happend".to_string(),
        )))
    }
}

fn find_available_port(start_port: u16) -> Option<u16> {
    (start_port..=u16::MAX).find(|&port| std::net::TcpListener::bind(("127.0.0.1", port)).is_ok())
}

#[ctor::ctor]
fn setup_server() {
    std::env::set_var(ENV_LOGGER, "trace");
    setup_logger();
    let address = format!("127.0.0.1:{}", find_available_port(3000).unwrap());

    std::env::set_var(ENV_SERVER_ADDRESS, address);
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();

    std::thread::spawn(move || {
        let local = LocalSet::new();
        local.spawn_local(async move {
            let (tx, rx) = unbounded_channel();

            // The server
            let server = tokio::spawn(async move {
                TaskManager::spawn(rx).await;
                Server::spawn(tx).await;
            });

            let _ = server.await;
        });
        runtime.block_on(local);
    });
}

#[tokio::test]
async fn test_server() {
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

    let process2_result = Arc::new(Mutex::new(JsonElem::String(String::new())));
    let process2_result2 = process2_result.clone();
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
            JsonElem::String("microsoft".to_string()),
        );

        let result = proxy
            .remote_call("mango", "login", Some(JsonElem::HashMap(param)))
            .await
            .unwrap();
        log::trace!("[Process 2]: {}", result);
        let mut actual = process2_result2.lock().await;
        *actual = result;
    });

    let process3_result = Arc::new(Mutex::new(JsonElem::String(String::new())));
    let process3_result2 = process3_result.clone();
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
        log::trace!("[Process 3]: {}", result);

        let mut actual = process3_result2.lock().await;
        *actual = result;
    });

    let process4_result = Arc::new(Mutex::new(Error::new(JsonElem::String(String::new()))));
    let process4_result2 = process4_result.clone();
    let process4 = tokio::spawn(async move {
        // Wait for objects before connecting.
        let list = vec![
            "mango".to_string(),
            "apple".to_string(),
            "orange".to_string(),
        ];
        wait_for_objects::wait_for_objects(list).await;

        let proxy = Connector::connect().await.unwrap();

        let result = proxy
            .remote_call("orange", "login", None)
            .await
            .unwrap_err();
        log::trace!("[Process 4]: {}", result);

        let mut actual = process4_result2.lock().await;
        *actual = result;
    });

    let _ = tokio::join!(process2, process3, process4);

    let res2 = process2_result.lock().await;
    assert_eq!(
        *res2,
        JsonElem::String("This is my response from mango".into())
    );

    let res3 = process3_result.lock().await;
    assert_eq!(
        *res3,
        JsonElem::String("This is my response from apple".into())
    );

    let res4 = process4_result.lock().await;
    assert_eq!(
        *res4,
        Error::new(JsonElem::String("exception happend".to_string()))
    );
}

struct TestEvent;

#[async_trait]
impl SharedObject for TestEvent {
    async fn remote_call(&self, method: &str, param: Option<JsonElem>) -> Result<JsonElem, Error> {
        log::trace!("[Event] Method: {} Param: {:?}", method, param);

        Ok(JsonElem::String("This is my response from event".into()))
    }
}

#[tokio::test]
async fn test_event() {
    // The process that shares objects
    let process1 = tokio::spawn(async move {
        let mut shared = ObjectDispatcher::new().await.unwrap();

        shared
            .register_object("event", Box::new(TestEvent))
            .await
            .unwrap();
        let _r = shared.spawn().await;
    });

    let process2 = tokio::spawn(async move {
        // Wait for objects before connecting.
        let list = vec!["event".to_string()];
        wait_for_objects::wait_for_objects(list).await;

        let proxy = Connector::connect().await.unwrap();

        for _n in 0..100 {
            proxy
                .send_event(
                    "event",
                    JsonElem::String("Sending you this event!!".to_string()),
                )
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    });

    let process3 = tokio::spawn(async move {
        // Wait for objects before connecting.
        let list = vec!["event".to_string()];
        wait_for_objects::wait_for_objects(list).await;

        let proxy = Connector::connect().await.unwrap();
        proxy
            .listen_for_event("event", |param| async move {
                log::trace!("I HAVE RECEIVED: {param:?}");

                if param == JsonElem::String("Sending you this event!!".to_string()) {
                    std::env::set_var("EVENT_TEST", true.to_string());
                }
                Ok::<(), Error>(())
            })
            .await
            .unwrap();
    });

    let (process1, process2, process3) = tokio::join!(process1, process2, process3);

    process1.unwrap();
    process2.unwrap();
    process3.unwrap();

    assert_eq!(
        std::env::var("EVENT_TEST").unwrap_or(false.to_string()),
        true.to_string()
    );
}

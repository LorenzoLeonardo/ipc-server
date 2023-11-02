use ipc_client::client::message::{CallObjectRequest, JsonValue};
use log::LevelFilter;

use crate::{message::IpcMessage, setup_logger};

#[test]
fn test_call_object_request() {
    setup_logger(LevelFilter::Trace);

    let obj = CallObjectRequest::new("object", "method")
        .parameter("key1", JsonValue::String("value1".into()));

    let ipc: IpcMessage = serde_json::from_slice(obj.serialize().unwrap().as_slice()).unwrap();
    log::trace!("{:?}", ipc);

    let obj = CallObjectRequest::new("object", "method")
        .parameter("provider", JsonValue::String("Microsoft".into()))
        .parameter("process", JsonValue::String("process".into()));

    let ipc: IpcMessage = serde_json::from_slice(obj.serialize().unwrap().as_slice()).unwrap();
    log::trace!("{:?}", ipc);

    let obj = CallObjectRequest::new("object", "method");

    let ipc: IpcMessage = serde_json::from_slice(obj.serialize().unwrap().as_slice()).unwrap();
    log::trace!("{:?}", ipc);
}

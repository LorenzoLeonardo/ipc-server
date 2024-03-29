use std::collections::HashMap;

use json_elem::jsonelem::JsonElem;

use ipc_client::client::message::CallObjectRequest;

use crate::{message::IpcMessage, setup_logger};

#[test]
fn test_call_object_request() {
    setup_logger();

    let mut hash = HashMap::new();
    hash.insert("key1".to_string(), JsonElem::String("value1".into()));
    let obj = CallObjectRequest::new("object", "method", Some(JsonElem::HashMap(hash)));

    let ipc: IpcMessage = serde_json::from_slice(obj.serialize().unwrap().as_slice()).unwrap();
    log::trace!("{:?}", ipc);

    let mut hash = HashMap::new();
    hash.insert("provider".to_string(), JsonElem::String("Microsoft".into()));
    hash.insert("process".to_string(), JsonElem::String("process".into()));
    let obj = CallObjectRequest::new("object", "method", Some(JsonElem::HashMap(hash)));

    let ipc: IpcMessage = serde_json::from_slice(obj.serialize().unwrap().as_slice()).unwrap();
    log::trace!("{:?}", ipc);

    let obj = CallObjectRequest::new("object", "method", None);

    let ipc: IpcMessage = serde_json::from_slice(obj.serialize().unwrap().as_slice()).unwrap();
    log::trace!("{:?}", ipc);
}

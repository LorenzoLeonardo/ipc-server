use std::collections::HashMap;

use ipc_client::client::message::CallObjectRequest;

use crate::{message::IpcMessage, setup_logger};

#[test]
fn test_call_object_request() {
    setup_logger().unwrap_or_else(|e| {
        println!("{}", e);
    });
    let obj = CallObjectRequest::new("object", "method").parameter("key1", "value1");

    let ipc: IpcMessage = serde_json::from_slice(obj.serialize().unwrap().as_slice()).unwrap();
    log::trace!("{:?}", ipc);

    let mut param = HashMap::new();

    param.insert("process".to_string(), "process name".to_string());
    param.insert("provider".to_string(), "Microsoft".to_string());
    let obj = CallObjectRequest::new("object", "method").parameters(Some(param));

    let ipc: IpcMessage = serde_json::from_slice(obj.serialize().unwrap().as_slice()).unwrap();
    log::trace!("{:?}", ipc);
}

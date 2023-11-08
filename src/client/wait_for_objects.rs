use tokio::net::TcpStream;

use crate::SERVER_ADDRESS;

use super::{
    message::{IncomingMessage, ListObjects, OutgoingMessage},
    socket::Socket,
};

/// A function that will guarantees that the object is already available for
/// remote method calls for synchronization purposes.
pub async fn wait_for_objects(list: Vec<String>) {
    let stream = TcpStream::connect(SERVER_ADDRESS).await.unwrap();

    let socket = Socket::new(stream).unwrap();

    loop {
        socket
            .write_all(
                OutgoingMessage::WaitForObjects(ListObjects::new(list.clone()))
                    .serialize()
                    .unwrap()
                    .as_slice(),
            )
            .await
            .unwrap_or_else(|e| {
                log::trace!("{:?}", e);
            });

        let mut buf = Vec::new();
        let n = match socket.read(&mut buf).await {
            Ok(size) => size,
            Err(e) => {
                log::error!("{:?}", e);
                break;
            }
        };

        if let Ok(response) = serde_json::from_slice(&buf[0..n]) {
            if let IncomingMessage::WaitForObjects(v) = response {
                if v.list.is_empty() {
                    continue;
                } else {
                    break;
                }
            }
        } else {
            log::trace!("Serde error!");
        }
    }
}

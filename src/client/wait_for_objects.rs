use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{MAX_DATA, SERVER_ADDRESS};

use super::message::{IncomingMessage, ListObjects, OutgoingMessage};

/// A function that will guarantees that the object is already available for
/// remote method calls for synchronization purposes.
pub async fn wait_for_objects(list: Vec<String>) {
    let mut stream = TcpStream::connect(SERVER_ADDRESS).await.unwrap();

    loop {
        stream
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

        let mut buf = [0u8; MAX_DATA];
        let n = stream.read(&mut buf).await.map_or_else(
            |e| {
                log::error!("{:?}", e);
                0
            },
            |size: usize| {
                log::trace!("Read size: {}", size);
                size
            },
        );
        if n == 0 {
            return;
        }
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

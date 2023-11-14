use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{CHUNK_SIZE, SERVER_ADDRESS};

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

        let mut buf = Vec::new();
        let n = read(&mut stream, &mut buf).await.map_or_else(
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

async fn read(socket: &mut TcpStream, data: &mut Vec<u8>) -> std::io::Result<usize> {
    loop {
        let mut buffer = [0u8; CHUNK_SIZE];
        match socket.read(&mut buffer).await {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    return Ok(bytes_read);
                }
                data.extend_from_slice(&buffer[0..bytes_read]);

                if bytes_read < CHUNK_SIZE {
                    return Ok(data.len());
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

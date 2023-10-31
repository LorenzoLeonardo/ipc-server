use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use super::message::{IncomingMessage, ListObjects, OutgoingMessage};

pub async fn wait_for_objects(list: Vec<String>) {
    let mut stream = TcpStream::connect("127.0.0.1:1986").await.unwrap();

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
        stream.read_buf(&mut buf).await.map_or_else(
            |e| {
                log::error!("{:?}", e);
            },
            |size: usize| {
                log::trace!("Read size: {}", size);
            },
        );
        if let Ok(response) = serde_json::from_slice(buf.as_slice()) {
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

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
            .unwrap();

        let mut buf = [0u8; u16::MAX as usize];
        let n = stream.read(&mut buf).await.unwrap();
        let response: IncomingMessage = serde_json::from_slice(&buf[0..n]).unwrap();

        if let IncomingMessage::WaitForObjects(v) = response {
            if v.list.is_empty() {
                continue;
            } else {
                break;
            }
        }
    }
}

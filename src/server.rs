use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::UnboundedSender,
        oneshot::{self},
    },
};

use crate::error::Error;
use crate::message::{IpcMessage, Message, Session};

pub struct Server;

impl Server {
    pub async fn spawn(tx: UnboundedSender<Message>) {
        let listener = TcpListener::bind("127.0.0.1:1986").await.unwrap();

        println!("Server listening on 127.0.0.1:1986");
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            tokio::spawn(Server::handle_client(socket, tx.clone()));
        }
    }

    async fn handle_client(socket: TcpStream, tx: UnboundedSender<Message>) {
        let ip = socket.peer_addr().unwrap().to_string();
        println!("Client connected: {}", ip);

        let mut buffer = [0u8; u16::MAX as usize];
        let tcp = Arc::new(Mutex::new(socket));

        loop {
            // Use try_read to check if there is data available to read without blocking.
            let mut socket = tcp.lock().await;
            if let Ok(bytes_read) = socket.try_read(&mut buffer) {
                if bytes_read == 0 {
                    // The client has closed the connection.
                    break;
                }
                let received_data = &buffer[..bytes_read];
                if let Ok(ipc_message) = serde_json::from_slice(received_data) {
                    let session: Session = Session::new(ipc_message, ip.clone(), tcp.clone());

                    let (oneshot_tx, oneshot_rx) = oneshot::channel();
                    tx.send(Message::ProcessInput(session, oneshot_tx))
                        .unwrap_or_else(|e| {
                            eprintln!("{:?}", e);
                        });

                    let reply = oneshot_rx.await.unwrap_or_else(|e| {
                        eprintln!("{:?}", e);
                        Vec::new()
                    });
                    if let Err(e) = socket.write_all(reply.as_slice()).await {
                        eprintln!("Error writing data to client: {}", e);
                        break;
                    }
                } else {
                    if let Err(e) = socket
                        .write_all(&Error::new("parse error").serialize())
                        .await
                    {
                        eprintln!("Error writing data to client: {}", e);
                        break;
                    }
                }
                println!("looping . . . .");
            } else {
                // No data available to read, continue other tasks or operations.
                tokio::task::yield_now().await;
            }
        }

        let session: Session = Session::new(IpcMessage::None, ip.clone(), tcp.clone());

        tx.send(Message::RemoveRegistered(session))
            .unwrap_or_else(|e| {
                eprintln!("{:?}", e);
            });
        println!("Client disconnected: {:?}", ip);
    }
}

use std::sync::Arc;

use tokio::io::Interest;
use tokio::sync::Mutex;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::UnboundedSender,
        oneshot::{self},
    },
};

use ipc_client::SERVER_ADDRESS;

use crate::error::Error;
use crate::message::{IpcMessage, Message, Session};

pub struct Server;

impl Server {
    pub async fn spawn(tx: UnboundedSender<Message>) {
        let listener = TcpListener::bind(SERVER_ADDRESS).await.unwrap();

        log::trace!("Server listening on {}", SERVER_ADDRESS);
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            tokio::spawn(Server::handle_client(socket, tx.clone()));
        }
    }

    async fn handle_client(socket: TcpStream, tx: UnboundedSender<Message>) {
        let ip = socket.peer_addr().unwrap().to_string();
        log::trace!("[{}]: Client connected", ip);

        let tcp = Arc::new(Mutex::new(socket));

        loop {
            let mut buffer = Vec::new();
            // Use try_read to check if there is data available to read without blocking.
            let mut socket = tcp.lock().await;
            let ready = socket
                .ready(Interest::READABLE | Interest::WRITABLE)
                .await
                .unwrap();

            if ready.is_readable() {
                if let Ok(bytes_read) = socket.try_read_buf(&mut buffer) {
                    if bytes_read == 0 {
                        // The client has closed the connection.
                        break;
                    }
                    match serde_json::from_slice(buffer.as_slice()) {
                        Ok(ipc_message) => {
                            let session: Session =
                                Session::new(ipc_message, ip.clone(), tcp.clone());

                            let (oneshot_tx, oneshot_rx) = oneshot::channel();
                            tx.send(Message::ProcessInput(session, oneshot_tx))
                                .unwrap_or_else(|e| {
                                    log::error!("{:?}", e);
                                });

                            let reply = oneshot_rx.await.unwrap_or_else(|e| {
                                log::error!("{:?}", e);
                                Vec::new()
                            });
                            if let Err(e) = socket.write_all(reply.as_slice()).await {
                                log::error!("Error writing data to client: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            if let Err(e) = socket
                                .write_all(&Error::new(e.to_string().as_str()).serialize().unwrap())
                                .await
                            {
                                log::error!("Error writing data to client: {}", e);
                                break;
                            }
                        }
                    }
                } else {
                    // No data available to read, continue other tasks or operations.
                    tokio::task::yield_now().await;
                }
            }
        }

        let session: Session = Session::new(IpcMessage::None, ip.clone(), tcp.clone());

        tx.send(Message::RemoveRegistered(session))
            .unwrap_or_else(|e| {
                log::error!("{:?}", e);
            });
        log::trace!("[{}]: Client disconnected", ip);
    }
}

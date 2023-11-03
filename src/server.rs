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

/// This is the IPC server, it handles incoming messages from different processes
/// and send each task to the TaskManager for proper handling.
pub struct Server;

impl Server {
    /// Spawn the IPC server to listen concurrent incoming messages.
    pub async fn spawn(tx: UnboundedSender<Message>) {
        let listener = TcpListener::bind(SERVER_ADDRESS).await.unwrap();

        log::trace!("Server listening on {}", SERVER_ADDRESS);
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            tokio::spawn(Server::handle_client(socket, tx.clone()));
        }
    }

    /// Handles the received messages and pass it into TaskManager for proper handling.
    async fn handle_client(socket: TcpStream, tx: UnboundedSender<Message>) {
        let ip = socket.peer_addr().unwrap().to_string();
        log::trace!("[{}]: Client connected", ip);

        let tcp = Arc::new(Mutex::new(socket));

        loop {
            let mut buffer = [0u8; u16::MAX as usize];
            // Use try_read to check if there is data available to read without blocking.
            let mut socket = tcp.lock().await;
            let ready = socket
                .ready(Interest::READABLE | Interest::WRITABLE)
                .await
                .unwrap();

            if ready.is_readable() {
                if let Ok(bytes_read) = socket.try_read(&mut buffer) {
                    if bytes_read == 0 {
                        // The client has closed the connection.
                        break;
                    }
                    log::trace!(
                        "SERVER RECEIVED: {}",
                        String::from_utf8(buffer[0..bytes_read].to_vec()).unwrap()
                    );
                    match serde_json::from_slice(&buffer[0..bytes_read]) {
                        Ok(ipc_message) => {
                            log::trace!("IpcMessage => {:?}", &ipc_message);
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
                            log::error!(
                                "{}: {}",
                                e,
                                String::from_utf8(buffer[0..bytes_read].to_vec()).unwrap()
                            );
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

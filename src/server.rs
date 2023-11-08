use ipc_client::client::socket::Socket;
use tokio::{
    net::TcpListener,
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
            let socket = Socket::new(socket).unwrap();
            tokio::spawn(Server::handle_client(socket, tx.clone()));
        }
    }

    /// Handles the received messages and pass it into TaskManager for proper handling.
    async fn handle_client(socket: Socket, tx: UnboundedSender<Message>) {
        let ip = socket.ip_address();
        log::trace!("[{}]: Client connected", ip);

        loop {
            let mut buffer = Vec::new();

            match socket.read(&mut buffer).await {
                Ok(bytes_read) => {
                    log::trace!(
                        "SERVER RECEIVED: {}",
                        String::from_utf8(buffer[0..bytes_read].to_vec()).unwrap()
                    );
                    match serde_json::from_slice(&buffer[0..bytes_read]) {
                        Ok(ipc_message) => {
                            log::trace!("IpcMessage => {:?}", &ipc_message);
                            let session: Session = Session::new(ipc_message, socket.clone());

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
                }
                Err(e) => {
                    log::error!("{:?}", e);
                    break;
                }
            }
        }

        let session: Session = Session::new(IpcMessage::None, socket.clone());

        tx.send(Message::RemoveRegistered(session))
            .unwrap_or_else(|e| {
                log::error!("{:?}", e);
            });
        log::trace!("[{}]: Client disconnected", ip);
    }
}

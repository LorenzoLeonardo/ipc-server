use std::collections::HashMap;
use std::sync::Arc;

use serde_derive::Deserialize;
use serde_derive::Serialize;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{unbounded_channel, UnboundedSender},
        oneshot::{self, Sender},
    },
};

enum Message {
    ProcessInput(Session, Sender<Vec<u8>>),
    RemoveRegistered(Session),
}

struct SocketHolder {
    name: String,
    socket: Arc<Mutex<TcpStream>>,
}
struct Session {
    msg: IpcMessage,
    socket_holder: SocketHolder,
}

#[derive(Serialize, Deserialize)]
struct RegisterObject {
    reg_object: String,
}

#[derive(Serialize, Deserialize)]
struct CallObject {
    object: String,
    method: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum IpcMessage {
    None,
    Register(RegisterObject),
    Call(CallObject),
}

impl Session {
    fn new(msg: IpcMessage, socket_holder: SocketHolder) -> Self {
        Self { msg, socket_holder }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (tx, mut rx) = unbounded_channel();

    tokio::spawn(async move {
        let mut list_session = HashMap::new();
        loop {
            let msg = rx.recv().await.unwrap();
            match msg {
                Message::ProcessInput(session, tx) => {
                    //println!("Message Received: {:?}", data);
                    match session.msg {
                        IpcMessage::Register(data) => {
                            println!("Register");
                            list_session.insert(data.reg_object, session.socket_holder);
                            println!("Number of list: {}", list_session.len());
                            tx.send("OK".as_bytes().to_vec()).unwrap();
                        }
                        IpcMessage::Call(data) => {
                            println!("Call");
                            if let Some(s) = list_session.get(data.object.as_str()) {
                                let mut socket = s.socket.lock().await;
                                let reply = serde_json::to_string(&data).unwrap();
                                socket.write_all(reply.as_bytes()).await.unwrap();

                                let mut buffer = [0u8; 1024];
                                if let Ok(bytes_read) = socket.read(&mut buffer).await {
                                    if bytes_read == 0 {
                                        let error = r#"{"error":"client has lost connection"}"#;
                                        tx.send(error.as_bytes().to_vec()).unwrap();
                                    } else {
                                        tx.send(buffer[..bytes_read].to_vec()).unwrap();
                                    }
                                } else {
                                    let error = r#"{"error":"read error"}"#;
                                    tx.send(error.as_bytes().to_vec()).unwrap();
                                }
                            } else {
                                let error = r#"{"error":"not found"}"#;
                                tx.send(error.as_bytes().to_vec()).unwrap();
                            }
                        }
                        _ => {}
                    }
                }
                Message::RemoveRegistered(session) => {
                    list_session.retain(|_, v| v.name != session.socket_holder.name);
                    println!("Number of list: {}", list_session.len());
                }
            }
        }
    });

    // Create a TCP listener that listens on a specific address and port.
    let listener = TcpListener::bind("0.0.0.0:1986").await.unwrap();

    println!("Server listening on 0.0.0.0:1986");

    // Main server loop to accept incoming client connections.
    loop {
        let (socket, _) = listener.accept().await.unwrap();

        // Spawn a new asynchronous task to handle the client.
        tokio::spawn(handle_client(socket, tx.clone()));
    }
}

async fn handle_client(socket: TcpStream, tx: UnboundedSender<Message>) {
    let ip = socket.peer_addr().unwrap().to_string();
    println!("Client connected: {:?}", socket.peer_addr().unwrap());

    let mut buffer = [0u8; 1024];
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
                let socket_holder = SocketHolder {
                    name: ip.clone(),
                    socket: tcp.clone(),
                };
                let session: Session = Session::new(ipc_message, socket_holder);

                let (oneshot_tx, oneshot_rx) = oneshot::channel();
                tx.send(Message::ProcessInput(session, oneshot_tx)).unwrap();

                let reply = oneshot_rx.await.unwrap();
                if let Err(e) = socket.write_all(reply.as_slice()).await {
                    eprintln!("Error writing data to client: {}", e);
                    break;
                }
            } else {
                let error = r#"{"error":"parse error."}"#;
                if let Err(e) = socket.write_all(error.as_bytes()).await {
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

    let socket_holder = SocketHolder {
        name: tcp.lock().await.peer_addr().unwrap().to_string(),
        socket: tcp.clone(),
    };
    let session: Session = Session::new(IpcMessage::None, socket_holder);

    tx.send(Message::RemoveRegistered(session)).unwrap();
    println!(
        "Client disconnected: {:?}",
        tcp.lock().await.peer_addr().unwrap()
    );
}

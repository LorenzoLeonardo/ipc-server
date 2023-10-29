mod error;
mod manager;
mod message;
mod server;
#[cfg(test)]
mod test;

use server::Server;
use tokio::sync::mpsc::unbounded_channel;

use crate::manager::TaskManager;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (tx, rx) = unbounded_channel();

    TaskManager::spawn(rx).await;
    Server::spawn(tx).await
}

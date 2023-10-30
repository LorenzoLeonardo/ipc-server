mod error;
mod manager;
mod message;
mod server;
#[cfg(test)]
mod test;

use server::Server;
use tokio::sync::mpsc::unbounded_channel;

use crate::manager::TaskManager;

pub fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}]: {}",
                chrono::Local::now().format("%H:%M:%S%.9f"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    setup_logger().unwrap();

    let (tx, rx) = unbounded_channel();

    TaskManager::spawn(rx).await;
    Server::spawn(tx).await
}

mod error;
mod manager;
mod message;
mod server;
#[cfg(test)]
mod test;

use std::error::Error;

use ipc_client::ENV_LOGGER;
use server::Server;
use tokio::sync::mpsc::unbounded_channel;

use crate::manager::TaskManager;

pub fn setup_logger() {
    let level = std::env::var(ENV_LOGGER)
        .map(|var| match var.to_lowercase().as_str() {
            "trace" => log::LevelFilter::Trace,
            "debug" => log::LevelFilter::Debug,
            "info" => log::LevelFilter::Info,
            "warn" => log::LevelFilter::Warn,
            "error" => log::LevelFilter::Error,
            "off" => log::LevelFilter::Off,
            _ => log::LevelFilter::Info,
        })
        .unwrap_or_else(|_| log::LevelFilter::Info);

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}:{}]: {}",
                chrono::Local::now().format("%H:%M:%S%.9f"),
                record.level(),
                record.target(),
                record.line().unwrap_or(0),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()
        .unwrap_or_else(|e| {
            eprintln!("{:?}", e);
        });
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let version = env!("CARGO_PKG_VERSION");
    log::info!("Starting ipc-server v.{}", version);

    let (tx, rx) = unbounded_channel();

    TaskManager::spawn(rx).await;
    Server::spawn(tx).await;

    log::info!("Stopping ipc-server v.{}", version);
    Ok(())
}

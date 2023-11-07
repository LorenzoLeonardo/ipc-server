use std::{
    io::{Error, ErrorKind},
    sync::Arc,
    time::Duration,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

pub const DATA_STREAM_SIZE: usize = 4096;

#[derive(Clone, Debug)]
pub struct Socket {
    socket: Arc<Mutex<TcpStream>>,
    ip_address: String,
    chunk_size: usize,
}

impl Socket {
    pub fn new(socket: TcpStream) -> Result<Self, Error> {
        let ip_address = socket.peer_addr()?.to_string();
        Ok(Self {
            socket: Arc::new(Mutex::new(socket)),
            ip_address,
            chunk_size: DATA_STREAM_SIZE,
        })
    }

    pub fn ip_address(&self) -> &str {
        &self.ip_address
    }

    pub fn set_chunk_size(&mut self, size: usize) {
        self.chunk_size = size;
    }

    pub async fn read(&self, read_data: &mut Vec<u8>) -> Result<usize, Error> {
        let mut socket = self.socket.lock().await;
        loop {
            let mut chunk = vec![0; self.chunk_size];
            let n = match socket.read(&mut chunk).await {
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    tokio::time::sleep(Duration::from_nanos(1)).await;
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            };

            if n == 0 {
                return Err(Error::new(
                    ErrorKind::ConnectionAborted,
                    ErrorKind::ConnectionAborted.to_string(),
                ));
            }
            chunk.truncate(n);
            read_data.extend_from_slice(&chunk);
            let len = read_data.len();
            if read_data[len - 1] == b'\n' {
                read_data.truncate(len - 1);
                return Ok(len - 1);
            }
        }
    }

    pub async fn write_all(&self, write_data: &[u8]) -> Result<(), Error> {
        let mut socket = self.socket.lock().await;

        let mut stream = write_data.to_vec();
        stream.extend_from_slice("\n".as_bytes());
        socket.write_all(stream.as_slice()).await
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use tokio::net::{TcpListener, TcpStream};

    use crate::client::socket::Socket;

    #[test_case(r#"{"object":{"key","value-1234"}}"#.as_bytes(), 32, "127.0.0.1:8888";"less than chunk")]
    #[test_case(r#"{"object":{"key","value-1234"},"object":{"key","value-1234"}}"#.as_bytes(), 32, "127.0.0.1:8889";"more than chunk")]
    #[test_case(r#"{"object":{"key","value-12345"}}"#.as_bytes(), 32, "127.0.0.1:8890";"equal chunk")]
    #[test_case(r#""#.as_bytes(), 32, "127.0.0.1:8891";"no value")]
    #[tokio::test]
    async fn test_socket(expected_data: &[u8], chunk_size: usize, ip: &str) {
        let listener = TcpListener::bind(ip).await.unwrap();

        let expected = expected_data.to_vec();
        let server_handle = tokio::spawn(async move {
            let (socket, _s) = listener.accept().await.unwrap();
            let mut socket = Socket::new(socket).unwrap();
            socket.set_chunk_size(chunk_size);

            let mut read_data = Vec::new();
            let size = socket.read(&mut read_data).await.unwrap();
            assert_eq!(read_data.as_slice(), expected.as_slice());
            assert_eq!(size, expected.len());
        });

        let ip = ip.to_string();
        let given = expected_data.to_vec();
        let client = tokio::spawn(async move {
            let socket = TcpStream::connect(ip.as_str()).await.unwrap();
            let socket = Socket::new(socket).unwrap();

            socket.write_all(given.as_slice()).await.unwrap();
        });

        let _ = tokio::join!(server_handle, client);
    }
}

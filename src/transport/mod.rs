use crate::protocol::Message;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info};

/// Network transport for P2P communication
pub struct Transport {
    listener: Option<TcpListener>,
}

impl Transport {
    pub fn new() -> Self {
        Self { listener: None }
    }

    /// Start listening for incoming connections
    pub async fn listen(&mut self, address: &str, port: u16) -> Result<()> {
        let addr = format!("{}:{}", address, port).parse::<SocketAddr>()?;
        let listener = TcpListener::bind(&addr).await?;
        info!("Transport listening on {}", addr);
        self.listener = Some(listener);
        Ok(())
    }

    /// Get the listener
    pub fn get_listener(&self) -> Option<&TcpListener> {
        self.listener.as_ref()
    }

    /// Send a message to a peer
    pub async fn send_message(&self, peer_addr: SocketAddr, message: &Message) -> Result<()> {
        let mut stream = TcpStream::connect(peer_addr).await?;
        let serialized = bincode::serialize(message)?;
        
        // Send length prefix (4 bytes)
        stream.write_all(&(serialized.len() as u32).to_le_bytes()).await?;
        stream.write_all(&serialized).await?;
        stream.flush().await?;
        
        debug!("Sent message to {}", peer_addr);
        Ok(())
    }

    /// Receive a message from a stream
    pub async fn recv_message(stream: &mut TcpStream) -> Result<Message> {
        // Read length prefix (4 bytes)
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;

        // Read message data
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;

        let message = bincode::deserialize(&buf)?;
        Ok(message)
    }

    /// Handle incoming connection
    pub async fn handle_connection<F>(stream: TcpStream, handler: F) -> Result<()>
    where
        F: Fn(Message) -> futures::future::BoxFuture<'static, Result<()>> + 'static,
    {
        let mut stream = stream;
        let peer_addr = stream.peer_addr()?;
        debug!("New connection from {}", peer_addr);

        loop {
            match Self::recv_message(&mut stream).await {
                Ok(message) => {
                    handler(message).await?;
                }
                Err(e) => {
                    // Check if it's EOF/disconnection
                    if e.to_string().contains("unexpected end") || 
                       e.to_string().contains("connection") {
                        debug!("Connection closed by {}", peer_addr);
                        break;
                    } else {
                        error!("Error receiving message from {}: {}", peer_addr, e);
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for Transport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_new() {
        let transport = Transport::new();
        assert!(transport.listener.is_none());
    }
}

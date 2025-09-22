//! Network layer for reference-node
//! 
//! This module provides P2P networking, peer management, and Bitcoin protocol
//! message handling for communication with other Bitcoin nodes.

pub mod peer;
pub mod protocol;
pub mod inventory;
pub mod relay;

use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing::{info, warn, error};

/// Network manager that coordinates all network operations
pub struct NetworkManager {
    listen_addr: SocketAddr,
    peers: HashMap<SocketAddr, peer::Peer>,
    peer_tx: mpsc::UnboundedSender<NetworkMessage>,
    peer_rx: mpsc::UnboundedReceiver<NetworkMessage>,
}

/// Network message types
#[derive(Debug, Clone)]
pub enum NetworkMessage {
    PeerConnected(SocketAddr),
    PeerDisconnected(SocketAddr),
    BlockReceived(Vec<u8>),
    TransactionReceived(Vec<u8>),
    InventoryReceived(Vec<u8>),
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(listen_addr: SocketAddr) -> Self {
        let (peer_tx, peer_rx) = mpsc::unbounded_channel();
        
        Self {
            listen_addr,
            peers: HashMap::new(),
            peer_tx,
            peer_rx,
        }
    }
    
    /// Start the network manager
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting network manager on {}", self.listen_addr);
        
        let listener = TcpListener::bind(self.listen_addr).await?;
        info!("Listening for connections on {}", self.listen_addr);
        
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from {}", addr);
                    self.handle_new_connection(stream, addr).await?;
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
    
    /// Handle a new peer connection
    async fn handle_new_connection(
        &mut self,
        stream: tokio::net::TcpStream,
        addr: SocketAddr,
    ) -> Result<()> {
        let peer = peer::Peer::new(stream, addr, self.peer_tx.clone());
        self.peers.insert(addr, peer);
        
        // Start peer handler
        tokio::spawn(async move {
            // Peer handling logic will be implemented
        });
        
        Ok(())
    }
    
    /// Get the number of connected peers
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }
    
    /// Get all peer addresses
    pub fn peer_addresses(&self) -> Vec<SocketAddr> {
        self.peers.keys().cloned().collect()
    }
    
    /// Broadcast a message to all peers
    pub async fn broadcast(&self, message: Vec<u8>) -> Result<()> {
        for (addr, peer) in &self.peers {
            if let Err(e) = peer.send_message(message.clone()).await {
                warn!("Failed to send message to peer {}: {}", addr, e);
            }
        }
        Ok(())
    }
    
    /// Send a message to a specific peer
    pub async fn send_to_peer(&self, addr: SocketAddr, message: Vec<u8>) -> Result<()> {
        if let Some(peer) = self.peers.get(&addr) {
            peer.send_message(message).await?;
        }
        Ok(())
    }
    
    /// Process incoming network messages
    pub async fn process_messages(&mut self) -> Result<()> {
        while let Some(message) = self.peer_rx.recv().await {
            match message {
                NetworkMessage::PeerConnected(addr) => {
                    info!("Peer connected: {}", addr);
                }
                NetworkMessage::PeerDisconnected(addr) => {
                    info!("Peer disconnected: {}", addr);
                    self.peers.remove(&addr);
                }
                NetworkMessage::BlockReceived(data) => {
                    info!("Block received: {} bytes", data.len());
                    // Process block with consensus layer
                }
                NetworkMessage::TransactionReceived(data) => {
                    info!("Transaction received: {} bytes", data.len());
                    // Process transaction with consensus layer
                }
                NetworkMessage::InventoryReceived(data) => {
                    info!("Inventory received: {} bytes", data.len());
                    // Process inventory
                }
            }
        }
        Ok(())
    }
}
//! RPC interface for reference-node
//! 
//! This module provides JSON-RPC server, blockchain query methods,
//! network info methods, transaction submission, and mining methods.

pub mod server;
pub mod blockchain;
pub mod network;
pub mod mining;
pub mod types;

use anyhow::Result;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tracing::{info, error};

/// RPC manager that coordinates all RPC operations
pub struct RpcManager {
    server_addr: SocketAddr,
    blockchain_rpc: blockchain::BlockchainRpc,
    network_rpc: network::NetworkRpc,
    mining_rpc: mining::MiningRpc,
    shutdown_tx: Option<mpsc::UnboundedSender<()>>,
}

impl RpcManager {
    /// Create a new RPC manager
    pub fn new(server_addr: SocketAddr) -> Self {
        Self {
            server_addr,
            blockchain_rpc: blockchain::BlockchainRpc::new(),
            network_rpc: network::NetworkRpc::new(),
            mining_rpc: mining::MiningRpc::new(),
            shutdown_tx: None,
        }
    }
    
    /// Start the RPC server
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting RPC server on {}", self.server_addr);
        
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        self.shutdown_tx = Some(shutdown_tx);
        
        let server = server::RpcServer::new(self.server_addr);
        
        // Start the server in a background task
        let server_handle = tokio::spawn(async move {
            if let Err(e) = server.start().await {
                error!("RPC server error: {}", e);
            }
        });
        
        // Wait for shutdown signal
        shutdown_rx.recv().await;
        
        // Shutdown the server
        server_handle.abort();
        info!("RPC server stopped");
        
        Ok(())
    }
    
    /// Stop the RPC server
    pub fn stop(&self) -> Result<()> {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
        Ok(())
    }
    
    /// Get blockchain RPC methods
    pub fn blockchain(&self) -> &blockchain::BlockchainRpc {
        &self.blockchain_rpc
    }
    
    /// Get network RPC methods
    pub fn network(&self) -> &network::NetworkRpc {
        &self.network_rpc
    }
    
    /// Get mining RPC methods
    pub fn mining(&self) -> &mining::MiningRpc {
        &self.mining_rpc
    }
}
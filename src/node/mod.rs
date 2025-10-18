//! Node orchestration for reference-node
//! 
//! This module provides sync coordination, mempool management,
//! mining coordination, and overall node state management.

pub mod sync;
pub mod mempool;
pub mod miner;

use anyhow::Result;
use std::net::SocketAddr;
use tracing::{info, warn};

use protocol_engine::{BitcoinProtocolEngine, ProtocolVersion};
use crate::storage::Storage;
use crate::network::NetworkManager;
use crate::rpc::RpcManager;

/// Main node orchestrator
pub struct Node {
    protocol: BitcoinProtocolEngine,
    storage: Storage,
    network: NetworkManager,
    rpc: RpcManager,
    #[allow(dead_code)]
    sync_coordinator: sync::SyncCoordinator,
    #[allow(dead_code)]
    mempool_manager: mempool::MempoolManager,
    #[allow(dead_code)]
    mining_coordinator: miner::MiningCoordinator,
}

impl Node {
    /// Create a new node
    pub fn new(
        data_dir: &str,
        network_addr: SocketAddr,
        rpc_addr: SocketAddr,
        protocol_version: Option<ProtocolVersion>,
    ) -> Result<Self> {
        info!("Initializing reference-node");
        
        // Initialize components
        let protocol = BitcoinProtocolEngine::new(
            protocol_version.unwrap_or(ProtocolVersion::Regtest)
        )?;
        let storage = Storage::new(data_dir)?;
        let network = NetworkManager::new(network_addr);
        let rpc = RpcManager::new(rpc_addr);
        let sync_coordinator = sync::SyncCoordinator::default();
        let mempool_manager = mempool::MempoolManager::new();
        let mining_coordinator = miner::MiningCoordinator::default();
        
        Ok(Self {
            protocol,
            storage,
            network,
            rpc,
            sync_coordinator,
            mempool_manager,
            mining_coordinator,
        })
    }
    
    /// Start the node
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting reference-node");
        
        // Start all components
        self.start_components().await?;
        
        // Main node loop
        self.run().await?;
        
        Ok(())
    }
    
    /// Start all node components
    async fn start_components(&mut self) -> Result<()> {
        info!("Starting node components");
        
        // Simplified component startup
        // In a real implementation, each component would be started in separate tasks
        // For now, we'll just initialize them
        
        info!("RPC server initialized");
        info!("Network manager initialized");
        info!("Sync coordinator initialized");
        info!("Mempool manager initialized");
        info!("Mining coordinator initialized");
        
        Ok(())
    }
    
    /// Main node run loop
    async fn run(&mut self) -> Result<()> {
        info!("Node running - main loop started");
        
        // Main node loop - in a real implementation this would coordinate
        // between all components and handle shutdown signals
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            // Check node health
            self.check_health().await?;
        }
    }
    
    /// Run node processing once (for testing)
    pub async fn run_once(&mut self) -> Result<()> {
        info!("Running node processing once");
        
        // Check node health
        self.check_health().await?;
        
        Ok(())
    }
    
    /// Check node health
    async fn check_health(&self) -> Result<()> {
        // Simplified health check
        let peer_count = self.network.peer_count();
        let storage_blocks = self.storage.blocks().block_count()?;
        
        if peer_count == 0 {
            warn!("No peers connected");
        }
        
        if storage_blocks == 0 {
            warn!("No blocks in storage");
        }
        
        Ok(())
    }
    
    /// Stop the node
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping reference-node");
        
        // Stop all components
        self.rpc.stop()?;
        
        // Flush storage
        self.storage.flush()?;
        
        info!("Node stopped");
        Ok(())
    }
    
    /// Get protocol engine
    pub fn protocol(&self) -> &BitcoinProtocolEngine {
        &self.protocol
    }
    
    /// Get storage
    pub fn storage(&self) -> &Storage {
        &self.storage
    }
    
    /// Get network manager
    pub fn network(&self) -> &NetworkManager {
        &self.network
    }
    
    /// Get RPC manager
    pub fn rpc(&self) -> &RpcManager {
        &self.rpc
    }
}
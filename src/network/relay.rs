//! Block and transaction relay
//! 
//! Handles relaying blocks and transactions to peers, managing relay policies,
//! and preventing duplicate relay.

use consensus_proof::Hash;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

/// Relay manager
pub struct RelayManager {
    /// Recently relayed blocks
    recently_relayed_blocks: HashMap<Hash, u64>,
    /// Recently relayed transactions
    recently_relayed_txs: HashMap<Hash, u64>,
    /// Relay policies
    policies: RelayPolicies,
}

/// Relay policies
#[derive(Debug, Clone)]
pub struct RelayPolicies {
    /// Maximum age for relayed items (seconds)
    pub max_relay_age: u64,
    /// Maximum number of items to track
    pub max_tracked_items: usize,
    /// Enable block relay
    pub enable_block_relay: bool,
    /// Enable transaction relay
    pub enable_tx_relay: bool,
}

impl Default for RelayPolicies {
    fn default() -> Self {
        Self {
            max_relay_age: 3600, // 1 hour
            max_tracked_items: 10000,
            enable_block_relay: true,
            enable_tx_relay: true,
        }
    }
}

impl Default for RelayManager {
    fn default() -> Self { Self::new() }
}

impl RelayManager {
    /// Create a new relay manager
    pub fn new() -> Self {
        Self {
            recently_relayed_blocks: HashMap::new(),
            recently_relayed_txs: HashMap::new(),
            policies: RelayPolicies::default(),
        }
    }
    
    /// Create a relay manager with custom policies
    pub fn with_policies(policies: RelayPolicies) -> Self {
        Self {
            recently_relayed_blocks: HashMap::new(),
            recently_relayed_txs: HashMap::new(),
            policies,
        }
    }
    
    /// Check if a block should be relayed
    pub fn should_relay_block(&self, block_hash: &Hash) -> bool {
        if !self.policies.enable_block_relay {
            return false;
        }
        
        if self.recently_relayed_blocks.contains_key(block_hash) {
            return false;
        }
        
        true
    }
    
    /// Check if a transaction should be relayed
    pub fn should_relay_transaction(&self, tx_hash: &Hash) -> bool {
        if !self.policies.enable_tx_relay {
            return false;
        }
        
        if self.recently_relayed_txs.contains_key(tx_hash) {
            return false;
        }
        
        true
    }
    
    /// Mark a block as relayed
    pub fn mark_block_relayed(&mut self, block_hash: Hash) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.recently_relayed_blocks.insert(block_hash, now);
        self.cleanup_old_items();
        
        debug!("Marked block {} as relayed", hex::encode(block_hash));
    }
    
    /// Mark a transaction as relayed
    pub fn mark_transaction_relayed(&mut self, tx_hash: Hash) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.recently_relayed_txs.insert(tx_hash, now);
        self.cleanup_old_items();
        
        debug!("Marked transaction {} as relayed", hex::encode(tx_hash));
    }
    
    /// Get relay statistics
    pub fn get_stats(&self) -> RelayStats {
        RelayStats {
            relayed_blocks: self.recently_relayed_blocks.len(),
            relayed_transactions: self.recently_relayed_txs.len(),
            policies: self.policies.clone(),
        }
    }
    
    /// Clean up old relayed items
    fn cleanup_old_items(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Clean up old blocks
        let old_blocks: Vec<Hash> = self.recently_relayed_blocks
            .iter()
            .filter(|(_, &timestamp)| now - timestamp > self.policies.max_relay_age)
            .map(|(hash, _)| *hash)
            .collect();
        
        for hash in old_blocks {
            self.recently_relayed_blocks.remove(&hash);
        }
        
        // Clean up old transactions
        let old_txs: Vec<Hash> = self.recently_relayed_txs
            .iter()
            .filter(|(_, &timestamp)| now - timestamp > self.policies.max_relay_age)
            .map(|(hash, _)| *hash)
            .collect();
        
        for hash in old_txs {
            self.recently_relayed_txs.remove(&hash);
        }
        
        // Limit total items
        if self.recently_relayed_blocks.len() > self.policies.max_tracked_items {
            let excess = self.recently_relayed_blocks.len() - self.policies.max_tracked_items;
            let oldest_blocks: Vec<Hash> = self.recently_relayed_blocks
                .iter()
                .min_by_key(|(_, &timestamp)| timestamp)
                .map(|(hash, _)| *hash)
                .into_iter()
                .take(excess)
                .collect();
            
            for hash in oldest_blocks {
                self.recently_relayed_blocks.remove(&hash);
            }
        }
        
        if self.recently_relayed_txs.len() > self.policies.max_tracked_items {
            let excess = self.recently_relayed_txs.len() - self.policies.max_tracked_items;
            let oldest_txs: Vec<Hash> = self.recently_relayed_txs
                .iter()
                .min_by_key(|(_, &timestamp)| timestamp)
                .map(|(hash, _)| *hash)
                .into_iter()
                .take(excess)
                .collect();
            
            for hash in oldest_txs {
                self.recently_relayed_txs.remove(&hash);
            }
        }
    }
}

/// Relay statistics
#[derive(Debug, Clone)]
pub struct RelayStats {
    pub relayed_blocks: usize,
    pub relayed_transactions: usize,
    pub policies: RelayPolicies,
}

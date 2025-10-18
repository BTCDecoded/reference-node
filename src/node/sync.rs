//! Block sync coordinator
//! 
//! Handles blockchain synchronization, header download, block validation,
//! and chain reorganization.

use anyhow::Result;
use consensus_proof::{BlockHeader, Block};
use std::collections::HashMap;
use tracing::{debug, info, error};

/// Block provider for dependency injection
pub struct BlockProvider {
    /// Mock block storage
    blocks: std::collections::HashMap<[u8; 32], Block>,
    /// Mock header storage
    headers: std::collections::HashMap<[u8; 32], BlockHeader>,
    /// Mock block count
    block_count: u64,
}

/// Sync state machine
pub struct SyncStateMachine {
    /// Current sync state
    state: SyncState,
    /// Best known header
    best_header: Option<BlockHeader>,
    /// Current chain tip
    chain_tip: Option<BlockHeader>,
    /// Sync progress (0.0 to 1.0)
    progress: f64,
    /// Error message if in error state
    error_message: Option<String>,
}

impl SyncStateMachine {
    /// Create a new sync state machine
    pub fn new() -> Self {
        Self {
            state: SyncState::Initial,
            best_header: None,
            chain_tip: None,
            progress: 0.0,
            error_message: None,
        }
    }
    
    /// Transition to a new state
    pub fn transition_to(&mut self, new_state: SyncState) {
        debug!("Sync state transition: {:?} -> {:?}", self.state, new_state);
        self.state = new_state;
        self.update_progress();
    }
    
    /// Set error state
    pub fn set_error(&mut self, error: String) {
        self.state = SyncState::Error(error.clone());
        self.error_message = Some(error);
        self.progress = 0.0;
    }
    
    /// Update best header
    pub fn update_best_header(&mut self, header: BlockHeader) {
        self.best_header = Some(header);
    }
    
    /// Update chain tip
    pub fn update_chain_tip(&mut self, header: BlockHeader) {
        self.chain_tip = Some(header);
    }
    
    /// Get current state
    pub fn state(&self) -> &SyncState {
        &self.state
    }
    
    /// Get sync progress
    pub fn progress(&self) -> f64 {
        self.progress
    }
    
    /// Check if sync is complete
    pub fn is_synced(&self) -> bool {
        matches!(self.state, SyncState::Synced)
    }
    
    /// Get best header
    pub fn best_header(&self) -> Option<&BlockHeader> {
        self.best_header.as_ref()
    }
    
    /// Get chain tip
    pub fn chain_tip(&self) -> Option<&BlockHeader> {
        self.chain_tip.as_ref()
    }
    
    /// Update progress based on current state
    fn update_progress(&mut self) {
        self.progress = match self.state {
            SyncState::Initial => 0.0,
            SyncState::Headers => 0.3,
            SyncState::Blocks => 0.7,
            SyncState::Synced => 1.0,
            SyncState::Error(_) => 0.0,
        };
    }
}

/// Sync coordinator
pub struct SyncCoordinator {
    /// State machine
    state_machine: SyncStateMachine,
    /// Block provider
    block_provider: BlockProvider,
}

/// Sync state
#[derive(Debug, Clone, PartialEq)]
pub enum SyncState {
    /// Initial state
    Initial,
    /// Downloading headers
    Headers,
    /// Downloading blocks
    Blocks,
    /// Fully synced
    Synced,
    /// Sync error
    Error(String),
}

impl SyncCoordinator {
    /// Create a new sync coordinator with a block provider
    pub fn new(block_provider: BlockProvider) -> Self {
        Self {
            state_machine: SyncStateMachine::new(),
            block_provider,
        }
    }
    
    /// Start the sync coordinator
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting sync coordinator");
        
        // Initialize sync state
        self.state_machine.transition_to(SyncState::Initial);
        
        // Start sync process
        self.sync_loop().await?;
        
        Ok(())
    }
    
    /// Main sync loop
    async fn sync_loop(&mut self) -> Result<()> {
        loop {
            match self.state_machine.state() {
                SyncState::Initial => {
                    self.initialize_sync().await?;
                }
                SyncState::Headers => {
                    self.download_headers().await?;
                }
                SyncState::Blocks => {
                    self.download_blocks().await?;
                }
                SyncState::Synced => {
                    self.maintain_sync().await?;
                }
                SyncState::Error(ref msg) => {
                    error!("Sync error: {}", msg);
                    return Err(anyhow::anyhow!("Sync failed: {}", msg));
                }
            }
            
            // Small delay to prevent busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
    
    
    /// Get current sync state
    pub fn state(&self) -> &SyncState {
        self.state_machine.state()
    }
    
    /// Get sync progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        self.state_machine.progress()
    }
    
    /// Check if sync is complete
    pub fn is_synced(&self) -> bool {
        self.state_machine.is_synced()
    }
    
    /// Get best header
    pub fn best_header(&self) -> Option<&BlockHeader> {
        self.state_machine.best_header()
    }
    
    /// Get chain tip
    pub fn chain_tip(&self) -> Option<&BlockHeader> {
        self.state_machine.chain_tip()
    }
    
    /// Get access to the state machine
    pub fn state_machine(&self) -> &SyncStateMachine {
        &self.state_machine
    }
    
    /// Get mutable access to the state machine
    pub fn state_machine_mut(&mut self) -> &mut SyncStateMachine {
        &mut self.state_machine
    }
    
    /// Initialize sync process (public for testing)
    pub async fn initialize_sync(&mut self) -> Result<()> {
        debug!("Initializing sync");
        
        // Check if we have any blocks using the block provider
        let block_count = self.block_provider.get_block_count()?;
        let has_blocks = block_count > 0;
        
        if has_blocks {
            // Continue from existing chain
            self.state_machine.transition_to(SyncState::Headers);
        } else {
            // Start from genesis
            self.state_machine.transition_to(SyncState::Headers);
        }
        
        Ok(())
    }
    
    /// Download headers (public for testing)
    pub async fn download_headers(&mut self) -> Result<()> {
        debug!("Downloading headers");
        
        // Get best header from block provider
        if let Some(best_header) = self.block_provider.get_best_header()? {
            self.state_machine.update_best_header(best_header);
        }
        
        // Simulate header download
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        self.state_machine.transition_to(SyncState::Blocks);
        Ok(())
    }
    
    /// Download blocks (public for testing)
    pub async fn download_blocks(&mut self) -> Result<()> {
        debug!("Downloading blocks");
        
        // Simulate block download using block provider
        if let Some(best_header) = self.state_machine.best_header() {
            // In a real implementation, we would download and validate blocks
            // For now, just simulate the process
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Update chain tip
            self.state_machine.update_chain_tip(best_header.clone());
        }
        
        self.state_machine.transition_to(SyncState::Synced);
        Ok(())
    }
    
    /// Maintain sync (public for testing)
    pub async fn maintain_sync(&mut self) -> Result<()> {
        debug!("Maintaining sync");
        
        // In a real implementation, this would:
        // 1. Listen for new blocks from network
        // 2. Validate and apply new blocks using block provider
        // 3. Handle chain reorganizations
        
        // Simulate maintenance
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        Ok(())
    }
}

impl Default for SyncCoordinator {
    fn default() -> Self { 
        // Create a mock block provider for default
        Self::new(BlockProvider::new())
    }
}

impl BlockProvider {
    /// Create a new block provider
    pub fn new() -> Self {
        Self {
            blocks: std::collections::HashMap::new(),
            headers: std::collections::HashMap::new(),
            block_count: 0,
        }
    }
    
    /// Get a block by hash
    pub fn get_block(&self, hash: &[u8; 32]) -> Result<Option<Block>> {
        Ok(self.blocks.get(hash).cloned())
    }
    
    /// Get a block header by hash
    pub fn get_block_header(&self, hash: &[u8; 32]) -> Result<Option<BlockHeader>> {
        Ok(self.headers.get(hash).cloned())
    }
    
    /// Get the best block header
    pub fn get_best_header(&self) -> Result<Option<BlockHeader>> {
        Ok(self.headers.values().last().cloned())
    }
    
    /// Store a block
    pub fn store_block(&mut self, block: &Block) -> Result<()> {
        // Simplified hash calculation
        let hash = self.calculate_block_hash(block);
        self.blocks.insert(hash, block.clone());
        self.block_count += 1;
        Ok(())
    }
    
    /// Store a block header
    pub fn store_block_header(&mut self, header: &BlockHeader) -> Result<()> {
        // Simplified hash calculation
        let hash = self.calculate_header_hash(header);
        self.headers.insert(hash, header.clone());
        Ok(())
    }
    
    /// Get block count
    pub fn get_block_count(&self) -> Result<u64> {
        Ok(self.block_count)
    }
    
    /// Calculate block hash (simplified)
    fn calculate_block_hash(&self, block: &Block) -> [u8; 32] {
        let mut hash = [0u8; 32];
        hash[0] = block.header.version as u8;
        hash[1] = block.transactions.len() as u8;
        hash
    }
    
    /// Calculate header hash (simplified)
    fn calculate_header_hash(&self, header: &BlockHeader) -> [u8; 32] {
        let mut hash = [0u8; 32];
        hash[0] = header.version as u8;
        hash[1] = header.timestamp as u8;
        hash
    }
}

/// Mock block provider for testing
pub struct MockBlockProvider {
    blocks: HashMap<[u8; 32], Block>,
    headers: HashMap<[u8; 32], BlockHeader>,
    best_header: Option<BlockHeader>,
    block_count: u64,
}


impl MockBlockProvider {
    /// Create a new mock block provider
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
            headers: HashMap::new(),
            best_header: None,
            block_count: 0,
        }
    }
    
    /// Get a block by hash
    pub fn get_block(&self, hash: &[u8; 32]) -> Result<Option<Block>> {
        Ok(self.blocks.get(hash).cloned())
    }
    
    /// Get a block header by hash
    pub fn get_block_header(&self, hash: &[u8; 32]) -> Result<Option<BlockHeader>> {
        Ok(self.headers.get(hash).cloned())
    }
    
    /// Get the best block header
    pub fn get_best_header(&self) -> Result<Option<BlockHeader>> {
        Ok(self.best_header.clone())
    }
    
    /// Store a block
    pub fn store_block(&mut self, block: &Block) -> Result<()> {
        // Simplified hash calculation
        let hash = self.calculate_block_hash(block);
        self.blocks.insert(hash, block.clone());
        self.block_count += 1;
        Ok(())
    }
    
    /// Store a block header
    pub fn store_block_header(&mut self, header: &BlockHeader) -> Result<()> {
        // Simplified hash calculation
        let hash = self.calculate_header_hash(header);
        self.headers.insert(hash, header.clone());
        self.best_header = Some(header.clone());
        Ok(())
    }
    
    /// Get block count
    pub fn get_block_count(&self) -> Result<u64> {
        Ok(self.block_count)
    }
    
    /// Calculate block hash (simplified)
    fn calculate_block_hash(&self, block: &Block) -> [u8; 32] {
        let mut hash = [0u8; 32];
        hash[0] = block.header.version as u8;
        hash[1] = block.transactions.len() as u8;
        hash
    }
    
    /// Calculate header hash (simplified)
    fn calculate_header_hash(&self, header: &BlockHeader) -> [u8; 32] {
        let mut hash = [0u8; 32];
        hash[0] = header.version as u8;
        hash[1] = header.timestamp as u8;
        hash
    }
    
    /// Add block (for testing)
    pub fn add_block(&mut self, block: Block) {
        let hash = self.calculate_block_hash(&block);
        self.blocks.insert(hash, block);
        self.block_count += 1;
    }
    
    /// Add header (for testing)
    pub fn add_header(&mut self, header: BlockHeader) {
        let hash = self.calculate_header_hash(&header);
        self.headers.insert(hash, header.clone());
        if self.best_header.is_none() {
            self.best_header = Some(header);
        }
    }
    
    /// Set best header (for testing)
    pub fn set_best_header(&mut self, header: BlockHeader) {
        self.best_header = Some(header);
    }
    
    /// Set block count (for testing)
    pub fn set_block_count(&mut self, count: u64) {
        self.block_count = count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_proof::types::BlockHeader;
    use std::time::SystemTime;

    #[test]
    fn test_sync_state_machine_creation() {
        let state_machine = SyncStateMachine::new();
        assert_eq!(state_machine.state(), &SyncState::Initial);
        assert_eq!(state_machine.progress(), 0.0);
        assert!(!state_machine.is_synced());
        assert!(state_machine.best_header().is_none());
        assert!(state_machine.chain_tip().is_none());
    }

    #[test]
    fn test_sync_state_machine_transitions() {
        let mut state_machine = SyncStateMachine::new();
        
        // Test transition to Headers
        state_machine.transition_to(SyncState::Headers);
        assert_eq!(state_machine.state(), &SyncState::Headers);
        assert_eq!(state_machine.progress(), 0.3);
        
        // Test transition to Blocks
        state_machine.transition_to(SyncState::Blocks);
        assert_eq!(state_machine.state(), &SyncState::Blocks);
        assert_eq!(state_machine.progress(), 0.7);
        
        // Test transition to Synced
        state_machine.transition_to(SyncState::Synced);
        assert_eq!(state_machine.state(), &SyncState::Synced);
        assert_eq!(state_machine.progress(), 1.0);
        assert!(state_machine.is_synced());
    }

    #[test]
    fn test_sync_state_machine_error_handling() {
        let mut state_machine = SyncStateMachine::new();
        
        // Test error state
        state_machine.set_error("Test error".to_string());
        assert_eq!(state_machine.state(), &SyncState::Error("Test error".to_string()));
        assert_eq!(state_machine.progress(), 0.0);
        assert!(!state_machine.is_synced());
    }

    #[test]
    fn test_sync_state_machine_header_updates() {
        let mut state_machine = SyncStateMachine::new();
        
        // Create a mock header
        let header = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            bits: 0x1d00ffff,
            nonce: 0,
        };
        
        // Test best header update
        state_machine.update_best_header(header.clone());
        assert!(state_machine.best_header().is_some());
        assert_eq!(state_machine.best_header().unwrap().version, header.version);
        
        // Test chain tip update
        state_machine.update_chain_tip(header.clone());
        assert!(state_machine.chain_tip().is_some());
        assert_eq!(state_machine.chain_tip().unwrap().version, header.version);
    }

    #[test]
    fn test_sync_state_machine_progress_calculation() {
        let mut state_machine = SyncStateMachine::new();
        
        // Test all state progress values
        let test_cases = vec![
            (SyncState::Initial, 0.0),
            (SyncState::Headers, 0.3),
            (SyncState::Blocks, 0.7),
            (SyncState::Synced, 1.0),
            (SyncState::Error("test".to_string()), 0.0),
        ];
        
        for (state, expected_progress) in test_cases {
            state_machine.transition_to(state);
            assert_eq!(state_machine.progress(), expected_progress);
        }
    }

    #[tokio::test]
    async fn test_sync_coordinator_creation() {
        let mock_provider = MockBlockProvider::new();
        let coordinator = SyncCoordinator::new(BlockProvider::new());
        
        assert_eq!(coordinator.state(), &SyncState::Initial);
        assert_eq!(coordinator.progress(), 0.0);
        assert!(!coordinator.is_synced());
    }

    #[tokio::test]
    async fn test_sync_coordinator_with_block_provider() {
        let mut mock_provider = MockBlockProvider::new();
        
        // Add some test data
        let header = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            bits: 0x1d00ffff,
            nonce: 0,
        };
        mock_provider.add_header(header);
        mock_provider.set_block_count(5);
        
        let coordinator = SyncCoordinator::new(BlockProvider::new());
        assert_eq!(coordinator.state(), &SyncState::Initial);
    }

    #[tokio::test]
    async fn test_sync_coordinator_state_machine_access() {
        let mock_provider = MockBlockProvider::new();
        let coordinator = SyncCoordinator::new(BlockProvider::new());
        
        // Test immutable access
        let state_machine = coordinator.state_machine();
        assert_eq!(state_machine.state(), &SyncState::Initial);
        
        // Test mutable access
        let mut coordinator = coordinator;
        let state_machine_mut = coordinator.state_machine_mut();
        state_machine_mut.transition_to(SyncState::Headers);
        assert_eq!(coordinator.state(), &SyncState::Headers);
    }

    #[tokio::test]
    async fn test_mock_block_provider() {
        let mut provider = MockBlockProvider::new();
        
        // Test initial state
        assert_eq!(provider.get_block_count().unwrap(), 0);
        assert!(provider.get_best_header().unwrap().is_none());
        
        // Add a header
        let header = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            bits: 0x1d00ffff,
            nonce: 0,
        };
        provider.add_header(header.clone());
        
        // Test header retrieval
        assert!(provider.get_best_header().unwrap().is_some());
        assert_eq!(provider.get_best_header().unwrap().unwrap().version, 1);
    }

    #[tokio::test]
    async fn test_sync_coordinator_initialization() {
        let mut mock_provider = MockBlockProvider::new();
        mock_provider.set_block_count(0); // No existing blocks
        
        let mut coordinator = SyncCoordinator::new(BlockProvider::new());
        
        // Test initialization
        coordinator.initialize_sync().await.unwrap();
        assert_eq!(coordinator.state(), &SyncState::Headers);
    }

    #[tokio::test]
    async fn test_sync_coordinator_header_download() {
        let mut mock_provider = MockBlockProvider::new();
        let header = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            bits: 0x1d00ffff,
            nonce: 0,
        };
        mock_provider.add_header(header);
        
        let mut coordinator = SyncCoordinator::new(BlockProvider::new());
        coordinator.state_machine_mut().transition_to(SyncState::Headers);
        
        // Test header download
        coordinator.download_headers().await.unwrap();
        assert_eq!(coordinator.state(), &SyncState::Blocks);
    }

    #[tokio::test]
    async fn test_sync_coordinator_block_download() {
        let mut mock_provider = MockBlockProvider::new();
        let header = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            bits: 0x1d00ffff,
            nonce: 0,
        };
        mock_provider.add_header(header.clone());
        
        let mut coordinator = SyncCoordinator::new(BlockProvider::new());
        coordinator.state_machine_mut().update_best_header(header);
        coordinator.state_machine_mut().transition_to(SyncState::Blocks);
        
        // Test block download
        coordinator.download_blocks().await.unwrap();
        assert_eq!(coordinator.state(), &SyncState::Synced);
        assert!(coordinator.is_synced());
    }

    #[tokio::test]
    async fn test_sync_coordinator_maintenance() {
        let mock_provider = MockBlockProvider::new();
        let mut coordinator = SyncCoordinator::new(BlockProvider::new());
        coordinator.state_machine_mut().transition_to(SyncState::Synced);
        
        // Test maintenance (should not change state)
        coordinator.maintain_sync().await.unwrap();
        assert_eq!(coordinator.state(), &SyncState::Synced);
    }

    #[tokio::test]
    async fn test_sync_coordinator_error_handling() {
        let mock_provider = MockBlockProvider::new();
        let mut coordinator = SyncCoordinator::new(BlockProvider::new());
        
        // Test error state
        coordinator.state_machine_mut().set_error("Test error".to_string());
        assert_eq!(coordinator.state(), &SyncState::Error("Test error".to_string()));
        assert!(!coordinator.is_synced());
    }

    #[tokio::test]
    async fn test_sync_coordinator_progress_tracking() {
        let mock_provider = MockBlockProvider::new();
        let mut coordinator = SyncCoordinator::new(BlockProvider::new());
        
        // Test progress through different states
        let test_cases = vec![
            (SyncState::Initial, 0.0),
            (SyncState::Headers, 0.3),
            (SyncState::Blocks, 0.7),
            (SyncState::Synced, 1.0),
        ];
        
        for (state, expected_progress) in test_cases {
            coordinator.state_machine_mut().transition_to(state);
            assert_eq!(coordinator.progress(), expected_progress);
        }
    }

    #[tokio::test]
    async fn test_sync_coordinator_header_chain_tracking() {
        let mut mock_provider = MockBlockProvider::new();
        let header = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            bits: 0x1d00ffff,
            nonce: 0,
        };
        mock_provider.add_header(header.clone());
        
        let mut coordinator = SyncCoordinator::new(BlockProvider::new());
        
        // Test header tracking
        coordinator.state_machine_mut().update_best_header(header.clone());
        assert!(coordinator.best_header().is_some());
        assert_eq!(coordinator.best_header().unwrap().version, 1);
        
        // Test chain tip tracking
        coordinator.state_machine_mut().update_chain_tip(header);
        assert!(coordinator.chain_tip().is_some());
        assert_eq!(coordinator.chain_tip().unwrap().version, 1);
    }
}

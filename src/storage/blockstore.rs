//! Block storage implementation
//! 
//! Stores blocks by hash and maintains block index by height.

use anyhow::Result;
use consensus_proof::{Block, BlockHeader, Hash};
use sled::Db;

/// Block storage manager
pub struct BlockStore {
    #[allow(dead_code)]
    db: Db,
    blocks: sled::Tree,
    headers: sled::Tree,
    height_index: sled::Tree,
}

impl BlockStore {
    /// Create a new block store
    pub fn new(db: Db) -> Result<Self> {
        let blocks = db.open_tree("blocks")?;
        let headers = db.open_tree("headers")?;
        let height_index = db.open_tree("height_index")?;
        
        Ok(Self {
            db,
            blocks,
            headers,
            height_index,
        })
    }
    
    /// Store a block
    pub fn store_block(&self, block: &Block) -> Result<()> {
        let block_hash = self.block_hash(block);
        let block_data = bincode::serialize(block)?;
        
        self.blocks.insert(block_hash.as_slice(), block_data)?;
        self.headers.insert(block_hash.as_slice(), bincode::serialize(&block.header)?)?;
        
        Ok(())
    }
    
    /// Get a block by hash
    pub fn get_block(&self, hash: &Hash) -> Result<Option<Block>> {
        if let Some(data) = self.blocks.get(hash.as_slice())? {
            let block: Block = bincode::deserialize(&data)?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }
    
    /// Get a block header by hash
    pub fn get_header(&self, hash: &Hash) -> Result<Option<BlockHeader>> {
        if let Some(data) = self.headers.get(hash.as_slice())? {
            let header: BlockHeader = bincode::deserialize(&data)?;
            Ok(Some(header))
        } else {
            Ok(None)
        }
    }
    
    /// Store block height index
    pub fn store_height(&self, height: u64, hash: &Hash) -> Result<()> {
        let height_bytes = height.to_be_bytes();
        self.height_index.insert(height_bytes, hash.as_slice())?;
        Ok(())
    }
    
    /// Get block hash by height
    pub fn get_hash_by_height(&self, height: u64) -> Result<Option<Hash>> {
        let height_bytes = height.to_be_bytes();
        if let Some(data) = self.height_index.get(height_bytes)? {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&data);
            Ok(Some(hash))
        } else {
            Ok(None)
        }
    }
    
    /// Get all blocks in a height range
    pub fn get_blocks_by_height_range(&self, start: u64, end: u64) -> Result<Vec<Block>> {
        let mut blocks = Vec::new();
        
        for height in start..=end {
            if let Some(hash) = self.get_hash_by_height(height)? {
                if let Some(block) = self.get_block(&hash)? {
                    blocks.push(block);
                }
            }
        }
        
        Ok(blocks)
    }
    
    /// Check if a block exists
    pub fn has_block(&self, hash: &Hash) -> Result<bool> {
        Ok(self.blocks.contains_key(hash.as_slice())?)
    }
    
    /// Get total number of blocks stored
    pub fn block_count(&self) -> Result<usize> {
        Ok(self.blocks.len())
    }
    
    /// Calculate block hash using proper Bitcoin double SHA256
    /// Get the hash of a block
    pub fn get_block_hash(&self, block: &Block) -> Hash {
        self.block_hash(block)
    }
    
    fn block_hash(&self, block: &Block) -> Hash {
        use crate::storage::hashing::double_sha256;
        
        // Serialize block header for hashing
        let mut header_data = Vec::new();
        header_data.extend_from_slice(&block.header.version.to_le_bytes());
        header_data.extend_from_slice(&block.header.prev_block_hash);
        header_data.extend_from_slice(&block.header.merkle_root);
        header_data.extend_from_slice(&block.header.timestamp.to_le_bytes());
        header_data.extend_from_slice(&block.header.bits.to_le_bytes());
        header_data.extend_from_slice(&block.header.nonce.to_le_bytes());
        
        // Calculate Bitcoin double SHA256 hash
        double_sha256(&header_data)
    }
}

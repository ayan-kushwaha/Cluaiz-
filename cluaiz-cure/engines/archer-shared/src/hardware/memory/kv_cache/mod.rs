use candle_core::{Result, Tensor};
use crate::hardware::memory::kv_cache::paging::BlockManager;

pub mod block;
pub mod paging;
pub mod stitching;
pub mod eviction;

/// 🧬 PagedKVCache: The Central Sovereign Memory Controller.
/// This implementation lives in archer-shared to ensure total reusability across all archer engines.
pub struct PagedKVCache {
    pub sequence_id: String,
    pub block_manager: std::sync::Arc<std::sync::Mutex<BlockManager>>,
    pub k_blocks: Vec<Tensor>,
    pub v_blocks: Vec<Tensor>,
    pub current_block_usage: usize,
    pub head_count_kv: usize,
    pub head_dim: usize,
    pub max_blocks: usize,
}

impl PagedKVCache {
    pub fn new(
        sequence_id: &str,
        head_count_kv: usize,
        head_dim: usize,
        max_context: usize,
        block_manager: std::sync::Arc<std::sync::Mutex<BlockManager>>
    ) -> Self {
        let block_size = 16; // Fixed Sovereign Block Size
        let max_blocks = max_context / block_size;

        Self {
            sequence_id: sequence_id.to_string(),
            block_manager,
            k_blocks: Vec::new(),
            v_blocks: Vec::new(),
            current_block_usage: 0,
            head_count_kv,
            head_dim,
            max_blocks,
        }
    }

    /// 🧱 Core Append: Manages block allocation and token filling.
    pub fn append(&mut self, k: &Tensor, v: &Tensor) -> Result<()> {
        let tokens_to_add = k.dim(2)?;
        let block_size = 16;

        if self.k_blocks.is_empty() || self.current_block_usage + tokens_to_add > block_size {
            // Requesting new silicon slot
            let mut mg = self.block_manager.lock().unwrap();
            if let Some(_idx) = mg.allocate_block(&self.sequence_id) {
                self.k_blocks.push(k.clone());
                self.v_blocks.push(v.clone());
                self.current_block_usage = tokens_to_add;
            } else {
                return Err(candle_core::Error::Msg("SILICON_VRAM_EXHAUSTED".to_string()));
            }
        } else {
            // Native Stitching into active block
            let last_k = self.k_blocks.pop().unwrap();
            let last_v = self.v_blocks.pop().unwrap();
            self.k_blocks.push(Tensor::cat(&[&last_k, k], 2)?);
            self.v_blocks.push(Tensor::cat(&[&last_v, v], 2)?);
            self.current_block_usage += tokens_to_add;
        }

        // 🛡️ Automatic Eviction Guard
        if self.k_blocks.len() > self.max_blocks {
            self.k_blocks.remove(0);
            self.v_blocks.remove(0);
        }

        Ok(())
    }

    /// 🔗 AtmaSteer Injection: Directly stiching skill tensors.
    pub fn inject_history(&mut self, k_skill: &Tensor, v_skill: &Tensor) -> Result<()> {
        tracing::info!("🔗 [AtmaSteer] Shared Signal Injection initiated for seq: {}", self.sequence_id);
        self.k_blocks.insert(0, k_skill.clone());
        self.v_blocks.insert(0, v_skill.clone());
        Ok(())
    }

    pub fn get_kv_pair(&self) -> Result<(Tensor, Tensor)> {
        if self.k_blocks.is_empty() {
             return Err(candle_core::Error::Msg("EMPTY_CACHE_ACCESS".to_string()));
        }
        let k = Tensor::cat(&self.k_blocks.iter().collect::<Vec<_>>(), 2)?;
        let v = Tensor::cat(&self.v_blocks.iter().collect::<Vec<_>>(), 2)?;
        Ok((k, v))
    }
}

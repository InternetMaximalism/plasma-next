use plonky2::hash::hash_types::RichField;
use rand::Rng;

use crate::{
    common::{asset::Assets, block::Block},
    utils::h256::H256,
};

pub fn generate_random_blocks<F: RichField, R: Rng>(rng: &mut R, num_blocks: usize) -> Vec<Block> {
    let mut blocks = vec![];
    let total_deposit = Assets::rand_full(rng);
    let mut prev_block_hash = Block::default().block_hash();
    let mut prev_block_number = 0;
    for _ in 0..num_blocks {
        let block = Block {
            prev_block_hash,
            transfer_tree_root: H256::rand(rng),
            total_deposit: total_deposit.clone(),
            block_number: prev_block_number + 1,
        };
        blocks.push(block);
        prev_block_hash = blocks.last().unwrap().block_hash();
        prev_block_number += 1;
    }
    blocks
}

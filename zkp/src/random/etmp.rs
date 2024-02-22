use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOut, RichField},
    plonk::config::{AlgebraicHasher, GenericConfig},
};
use rand::Rng;

use crate::{
    common::{
        address::Address, asset::Assets, block::Block, transfer::Transfer,
        transfer_info::TransferInfo,
    },
    constants::TRANSFER_TREE_HEIGHT,
    utils::{
        h256::H256,
        trees::merkle_tree_with_leaves::{MerkleProofWithLeaves, MerkleTreeWithLeaves},
    },
};

use super::transfers::generate_random_transfers;

#[derive(Debug, Clone)]
pub struct ExtendedTransferMerkleProof<F: RichField> {
    pub block_root: HashOut<F>,
    pub block_merkle_proof: MerkleProofWithLeaves<F, Block>,
    pub transfer_info: TransferInfo<F>,
}

pub fn generate_random_etmps<F, C, const D: usize, R: Rng>(
    rng: &mut R,
    num_blocks: usize,
    num_transfers_per_recipient: usize,
    recipients: &[Address],
) -> Vec<ExtendedTransferMerkleProof<F>>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
{
    let transfers_vec =
        generate_random_transfers::<F, _>(rng, num_blocks, num_transfers_per_recipient, recipients);

    let mut block_tree = MerkleTreeWithLeaves::<F, Block>::new(32);
    block_tree.push(Block::default());
    let mut prev_block = Block::default();
    let initial_deposit = Assets::rand_full(rng);
    let mut transfer_merkle_proofs_vec = vec![];
    for transfers in transfers_vec.iter() {
        let mut transfer_tree = MerkleTreeWithLeaves::<F, Transfer>::new(TRANSFER_TREE_HEIGHT);
        for transfer in transfers.iter() {
            transfer_tree.push(*transfer);
        }
        let transfer_tree_root: H256 = transfer_tree.get_root().into();
        let transfer_merkle_proofs = transfers
            .iter()
            .enumerate()
            .map(|(i, _transfer)| transfer_tree.prove(i))
            .collect::<Vec<_>>();
        transfer_merkle_proofs_vec.push(transfer_merkle_proofs);
        let block = Block {
            prev_block_hash: prev_block.block_hash(),
            transfer_tree_root,
            total_deposit: initial_deposit.clone(),
            block_number: prev_block.block_number + 1,
        };
        block_tree.push(block.clone());
        prev_block = block;
    }

    let block_root = block_tree.get_root();
    let mut all_etmps = vec![];
    for (i, transfer_merkle_proofs) in transfer_merkle_proofs_vec.iter().enumerate() {
        let transfers = transfers_vec[i].clone();
        let block_number = i + 1;
        let block = block_tree.get_leaf(block_number);
        assert!(block.block_number == block_number as u32);
        let block_tree_proof = block_tree.prove(block_number);
        let etmps = transfer_merkle_proofs
            .iter()
            .enumerate()
            .map(|(transfer_index, tmp)| {
                let etmp = ExtendedTransferMerkleProof {
                    block_root: block_root.clone(),
                    block_merkle_proof: block_tree_proof.clone(),
                    transfer_info: TransferInfo {
                        transfer: transfers[transfer_index],
                        transfer_merkle_proof: tmp.clone(),
                        transfer_index,
                        block: block.clone(),
                    },
                };
                etmp
            })
            .collect::<Vec<_>>();
        all_etmps.extend(etmps);
    }
    all_etmps.sort_by_key(|w| w.transfer_info.ebn());
    all_etmps
}

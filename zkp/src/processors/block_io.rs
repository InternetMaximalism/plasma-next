use anyhow::ensure;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::HashOut;
use plonky2::plonk::config::AlgebraicHasher;
use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, plonk::config::GenericConfig,
};
use serde::{Deserialize, Serialize};

use crate::base_circuits::block_tree_circuit::{BlockTreeCircuit, BlockTreePublicInputs};
use crate::base_circuits::validity_circuit::{ValidityCircuit, ValidityPublicInputs};
use crate::common::block::Block;
use crate::common::transfer_info::TransferInfo;
use crate::utils::trees::merkle_tree_with_leaves::MerkleTreeWithLeaves;

pub struct BlockInfo<F, C, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    pub block: Block,
    pub transfer_info: Vec<TransferInfo<F>>,
    pub spent_proof: ProofWithPublicInputs<F, C, D>,
}

pub struct BlockStatus<F, C, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    pub latest_block: Block,
    pub block_root: HashOut<F>,
    pub validity_proof: Option<ProofWithPublicInputs<F, C, D>>,
    pub block_tree_proof: Option<ProofWithPublicInputs<F, C, D>>,
}

impl<F, C, const D: usize> BlockStatus<F, C, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    pub fn verify(
        &self,
        validity_circuit: &ValidityCircuit<F, C, D>,
        block_tree_circuit: &BlockTreeCircuit<F, C, D>,
    ) -> anyhow::Result<()> {
        // in the case that `latest_block` is the genesis block
        if self.latest_block == Block::default() {
            let mut block_tree = MerkleTreeWithLeaves::<F, Block>::new(32);
            let genesis_block = Block::default();
            block_tree.push(genesis_block.clone());
            let expected_block_root = block_tree.get_root();
            ensure!(
                self.block_root == expected_block_root,
                "block_root mismatch"
            );
            ensure!(
                self.validity_proof.is_none(),
                "validity_proof should be none"
            );
            ensure!(
                self.block_tree_proof.is_none(),
                "block_tree_proof should be none"
            );
        } else {
            ensure!(
                self.validity_proof.is_some(),
                "validity_proof should be set"
            );
            ensure!(
                self.block_tree_proof.is_some(),
                "block_tree_proof should be set"
            );
            ensure!(
                validity_circuit
                    .verify(self.validity_proof.clone().unwrap())
                    .is_ok(),
                "validity_proof verify failed"
            );
            ensure!(
                block_tree_circuit
                    .verify(self.block_tree_proof.clone().unwrap())
                    .is_ok(),
                "block_tree_proof verify failed"
            );
            let validity_pis = ValidityPublicInputs::from_pis(
                &self.validity_proof.as_ref().unwrap().public_inputs,
            );
            let block_tree_pis = BlockTreePublicInputs::from_pis(
                &self.block_tree_proof.as_ref().unwrap().public_inputs,
            );
            ensure!(
                block_tree_pis.block_root == self.block_root,
                "block_root mismatch"
            );
            ensure!(
                validity_pis.block_hash == self.latest_block.block_hash()
                    && block_tree_pis.block_hash == self.latest_block.block_hash(),
                "block_hash mismatch"
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound = "")]
pub struct BlockTreeStatus<F: Field> {
    pub latest_block_number: u32,
    pub block_root: HashOut<F>,
}

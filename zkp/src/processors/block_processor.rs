use anyhow::ensure;
use plonky2::hash::hash_types::HashOut;
use plonky2::plonk::config::AlgebraicHasher;
use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, plonk::config::GenericConfig,
};

use crate::base_circuits::block_tree_circuit::{BlockTreeCircuit, BlockTreeValue};
use crate::base_circuits::spent_circuit::{SpentCircuit, SpentPublicInputs, SpentValue};
use crate::base_circuits::validity_circuit::{ValidityCircuit, ValidityPublicInputs};
use crate::common::asset::Assets;
use crate::common::block::Block;
use crate::common::transfer::Transfer;
use crate::common::transfer_info::TransferInfo;
use crate::constants::TRANSFER_TREE_HEIGHT;
use crate::utils::trees::merkle_tree_with_leaves::MerkleTreeWithLeaves;

use super::block_io::{BlockInfo, BlockStatus, BlockTreeStatus};

pub struct BlockProcessor<F, C, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    latest_block: Block,
    validity_proof: Option<ProofWithPublicInputs<F, C, D>>,
    block_tree_proof: Option<ProofWithPublicInputs<F, C, D>>,
    pub block_tree: MerkleTreeWithLeaves<F, Block>,
}

impl<F, C, const D: usize> BlockProcessor<F, C, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    pub fn new() -> Self {
        let mut block_tree = MerkleTreeWithLeaves::<F, Block>::new(32);
        let genesis_block = Block::default();
        block_tree.push(genesis_block.clone());
        Self {
            latest_block: genesis_block,
            validity_proof: None,
            block_tree_proof: None,
            block_tree,
        }
    }

    pub fn generate_block(
        &self,
        spent_circuit: &SpentCircuit<F, C, D>,
        transfers: &[Transfer],
        deposit: &Assets,
    ) -> anyhow::Result<BlockInfo<F, C, D>> {
        let prev_total_deposit = if self.validity_proof.is_none() {
            Assets::default()
        } else {
            let validity_pis = ValidityPublicInputs::from_pis(
                &self.validity_proof.as_ref().unwrap().public_inputs,
            );
            validity_pis.total_deposit
        };
        let new_total_deposit = prev_total_deposit + deposit.clone();
        let spent_value = SpentValue::new::<F>(
            &transfers,
            &new_total_deposit,
            &self.latest_block.block_hash(),
            self.latest_block.block_number + 1,
        );

        // generate transfer_info
        let spent_proof = spent_circuit.prove(&spent_value)?;
        let spent_pis = SpentPublicInputs::from_vec(&spent_proof.public_inputs);
        let block = spent_pis.block.clone();

        let mut transfer_tree = MerkleTreeWithLeaves::<F, Transfer>::new(TRANSFER_TREE_HEIGHT);
        for transfer in transfers.iter() {
            transfer_tree.push(*transfer);
        }

        let transfer_info: Vec<TransferInfo<F>> = transfers
            .iter()
            .enumerate()
            .map(|(transfer_index, transfer)| {
                let transfer_merkle_proof = transfer_tree.prove(transfer_index);
                TransferInfo {
                    transfer: *transfer,
                    transfer_index,
                    transfer_merkle_proof,
                    block: block.clone(),
                }
            })
            .collect();

        Ok(BlockInfo {
            block,
            transfer_info,
            spent_proof,
        })
    }

    pub fn get_status(&self) -> BlockStatus<F, C, D> {
        BlockStatus {
            latest_block: self.latest_block.clone(),
            block_root: self.block_tree.get_root(),
            validity_proof: self.validity_proof.clone(),
            block_tree_proof: self.block_tree_proof.clone(),
        }
    }

    pub fn tick(
        &mut self,
        validity_circuit: &ValidityCircuit<F, C, D>,
        block_tree_circuit: &BlockTreeCircuit<F, C, D>,
        spent_proof: &ProofWithPublicInputs<F, C, D>,
    ) -> anyhow::Result<()> {
        let spent_proof_pis = SpentPublicInputs::from_vec(&spent_proof.public_inputs);
        // update block tree
        let new_block = spent_proof_pis.block.clone();
        ensure!(
            new_block.block_number == self.latest_block.block_number + 1,
            "block_number mismatch: {} != {} + 1",
            new_block.block_number,
            self.latest_block.block_number
        );
        ensure!(
            new_block.prev_block_hash == self.latest_block.block_hash(),
            "prev_block_hash mismatch"
        );
        let prev_block_root = self.block_tree.get_root();
        self.block_tree.push(new_block.clone());
        let new_block_root = self.block_tree.get_root();
        let block_merkle_proof = self.block_tree.prove(new_block.block_number as usize);
        let block_value = BlockTreeValue::new(
            new_block.clone(),
            prev_block_root,
            new_block_root,
            block_merkle_proof,
        );
        let block_tree_proof = block_tree_circuit.prove(&block_value, &self.block_tree_proof)?;
        self.block_tree_proof = Some(block_tree_proof);
        self.latest_block = new_block;
        // generate validity proof
        let validity_proof = validity_circuit.prove(spent_proof, &self.validity_proof)?;
        self.validity_proof = Some(validity_proof);
        Ok(())
    }

    pub fn get_validity_proof(&self) -> Option<ProofWithPublicInputs<F, C, D>> {
        self.validity_proof.clone()
    }

    pub fn get_block_tree_proof(&self) -> Option<ProofWithPublicInputs<F, C, D>> {
        self.block_tree_proof.clone()
    }

    // reset block tree only
    pub fn reset_block_tree(&mut self) {
        let mut block_tree = MerkleTreeWithLeaves::<F, Block>::new(32);
        block_tree.push(Block::default());
        self.block_tree = block_tree;
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    // this method is used in evidence/withdraw processor
    pub fn get_block_tree_snapshot(&self) -> MerkleTreeWithLeaves<F, Block> {
        self.block_tree.clone()
    }

    pub fn get_block_tree_status(&self) -> BlockTreeStatus<F> {
        BlockTreeStatus {
            latest_block_number: (self.block_tree.len() - 1) as u32,
            block_root: self.block_tree.get_root(),
        }
    }

    // add blocks to the block tree
    pub fn sync_block_tree(
        &mut self,
        blocks: &[Block],
        expected_block_root: HashOut<F>,
    ) -> anyhow::Result<()> {
        let mut block_tree_snapshot = self.get_block_tree_snapshot();
        for block in blocks {
            block_tree_snapshot.push(block.clone());
        }
        ensure!(
            block_tree_snapshot.get_root() == expected_block_root,
            "block_tree_root mismatch: {:?} != {:?}",
            block_tree_snapshot.get_root(),
            expected_block_root
        );
        self.block_tree = block_tree_snapshot;
        Ok(())
    }

    // restore the state of the processor from the given status
    // assume that block tree has been synced.
    pub fn restore(
        &mut self,
        validity_circuit: &ValidityCircuit<F, C, D>,
        block_tree_circuit: &BlockTreeCircuit<F, C, D>,
        status: &BlockStatus<F, C, D>,
    ) -> anyhow::Result<()> {
        status.verify(validity_circuit, block_tree_circuit)?;
        ensure!(
            self.block_tree.get_root() == status.block_root,
            "block_root mismatch"
        );
        self.latest_block = status.latest_block.clone();
        self.validity_proof = status.validity_proof.clone();
        self.block_tree_proof = status.block_tree_proof.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

    use crate::{
        base_circuits::{
            block_tree_circuit::BlockTreeCircuit, spent_circuit::SpentCircuit,
            validity_circuit::ValidityCircuit,
        },
        common::{address::Address, asset::Assets},
        random::transfers::generate_random_transfers,
    };

    use super::BlockProcessor;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_block_processor() {
        let mut rng = rand::thread_rng();
        let recipients = vec![Address::rand(&mut rng)];
        let transfers = generate_random_transfers::<F, _>(&mut rng, 1, 4, &recipients)[0].clone();
        let deposit = Assets::rand_full(&mut rng);

        let mut block_processor = BlockProcessor::<F, C, D>::new();
        let spent_circuit = SpentCircuit::new();
        let validity_circuit = ValidityCircuit::new(&spent_circuit);
        let block_tree_circuit = BlockTreeCircuit::new();

        let block_info = block_processor
            .generate_block(&spent_circuit, &transfers, &deposit)
            .unwrap();
        block_processor
            .tick(
                &validity_circuit,
                &block_tree_circuit,
                &block_info.spent_proof,
            )
            .unwrap();
        block_processor
            .get_status()
            .verify(&validity_circuit, &block_tree_circuit)
            .unwrap();

        // again
        let block_info = block_processor
            .generate_block(&spent_circuit, &transfers, &Assets::default())
            .unwrap();
        block_processor
            .tick(
                &validity_circuit,
                &block_tree_circuit,
                &block_info.spent_proof,
            )
            .unwrap();
        block_processor
            .get_status()
            .verify(&validity_circuit, &block_tree_circuit)
            .unwrap();
    }
}

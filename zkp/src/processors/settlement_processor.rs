use anyhow::ensure;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOut, RichField},
    plonk::{
        config::{AlgebraicHasher, GenericConfig},
        proof::ProofWithPublicInputs,
    },
};

use crate::{
    base_circuits::{
        block_tree_circuit::{BlockTreeCircuit, BlockTreePublicInputs},
        withdraw_circuit::{WithdrawCircuit, WithdrawPublicInputs, WithdrawValue},
    },
    common::{
        asset::Assets, block::Block, extended_block_number::ExtendedBlockNumber,
        transfer_info::TransferInfo,
    },
    tree_circuits::{
        settlement_leaf_circuit::{SettlementLeaf, SettlementLeafCircuit},
        settlement_tree_circuit::common_data_for_settlement_tree_circuit,
        tree_processor::{DynamicMerkleProofWithLeaf, ProofWithHash, TreeProcessor},
    },
    utils::trees::merkle_tree_with_leaves::MerkleTreeWithLeaves,
};

pub type SettlementTreeProcessor<F, C, const D: usize> =
    TreeProcessor<F, C, D, SettlementLeaf, SettlementLeafCircuit<F, C, D>>;

pub type SettlementMerkleProof = DynamicMerkleProofWithLeaf<SettlementLeaf>;

pub struct SettlementProcessor<F, C, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    pub withdraw_circuit: WithdrawCircuit<F, C, D>,
    pub settlement_leaf_circuit: SettlementLeafCircuit<F, C, D>,
    pub settlement_tree_processor: SettlementTreeProcessor<F, C, D>,
    pub block_root: Option<HashOut<F>>,
}

impl<F, C, const D: usize> SettlementProcessor<F, C, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    pub fn new(block_tree_circuit: &BlockTreeCircuit<F, C, D>) -> Self {
        let withdraw_circuit = WithdrawCircuit::new(block_tree_circuit);
        let settlement_leaf_circuit = SettlementLeafCircuit::new(&withdraw_circuit);
        let mut common_data = common_data_for_settlement_tree_circuit::<F, C, D>();
        let evidence_tree_processor =
            SettlementTreeProcessor::new(&settlement_leaf_circuit, &mut common_data);
        Self {
            withdraw_circuit,
            settlement_leaf_circuit,
            settlement_tree_processor: evidence_tree_processor,
            block_root: None,
        }
    }

    pub(crate) fn generate_leaf_proof(
        &self,
        block_tree_snapshot: &MerkleTreeWithLeaves<F, Block>,
        withdraw_proof: &ProofWithPublicInputs<F, C, D>,
        evidence_transfer_info: &TransferInfo<F>,
    ) -> anyhow::Result<(SettlementLeaf, ProofWithPublicInputs<F, C, D>)> {
        let block_root = block_tree_snapshot.get_root();
        let withdraw_pis = WithdrawPublicInputs::from_pis(&withdraw_proof.public_inputs);
        let block_merkle_proof_for_withdraw =
            block_tree_snapshot.prove(withdraw_pis.block.block_number as usize);
        let block_merkle_proof_for_evidence =
            block_tree_snapshot.prove(evidence_transfer_info.block.block_number as usize);
        let leaf = SettlementLeaf::new(
            &self.withdraw_circuit,
            &block_root,
            &block_merkle_proof_for_withdraw,
            &block_merkle_proof_for_evidence,
            withdraw_proof,
            evidence_transfer_info,
        )?;
        let leaf_proof = self.settlement_leaf_circuit.prove(
            &block_root,
            &block_merkle_proof_for_withdraw,
            &block_merkle_proof_for_evidence,
            withdraw_proof,
            evidence_transfer_info,
        )?;
        Ok((leaf, leaf_proof))
    }

    fn append_withdraw_proof_single(
        &self,
        block_tree_circuit: &BlockTreeCircuit<F, C, D>,
        block_tree: &MerkleTreeWithLeaves<F, Block>,
        block_tree_proof: &ProofWithPublicInputs<F, C, D>,
        transfer_info: &TransferInfo<F>,
        withdraw_proof: &Option<ProofWithPublicInputs<F, C, D>>,
    ) -> anyhow::Result<ProofWithPublicInputs<F, C, D>> {
        block_tree_circuit
            .verify(block_tree_proof.clone())
            .map_err(|_| anyhow::anyhow!("block_tree_proof verification failed"))?;
        let block_tree_pis = BlockTreePublicInputs::from_pis(&block_tree_proof.public_inputs);
        ensure!(
            block_tree_pis.block_root == block_tree.get_root(),
            "block root mismatch",
        );
        ensure!(
            transfer_info.block.block_number <= block_tree_pis.block.block_number,
            "block_tree is too old"
        );
        let new_withdraw_proof = if let Some(withdraw_proof) = withdraw_proof {
            self.withdraw_circuit
                .verify(&withdraw_proof)
                .map_err(|_| anyhow::anyhow!("withdraw_proof verification failed"))?;
            let withdraw_pis = WithdrawPublicInputs::from_pis(&withdraw_proof.public_inputs);
            let block_merkle_proof_prev =
                block_tree.prove(withdraw_pis.block.block_number as usize);
            let block_merkle_proof_transfer =
                block_tree.prove(transfer_info.block.block_number as usize);
            let withdraw_value = WithdrawValue::new(
                block_tree_circuit,
                false,
                withdraw_pis,
                transfer_info.clone(),
                block_tree_proof.clone(),
                block_merkle_proof_prev,
                block_merkle_proof_transfer,
            )
            .map_err(|e| anyhow::anyhow!("failed to construct withdraw_value: {}", e))?;
            self.withdraw_circuit
                .prove(&withdraw_value, Some(withdraw_proof.clone()))
                .map_err(|_| anyhow::anyhow!("failed to prove withdraw"))?
        } else {
            let withdraw_pis = WithdrawPublicInputs {
                recipient: transfer_info.transfer.recipient,
                total_amount: Assets::default(),
                start_ebn: ExtendedBlockNumber::default(),
                end_ebn: ExtendedBlockNumber::default(),
                block: Block::default(),
            };
            let block_merkle_proof_prev = block_tree.prove(0);
            let block_merkle_proof_transfer =
                block_tree.prove(transfer_info.block.block_number as usize);
            let withdraw_value = WithdrawValue::new(
                block_tree_circuit,
                true,
                withdraw_pis,
                transfer_info.clone(),
                block_tree_proof.clone(),
                block_merkle_proof_prev,
                block_merkle_proof_transfer,
            )
            .map_err(|e| anyhow::anyhow!("failed to construct withdraw_value: {}", e))?;
            self.withdraw_circuit
                .prove(&withdraw_value, None)
                .map_err(|_| anyhow::anyhow!("failed to prove withdraw"))?
        };
        Ok(new_withdraw_proof)
    }

    pub fn append_withdraw_proof(
        &self,
        block_tree_circuit: &BlockTreeCircuit<F, C, D>,
        block_tree: &MerkleTreeWithLeaves<F, Block>,
        block_tree_proof: &ProofWithPublicInputs<F, C, D>,
        transfer_info: &[TransferInfo<F>],
        withdraw_proof: &Option<ProofWithPublicInputs<F, C, D>>,
    ) -> anyhow::Result<ProofWithPublicInputs<F, C, D>> {
        ensure!(transfer_info.len() > 0, "transfer_info is empty");
        let mut transfer_info = transfer_info.to_vec();
        transfer_info.sort_by_key(|t| t.ebn());
        let mut new_withdraw_proof = withdraw_proof.clone();
        for t in transfer_info {
            new_withdraw_proof = Some(self.append_withdraw_proof_single(
                block_tree_circuit,
                block_tree,
                block_tree_proof,
                &t,
                &new_withdraw_proof,
            )?);
        }
        Ok(new_withdraw_proof.unwrap())
    }

    // valiate before `add`
    pub fn validate(
        &self,
        block_tree_snapshot: &MerkleTreeWithLeaves<F, Block>,
        withdraw_proof: &ProofWithPublicInputs<F, C, D>,
        evidence_transfer_info: &TransferInfo<F>,
    ) -> anyhow::Result<()> {
        ensure!(self.block_root.is_some(), "you have to initialize first");
        ensure!(
            self.block_root.unwrap() == block_tree_snapshot.get_root(),
            "block_tree_snapshot modified since initialize"
        );
        let last_block_number = (block_tree_snapshot.len() - 1) as u32;
        let withdaw_pis = WithdrawPublicInputs::from_pis(&withdraw_proof.public_inputs);
        ensure!(
            withdaw_pis.block.block_number <= last_block_number,
            "block_tree_snapshot is too old for withdraw_proof"
        );
        ensure!(
            evidence_transfer_info.block.block_number <= last_block_number,
            "block_tree_snapshot is too old for evidence_transfer_info"
        );
        self.withdraw_circuit
            .verify(withdraw_proof)
            .map_err(|_| anyhow::anyhow!("withdraw_proof verification failed"))?;
        evidence_transfer_info
            .verify()
            .map_err(|_| anyhow::anyhow!("evidence_transfer_info verification failed"))?;
        Ok(())
    }

    pub fn initialize(&mut self, block_tree_snapshot: &MerkleTreeWithLeaves<F, Block>) {
        self.block_root = Some(block_tree_snapshot.get_root());
        self.settlement_tree_processor.initialize();
    }

    pub fn add(
        &mut self,
        block_tree_snapshot: &MerkleTreeWithLeaves<F, Block>,
        withdraw_proof: &ProofWithPublicInputs<F, C, D>,
        evidence_transfer_info: &TransferInfo<F>,
    ) -> anyhow::Result<()> {
        self.validate(block_tree_snapshot, withdraw_proof, evidence_transfer_info)?;
        let (leaf, leaf_proof) =
            self.generate_leaf_proof(block_tree_snapshot, withdraw_proof, evidence_transfer_info)?;
        self.settlement_tree_processor.add(leaf, leaf_proof)?;
        Ok(())
    }

    pub fn get(&self) -> Vec<SettlementLeaf> {
        self.settlement_tree_processor.get()
    }

    pub fn finalize(&mut self) -> Option<(ProofWithHash<F, C, D>, Vec<SettlementMerkleProof>)> {
        self.block_root = None;
        self.settlement_tree_processor.finalize()
    }
}

#[cfg(test)]
mod tests {

    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
    use rand::seq::SliceRandom;

    use crate::{
        base_circuits::{
            block_tree_circuit::BlockTreeCircuit, spent_circuit::SpentCircuit,
            validity_circuit::ValidityCircuit,
        },
        common::{address::Address, asset::Assets},
        processors::block_processor::BlockProcessor,
        random::transfers::generate_random_transfers,
    };

    use super::SettlementProcessor;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_settlement_processor() {
        let mut rng = rand::thread_rng();
        let recipient = Address::rand(&mut rng);
        let latest_block_number = 2;
        let transfers_vec =
            generate_random_transfers::<F, _>(&mut rng, latest_block_number, 4, &[recipient]);
        let spent_circuit = SpentCircuit::new();
        let validity_circuit = ValidityCircuit::new(&spent_circuit);
        let block_tree_circuit = BlockTreeCircuit::new();
        let mut block_processor = BlockProcessor::<F, C, D>::new();

        let mut transfer_info = vec![];
        let mut deposits = vec![Assets::rand_full(&mut rng)];
        deposits.resize(transfers_vec.len(), Assets::default());
        for (transfers, deposit) in transfers_vec.iter().zip(deposits.iter()) {
            let res = block_processor
                .generate_block(&spent_circuit, transfers, deposit)
                .unwrap();
            block_processor
                .tick(&validity_circuit, &block_tree_circuit, &res.spent_proof)
                .unwrap();
            transfer_info.extend(res.transfer_info);
        }
        transfer_info.shuffle(&mut rng);

        let block_tree_snapshot = block_processor.get_block_tree_snapshot();
        let block_tree_proof_snapshot = block_processor.get_block_tree_proof().unwrap();

        let mut settlement_processor = SettlementProcessor::<F, C, D>::new(&block_tree_circuit);
        settlement_processor.initialize(&block_tree_snapshot);

        let mut settlement_witnesses = vec![];
        for info in &transfer_info {
            let withdraw_proof = settlement_processor
                .append_withdraw_proof(
                    &block_tree_circuit,
                    &block_tree_snapshot,
                    &block_tree_proof_snapshot,
                    &[info.clone()],
                    &None,
                )
                .unwrap();
            settlement_witnesses.push((withdraw_proof, info.clone()));
        }

        for w in &settlement_witnesses {
            settlement_processor
                .add(&block_tree_snapshot, &w.0, &w.1)
                .unwrap();
        }
        settlement_processor.finalize().unwrap();
    }
}

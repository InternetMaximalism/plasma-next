use super::io::{
    AddInput, AppendToProofInput, AppendToProofOutput, FinalizeOutput, GenerateBlockInput,
    SerializedBlockInfo, SerializedBlockStatus, SyncBlockTreeInput, TickInput,
};
use crate::{
    base_circuits::{
        block_tree_circuit::BlockTreeCircuit, spent_circuit::SpentCircuit,
        validity_circuit::ValidityCircuit, withdraw_circuit::WithdrawPublicInputs,
    },
    common::{block::Block, transfer_info::TransferInfo},
    processors::{
        block_io::{BlockStatus, BlockTreeStatus},
        block_processor::BlockProcessor,
        settlement_processor::SettlementProcessor,
        wrap_processor::{validate_balance_block_proof, WrapProcessor},
    },
    serialization::{
        serialized_hashout::SerializedHashOut, serialized_proof::SerializedProof,
        serialized_transfer_info::SerializedTransferInfo,
    },
    utils::trees::merkle_tree_with_leaves::MerkleTreeWithLeaves,
};
use anyhow::{anyhow, ensure};
use parking_lot::RwLock;
use plonky2::plonk::{
    config::{GenericConfig, PoseidonGoldilocksConfig},
    proof::ProofWithPublicInputs,
};
use stark_verifier::bn254_poseidon::plonky2_config::{
    standard_inner_stark_verifier_config, standard_stark_verifier_config,
    Bn254PoseidonGoldilocksConfig,
};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type OuterC = Bn254PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

pub struct ServerState {
    pub spent_circuit: SpentCircuit<F, C, D>,
    pub validity_circuit: ValidityCircuit<F, C, D>,
    pub block_tree_circuit: BlockTreeCircuit<F, C, D>,
    pub block_processor: RwLock<BlockProcessor<F, C, D>>,
    pub block_tree_snapshot: RwLock<Option<MerkleTreeWithLeaves<F, Block>>>,
    pub validity_proof_snapshot: RwLock<Option<ProofWithPublicInputs<F, C, D>>>,
    pub block_tree_proof_snapshot: RwLock<Option<ProofWithPublicInputs<F, C, D>>>,
    pub settlement_processor: RwLock<SettlementProcessor<F, C, D>>,
    pub wrap_processor: WrapProcessor<F, C, OuterC, D>,
}

impl ServerState {
    pub fn new() -> Self {
        let spent_circuit = SpentCircuit::<F, C, D>::new();
        let validity_circuit = ValidityCircuit::<F, C, D>::new(&spent_circuit);
        let block_tree_circuit = BlockTreeCircuit::<F, C, D>::new();
        let block_processor = BlockProcessor::<F, C, D>::new();
        let settlement_processor = SettlementProcessor::<F, C, D>::new(&block_tree_circuit);
        let inner_config = standard_inner_stark_verifier_config();
        let outer_config = standard_stark_verifier_config();
        let wrap_processor = WrapProcessor::<F, C, OuterC, D>::new(
            inner_config,
            outer_config,
            &validity_circuit,
            &block_tree_circuit,
            &settlement_processor,
        );
        Self {
            spent_circuit,
            validity_circuit,
            block_tree_circuit,
            block_processor: RwLock::new(block_processor),
            block_tree_snapshot: RwLock::new(None),
            validity_proof_snapshot: RwLock::new(None),
            block_tree_proof_snapshot: RwLock::new(None),
            settlement_processor: RwLock::new(settlement_processor),
            wrap_processor,
        }
    }

    pub fn get_status(&self) -> SerializedBlockStatus {
        let status = self.block_processor.read().get_status();
        let validity_proof = status
            .validity_proof
            .map(|proof| SerializedProof::from_proof(&self.validity_circuit.data, &proof));
        let block_tree_proof = status
            .block_tree_proof
            .map(|proof| SerializedProof::from_proof(&self.block_tree_circuit.data, &proof));
        SerializedBlockStatus {
            latest_block: status.latest_block.clone(),
            block_root: SerializedHashOut(status.block_root),
            validity_proof,
            block_tree_proof,
        }
    }

    pub fn generate_block(&self, input: GenerateBlockInput) -> anyhow::Result<SerializedBlockInfo> {
        let block_info = self.block_processor.read().generate_block(
            &self.spent_circuit,
            &input.transfers,
            &input.deposit,
        )?;
        let spent_proof =
            SerializedProof::from_proof(&self.spent_circuit.data, &block_info.spent_proof);
        let transfer_info = block_info
            .transfer_info
            .iter()
            .map(|t| t.clone().into())
            .collect::<Vec<SerializedTransferInfo>>();
        Ok(SerializedBlockInfo {
            block: block_info.block.clone(),
            transfer_info,
            spent_proof,
        })
    }

    pub fn tick(&self, input: TickInput) -> anyhow::Result<SerializedBlockStatus> {
        let spent_proof = input.spent_proof.to_proof(&self.spent_circuit.data)?;
        self.block_processor.write().tick(
            &self.validity_circuit,
            &self.block_tree_circuit,
            &spent_proof,
        )?;
        Ok(self.get_status())
    }

    pub fn reset_block_tree(&self) {
        self.block_processor.write().reset_block_tree();
    }

    pub fn reset(&self) {
        self.block_processor.write().reset();
    }

    pub fn get_block_tree_status(&self) -> BlockTreeStatus<F> {
        self.block_processor.read().get_block_tree_status()
    }

    pub fn get_snapshot_block_number(&self) -> u32 {
        let snapshot_block_number = match &self.block_tree_snapshot.read().as_ref() {
            None => 0,
            Some(leafs) => leafs.len() - 1,
        };
        snapshot_block_number as u32
    }

    pub fn sync_block_tree(&self, input: SyncBlockTreeInput) -> anyhow::Result<()> {
        let expected_block_root = input.expected_block_root.0;
        self.block_processor
            .write()
            .sync_block_tree(&input.blocks, expected_block_root)
    }

    pub fn restore(&self, input: SerializedBlockStatus) -> anyhow::Result<()> {
        let validity_proof = input
            .validity_proof
            .map(|proof| proof.to_proof(&self.validity_circuit.data))
            .transpose()?;
        let block_tree_proof = input
            .block_tree_proof
            .map(|proof| proof.to_proof(&self.block_tree_circuit.data))
            .transpose()?;
        let status = BlockStatus {
            latest_block: input.latest_block.clone(),
            block_root: input.block_root.0,
            validity_proof,
            block_tree_proof,
        };
        self.block_processor.write().restore(
            &self.validity_circuit,
            &self.block_tree_circuit,
            &status,
        )?;
        Ok(())
    }

    pub fn append_to_withdraw_proof(
        &self,
        input: AppendToProofInput,
    ) -> anyhow::Result<AppendToProofOutput> {
        ensure!(
            self.block_tree_snapshot.read().is_some(),
            "block_tree_snapshot is None"
        );
        let withdraw_proof = input
            .withdraw_proof
            .map(|proof| proof.to_proof(&self.settlement_processor.read().withdraw_circuit.data))
            .transpose()?;
        let transfer_info = input
            .transfer_info
            .iter()
            .map(|t| t.clone().into())
            .collect::<Vec<TransferInfo<F>>>();
        let new_withdraw_proof = self.settlement_processor.read().append_withdraw_proof(
            &self.block_tree_circuit,
            self.block_tree_snapshot.read().as_ref().unwrap(),
            self.block_tree_proof_snapshot.read().as_ref().unwrap(),
            &transfer_info,
            &withdraw_proof,
        )?;
        let withdraw_pis = WithdrawPublicInputs::from_pis(&new_withdraw_proof.public_inputs);
        Ok(AppendToProofOutput {
            withdraw_pis,
            withdraw_proof: SerializedProof::from_proof(
                &self.settlement_processor.read().withdraw_circuit.data,
                &new_withdraw_proof,
            ),
        })
    }

    pub fn initialize(&self) -> u32 {
        *self.block_tree_snapshot.write() =
            Some(self.block_processor.read().get_block_tree_snapshot());
        self.settlement_processor
            .write()
            .initialize(&self.block_tree_snapshot.read().as_ref().unwrap());
        *self.validity_proof_snapshot.write() = self.block_processor.read().get_validity_proof();
        *self.block_tree_proof_snapshot.write() =
            self.block_processor.read().get_block_tree_proof();
        let snapshot_block_number = &self.block_tree_snapshot.read().as_ref().unwrap().len() - 1;
        snapshot_block_number as u32
    }

    pub fn add(&self, input: AddInput) -> anyhow::Result<()> {
        ensure!(
            self.block_tree_snapshot.read().is_some(),
            "block_tree_snapshot is None"
        );
        let withdraw_proof = input
            .withdraw_proof
            .to_proof(&self.settlement_processor.read().withdraw_circuit.data)?;
        self.settlement_processor.write().add(
            &self.block_tree_snapshot.read().as_ref().unwrap(),
            &withdraw_proof,
            &input.evidence_transfer_info.into(),
        )?;
        Ok(())
    }

    pub fn finalize_and_wrap(&self) -> anyhow::Result<FinalizeOutput> {
        let validity_proof = self
            .validity_proof_snapshot
            .read()
            .clone()
            .ok_or(anyhow!("validity_proof is none"))?;
        let block_tree_proof = self
            .block_tree_proof_snapshot
            .read()
            .clone()
            .ok_or(anyhow!("block_tree_proof is none"))?;
        let block_root = validate_balance_block_proof(
            &self.validity_circuit,
            &self.block_tree_circuit,
            &validity_proof,
            &block_tree_proof,
        )?;
        ensure!(
            self.block_tree_snapshot.read().is_some(),
            "block_tree_snapshot is None"
        );
        ensure!(
            self.block_tree_snapshot.read().as_ref().unwrap().get_root() == block_root,
            "block_tree_snapshot root is not equal to block_root"
        );
        let settlement_res = { self.settlement_processor.write().finalize() };
        let (settlement_tree_proof, settlment_merkle_proofs) = if settlement_res.is_some() {
            (
                settlement_res.as_ref().unwrap().0.proof.clone(),
                settlement_res.unwrap().1,
            )
        } else {
            return Ok(FinalizeOutput {
                settlement_merkle_proofs: None,
                wrap_public_inputs: None,
                wrap_proof: None,
            });
        };

        let (wrap_public_inputs, wrap_proof) = self.wrap_processor.wrap(
            &self.validity_circuit,
            &self.block_tree_circuit,
            &self.settlement_processor.read(),
            validity_proof,
            block_tree_proof,
            settlement_tree_proof,
        )?;
        let wrap_proof =
            SerializedProof::from_proof(&self.wrap_processor.wrap2_circuit.data, &wrap_proof);
        Ok(FinalizeOutput {
            settlement_merkle_proofs: Some(settlment_merkle_proofs),
            wrap_public_inputs: Some(wrap_public_inputs),
            wrap_proof: Some(wrap_proof),
        })
    }
}

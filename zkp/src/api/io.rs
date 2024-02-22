use serde::{Deserialize, Serialize};

use crate::{
    base_circuits::withdraw_circuit::WithdrawPublicInputs,
    common::{asset::Assets, block::Block, transfer::Transfer},
    processors::settlement_processor::SettlementMerkleProof,
    wrap_circuits::wrap::WrapPublicInputs,
};

use crate::serialization::{
    serialized_hashout::SerializedHashOut, serialized_proof::SerializedProof,
    serialized_transfer_info::SerializedTransferInfo,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateBlockInput {
    pub transfers: Vec<Transfer>,
    pub deposit: Assets,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickInput {
    pub spent_proof: SerializedProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializedBlockStatus {
    pub latest_block: Block,
    pub block_root: SerializedHashOut,
    pub validity_proof: Option<SerializedProof>,
    pub block_tree_proof: Option<SerializedProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendToProofInput {
    pub transfer_info: Vec<SerializedTransferInfo>,
    pub withdraw_proof: Option<SerializedProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendToProofOutput {
    pub withdraw_pis: WithdrawPublicInputs,
    pub withdraw_proof: SerializedProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncBlockTreeInput {
    pub blocks: Vec<Block>,
    pub expected_block_root: SerializedHashOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddInput {
    pub withdraw_proof: SerializedProof,
    pub evidence_transfer_info: SerializedTransferInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalizeOutput {
    pub wrap_public_inputs: Option<WrapPublicInputs>,
    pub settlement_merkle_proofs: Option<Vec<SettlementMerkleProof>>,
    pub wrap_proof: Option<SerializedProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializedBlockInfo {
    pub block: Block,
    pub transfer_info: Vec<SerializedTransferInfo>,
    pub spent_proof: SerializedProof,
}

use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOut, RichField},
    iop::{target::Target, witness::Witness},
    plonk::circuit_builder::CircuitBuilder,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    common::{
        block::Block,
        extended_block_number::{ExtendedBlockNumber, ExtendedBlockNumberTarget},
        transfer_info::TransferInfo,
    },
    utils::{
        h256::{H256Target, H256},
        trees::merkle_tree_with_leaves::MerkleProofWithLeaves,
    },
};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceLeaf {
    pub transfer_commitment: H256,
    pub ebn: ExtendedBlockNumber,
}

pub const EVIDENCE_LEAF_LEN: usize = 10;

impl Display for EvidenceLeaf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(transfer_commitment: {}, ebn: {})",
            self.transfer_commitment, self.ebn
        )
    }
}

impl EvidenceLeaf {
    pub fn new<F: RichField>(
        block_root: &HashOut<F>,
        block_merkle_proof: &MerkleProofWithLeaves<F, Block>,
        transfer_info: &TransferInfo<F>,
    ) -> anyhow::Result<Self> {
        transfer_info
            .verify()
            .map_err(|e| anyhow::anyhow!("transfer_info verification failed: {}", e))?;
        block_merkle_proof
            .verify(
                &transfer_info.block,
                transfer_info.block.block_number as usize,
                block_root.clone(),
            )
            .map_err(|_| anyhow::anyhow!("block_merkle_proof verification failed"))?;
        let transfer_commitment = transfer_info.transfer.keccak_hash();
        let ebn = transfer_info.ebn();
        Ok(Self {
            transfer_commitment,
            ebn,
        })
    }

    pub fn rand<R: Rng>(rng: &mut R) -> Self {
        Self {
            transfer_commitment: H256::rand(rng),
            ebn: ExtendedBlockNumber::new(rng.gen()),
        }
    }

    pub fn to_u32_digits(&self) -> Vec<u32> {
        let mut u32_digits = self.transfer_commitment.to_u32_digits().to_vec();
        u32_digits.extend(self.ebn.to_u32_digits().to_vec());
        assert_eq!(u32_digits.len(), EVIDENCE_LEAF_LEN);
        u32_digits
    }
}

#[derive(Debug, Clone)]
pub struct EvidenceLeafTarget {
    pub transfer_commitment: H256Target,
    pub ebn: ExtendedBlockNumberTarget,
}

impl EvidenceLeafTarget {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        Self {
            transfer_commitment: H256Target::new_unsafe(builder),
            ebn: ExtendedBlockNumberTarget::new(builder),
        }
    }

    pub fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        input: &EvidenceLeaf,
    ) -> Self {
        Self {
            transfer_commitment: H256Target::constant(builder, input.transfer_commitment),
            ebn: ExtendedBlockNumberTarget::constant(builder, &input.ebn),
        }
    }

    pub fn to_u32_digits<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Vec<Target> {
        let mut vec = Vec::new();
        vec.extend(self.transfer_commitment.to_vec());
        vec.extend(self.ebn.to_u32_digits(builder).to_vec());
        assert_eq!(vec.len(), EVIDENCE_LEAF_LEN);
        vec
    }

    pub fn set_witness<F: RichField, W: Witness<F>>(&self, pw: &mut W, input: &EvidenceLeaf) {
        self.transfer_commitment
            .set_witness(pw, input.transfer_commitment);
        self.ebn.set_witness(pw, &input.ebn);
    }
}

use crate::{
    common::{block::Block, transfer::Transfer},
    constants::TRANSFER_TREE_HEIGHT,
    utils::trees::merkle_tree_with_leaves::{MerkleProofWithLeaves, MerkleProofWithLeavesTarget},
};
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::{target::Target, witness::Witness},
    plonk::circuit_builder::CircuitBuilder,
};

use super::{
    block::BlockTarget,
    extended_block_number::{ExtendedBlockNumber, ExtendedBlockNumberTarget},
    transfer::TransferTarget,
};

// Information that user needs to know to check the validity of a transfer besides the balance proof
#[derive(Debug, Clone)]
pub struct TransferInfo<F: RichField> {
    pub transfer: Transfer,
    pub transfer_index: usize,
    pub transfer_merkle_proof: MerkleProofWithLeaves<F, Transfer>,
    pub block: Block,
}

impl<F: RichField> TransferInfo<F> {
    pub fn verify(&self) -> anyhow::Result<()> {
        let transfer_tree_root = self.block.transfer_tree_root.reduce_to_hash_out();
        self.transfer_merkle_proof.verify(
            &self.transfer,
            self.transfer_index,
            transfer_tree_root,
        )?;
        Ok(())
    }

    pub fn ebn(&self) -> ExtendedBlockNumber {
        ExtendedBlockNumber::construct(self.block.block_number, self.transfer_index)
    }
}

#[derive(Debug, Clone)]
pub struct TransferInfoTarget {
    pub transfer: TransferTarget,
    pub transfer_index: Target,
    pub transfer_merkle_proof: MerkleProofWithLeavesTarget<TransferTarget>,
    pub block: BlockTarget,
}

impl TransferInfoTarget {
    pub fn verify<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) {
        let transfer_tree_root = self
            .block
            .transfer_tree_root
            .reduce_to_hash_out_target(builder);
        self.transfer_merkle_proof.verify(
            builder,
            &self.transfer,
            self.transfer_index,
            transfer_tree_root,
        );
    }

    pub fn ebn<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> ExtendedBlockNumberTarget {
        ExtendedBlockNumberTarget::construct(builder, self.block.block_number, self.transfer_index)
    }

    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        let transfer = TransferTarget::new(builder);
        let transfer_index = builder.add_virtual_target();
        let transfer_merkle_proof = MerkleProofWithLeavesTarget::new(builder, TRANSFER_TREE_HEIGHT);
        let block = BlockTarget::new(builder);
        Self {
            transfer,
            transfer_index,
            transfer_merkle_proof,
            block,
        }
    }

    pub fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        input: &TransferInfo<F>,
    ) -> Self {
        let transfer = TransferTarget::constant(builder, input.transfer);
        let transfer_index = builder.constant(F::from_canonical_usize(input.transfer_index));
        let transfer_merkle_proof =
            MerkleProofWithLeavesTarget::constant(builder, &input.transfer_merkle_proof);
        let block = BlockTarget::constant(builder, &input.block);
        Self {
            transfer,
            transfer_index,
            transfer_merkle_proof,
            block,
        }
    }

    pub fn set_witness<F: RichField>(&self, pw: &mut impl Witness<F>, value: &TransferInfo<F>) {
        self.transfer.set_witness(pw, &value.transfer);
        pw.set_target(
            self.transfer_index,
            F::from_canonical_usize(value.transfer_index),
        );
        self.transfer_merkle_proof
            .set_witness(pw, &value.transfer_merkle_proof);
        self.block.set_witness(pw, &value.block);
    }
}

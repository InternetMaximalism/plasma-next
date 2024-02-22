use std::fmt::Display;

use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOut, RichField},
    iop::{target::Target, witness::Witness},
    plonk::{
        circuit_builder::CircuitBuilder,
        config::{AlgebraicHasher, GenericConfig},
        proof::ProofWithPublicInputs,
    },
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    base_circuits::withdraw_circuit::{WithdrawCircuit, WithdrawPublicInputs},
    common::{
        address::{Address, AddressTarget},
        asset::{Assets, AssetsTarget},
        block::Block,
        extended_block_number::{ExtendedBlockNumber, ExtendedBlockNumberTarget},
    },
    constants::NUM_ASSETS,
    utils::trees::merkle_tree_with_leaves::MerkleProofWithLeaves,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawLeaf {
    pub recipient: Address,
    pub amount: Assets,
    pub start_ebn: ExtendedBlockNumber,
    pub end_ebn: ExtendedBlockNumber,
}

pub const WITHDRAW_LEAF_LEN: usize = 5 + 8 * NUM_ASSETS + 2 + 2;

impl Display for WithdrawLeaf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(recipient: {}, amount: {}, start_ebn: {}, end_ebn: {})",
            self.recipient, self.amount, self.start_ebn, self.end_ebn
        )
    }
}

impl WithdrawLeaf {
    pub fn new<F, C, const D: usize>(
        withdraw_circuit: &WithdrawCircuit<F, C, D>,
        block_root: &HashOut<F>,
        block_merkle_proof: &MerkleProofWithLeaves<F, Block>,
        withdraw_proof: &ProofWithPublicInputs<F, C, D>,
    ) -> anyhow::Result<Self>
    where
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F> + 'static,
        <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
    {
        withdraw_circuit.verify(withdraw_proof)?;
        let withdraw_pis = WithdrawPublicInputs::from_pis(&withdraw_proof.public_inputs);
        block_merkle_proof.verify(
            &withdraw_pis.block,
            withdraw_pis.block.block_number as usize,
            block_root.clone(),
        )?;
        Ok(Self {
            recipient: withdraw_pis.recipient,
            amount: withdraw_pis.total_amount,
            start_ebn: withdraw_pis.start_ebn,
            end_ebn: withdraw_pis.end_ebn,
        })
    }

    pub fn rand<R: Rng>(rng: &mut R) -> Self {
        Self {
            recipient: Address::rand(rng),
            amount: Assets::rand(rng),
            start_ebn: ExtendedBlockNumber::new(rng.gen()),
            end_ebn: ExtendedBlockNumber::new(rng.gen()),
        }
    }

    pub fn to_u32_digits(&self) -> Vec<u32> {
        let mut u32_digits = self.recipient.to_u32_digits().to_vec();
        u32_digits.extend(self.amount.to_u32_digits().to_vec());
        u32_digits.extend(self.start_ebn.to_u32_digits().to_vec());
        u32_digits.extend(self.end_ebn.to_u32_digits().to_vec());
        assert_eq!(u32_digits.len(), WITHDRAW_LEAF_LEN);
        u32_digits
    }
}

#[derive(Debug, Clone)]
pub struct WithdrawLeafTarget {
    pub recipient: AddressTarget,
    pub amount: AssetsTarget,
    pub start_ebn: ExtendedBlockNumberTarget,
    pub end_ebn: ExtendedBlockNumberTarget,
}

impl WithdrawLeafTarget {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        Self {
            recipient: AddressTarget::new(builder),
            amount: AssetsTarget::new(builder),
            start_ebn: ExtendedBlockNumberTarget::new(builder),
            end_ebn: ExtendedBlockNumberTarget::new(builder),
        }
    }

    pub fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        input: &WithdrawLeaf,
    ) -> Self {
        Self {
            recipient: AddressTarget::constant(builder, input.recipient),
            amount: AssetsTarget::constant(builder, &input.amount),
            start_ebn: ExtendedBlockNumberTarget::constant(builder, &input.start_ebn),
            end_ebn: ExtendedBlockNumberTarget::constant(builder, &input.end_ebn),
        }
    }

    pub fn to_vec(&self) -> Vec<Target> {
        let mut vec = Vec::new();
        vec.extend(self.recipient.to_vec());
        vec.extend(self.amount.to_vec());
        vec.extend(self.start_ebn.to_vec());
        vec.extend(self.end_ebn.to_vec());
        assert_eq!(vec.len(), 5 + 8 * NUM_ASSETS + 1 + 1);
        vec
    }

    pub fn to_u32_digits<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Vec<Target> {
        let mut vec = Vec::new();
        vec.extend(self.recipient.to_u32_digits().to_vec());
        vec.extend(self.amount.to_vec());
        vec.extend(self.start_ebn.to_u32_digits(builder).to_vec());
        vec.extend(self.end_ebn.to_u32_digits(builder).to_vec());
        assert_eq!(vec.len(), WITHDRAW_LEAF_LEN);
        vec
    }

    pub fn set_witness<F: RichField, W: Witness<F>>(&self, pw: &mut W, input: &WithdrawLeaf) {
        self.recipient.set_witness(pw, input.recipient);
        self.amount.set_witness(pw, &input.amount);
        self.start_ebn.set_witness(pw, &input.start_ebn);
        self.end_ebn.set_witness(pw, &input.end_ebn);
    }
}

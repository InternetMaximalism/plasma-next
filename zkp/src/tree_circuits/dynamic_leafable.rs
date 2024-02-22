use std::fmt::Display;

use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder, config::GenericConfig, proof::ProofWithPublicInputsTarget,
    },
};

use crate::utils::{dummy::DummyProof, h256::H256};

pub trait DynamicLeafableCircuit<F, C, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    fn add_proof_target_and_conditionally_verify(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        condition: &BoolTarget,
    ) -> ProofWithPublicInputsTarget<D>;

    fn dummy_leaf(&self) -> DummyProof<F, C, D>;
}

pub trait DynamicLeafable: Clone + Display {
    fn hash(&self) -> H256;
}

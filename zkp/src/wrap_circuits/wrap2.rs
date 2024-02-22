use std::marker::PhantomData;

use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{AlgebraicHasher, GenericConfig},
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};

use super::wrap::WrapCircuit;

// By further wrapping the wrap_circuit, we reduce the degree_bits.
pub struct Wrap2Circuit<F, C, OuterC, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    OuterC: GenericConfig<D, F = F>,
{
    pub data: CircuitData<F, OuterC, D>,
    pub wrap_proof: ProofWithPublicInputsTarget<D>,
    _maker: PhantomData<C>,
}

impl<F, C, OuterC, const D: usize> Wrap2Circuit<F, C, OuterC, D>
where
    F: RichField + Extendable<D>,
    OuterC: GenericConfig<D, F = F>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
{
    pub fn new(config: CircuitConfig, wrap_circuit: &WrapCircuit<F, C, D>) -> Self {
        let mut builder = CircuitBuilder::new(config);
        let wrap_proof = wrap_circuit.add_proof_target_and_verify(&mut builder);
        assert_eq!(wrap_proof.public_inputs.len(), 4);
        builder.register_public_inputs(&wrap_proof.public_inputs);
        let data = builder.build();
        Self {
            data,
            wrap_proof,
            _maker: PhantomData,
        }
    }

    pub fn prove(
        &self,
        wrap_proof: &ProofWithPublicInputs<F, C, D>,
    ) -> anyhow::Result<ProofWithPublicInputs<F, OuterC, D>> {
        let mut pw = PartialWitness::new();
        pw.set_proof_with_pis_target(&self.wrap_proof, wrap_proof);
        self.data.prove(pw)
    }
}

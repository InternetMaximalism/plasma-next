use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    plonk::{
        circuit_data::{CircuitData, CommonCircuitData},
        config::{AlgebraicHasher, GenericConfig},
        proof::ProofWithPublicInputs,
    },
    recursion::dummy_circuit::{cyclic_base_proof, dummy_circuit, dummy_proof},
};

#[derive(Debug)]
pub struct DummyProof<F, C, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    pub proof: ProofWithPublicInputs<F, C, D>,
}

impl<F, C, const D: usize> DummyProof<F, C, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    pub fn new(common: &CommonCircuitData<F, D>) -> Self {
        let data = dummy_circuit::<F, C, D>(&common);
        let proof = dummy_proof(&data, vec![].into_iter().enumerate().collect()).unwrap();
        Self { proof }
    }

    pub fn new_cyclic(data: &CircuitData<F, C, D>) -> Self {
        let proof = cyclic_base_proof(
            &data.common,
            &data.verifier_only,
            vec![].into_iter().enumerate().collect(),
        );
        Self { proof }
    }
}

use std::marker::PhantomData;

use plonky2::{
    field::extension::Extendable,
    gates::{noop::NoopGate, random_access::RandomAccessGate},
    hash::hash_types::{HashOut, HashOutTarget, RichField},
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, CommonCircuitData, VerifierCircuitTarget},
        config::{AlgebraicHasher, GenericConfig},
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
    recursion::cyclic_recursion::check_cyclic_proof_verifier_data,
};
use starky_keccak::builder::CircuitBuilderWithKeccak;

use crate::utils::{
    dummy::DummyProof,
    h256::{H256Target, H256},
    logic::enforce_equal_if_enabled,
};

use super::dynamic_leafable::DynamicLeafableCircuit;

pub struct DynamicTreePublicInputs<F: RichField> {
    pub hash: H256,
    pub block_root: HashOut<F>,
}

impl<F: RichField> DynamicTreePublicInputs<F> {
    pub fn from_pis(input: &[F]) -> Self {
        let hash = H256::from_vec(&input[0..8]);
        let block_root = HashOut::from_vec(input[8..12].to_vec());
        Self { hash, block_root }
    }
}

pub struct DynamicTreePublicInputsTarget {
    pub hash: H256Target,
    pub block_root: HashOutTarget,
}

impl DynamicTreePublicInputsTarget {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        Self {
            hash: H256Target::new_unsafe(builder),
            block_root: builder.add_virtual_hash(),
        }
    }

    pub fn to_vec(&self) -> Vec<Target> {
        let mut vec = Vec::new();
        vec.extend(self.hash.to_vec());
        vec.extend(self.block_root.elements);
        assert_eq!(vec.len(), 12);
        vec
    }

    pub fn from_pis(input: &[Target]) -> Self {
        let hash = H256Target::from_vec(&input[0..8]);
        let block_root = HashOutTarget::from_vec(input[8..12].to_vec());
        Self { hash, block_root }
    }
}

pub struct DynamicTreeCircuit<F, C, const D: usize, InnerCircuit>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    InnerCircuit: DynamicLeafableCircuit<F, C, D>,
{
    pub data: CircuitData<F, C, D>,
    pub is_not_first_step: BoolTarget,
    pub leaf_proof: ProofWithPublicInputsTarget<D>,
    pub prev_left_proof: ProofWithPublicInputsTarget<D>,
    pub prev_right_proof: ProofWithPublicInputsTarget<D>,
    pub dummy_leaf: DummyProof<F, C, D>,
    pub dummy_node: DummyProof<F, C, D>,
    pub vd: VerifierCircuitTarget,
    _phantom: PhantomData<InnerCircuit>,
}

impl<F, C, const D: usize, InnerCircuit> DynamicTreeCircuit<F, C, D, InnerCircuit>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
    InnerCircuit: DynamicLeafableCircuit<F, C, D>,
{
    pub fn new(inner_circuit: &InnerCircuit, common_data: &mut CommonCircuitData<F, D>) -> Self {
        let mut builder = CircuitBuilderWithKeccak::<F, D>::new(CircuitConfig::default());
        let cur_pis = DynamicTreePublicInputsTarget::new(&mut builder);
        builder.register_public_inputs(&cur_pis.to_vec());

        let vd = builder.add_verifier_data_public_inputs();
        common_data.num_public_inputs = builder.num_public_inputs();

        let is_not_first_step = builder.add_virtual_bool_target_safe(); // whether this circuit uses recursive proof or not
        let is_first_step = builder.not(is_not_first_step);
        let leaf_proof =
            inner_circuit.add_proof_target_and_conditionally_verify(&mut builder, &is_first_step);
        let prev_left_proof = builder.add_virtual_proof_with_pis(&common_data);
        let prev_right_proof = builder.add_virtual_proof_with_pis(&common_data);

        builder
            .conditionally_verify_cyclic_proof_or_dummy::<C>(
                is_not_first_step,
                &prev_left_proof,
                &common_data,
            )
            .unwrap();
        builder
            .conditionally_verify_cyclic_proof_or_dummy::<C>(
                is_not_first_step,
                &prev_right_proof,
                &common_data,
            )
            .unwrap();

        // in the case of leaf
        let leaf_pis = DynamicTreePublicInputsTarget::from_pis(&leaf_proof.public_inputs);
        let leaf_hash = leaf_pis.hash;
        let leaf_block_root = leaf_pis.block_root;

        // in the case of non-leaf
        let left_pis = DynamicTreePublicInputsTarget::from_pis(&prev_left_proof.public_inputs);
        let left_hash = left_pis.hash;
        let left_block_root = left_pis.block_root;
        let right_pis = DynamicTreePublicInputsTarget::from_pis(&prev_right_proof.public_inputs);
        let right_hash = right_pis.hash;
        let right_block_root = right_pis.block_root;
        enforce_equal_if_enabled(
            &mut builder,
            left_block_root,
            right_block_root,
            is_not_first_step,
        );
        let node_hash = H256Target::from_vec(
            &builder.keccak256(vec![left_hash.to_vec(), right_hash.to_vec()].concat()),
        );
        let node_block_root = left_block_root;

        let next_hash = H256Target::select(&mut builder, is_first_step, leaf_hash, node_hash);
        let next_block_root = builder.select_hash(is_first_step, leaf_block_root, node_block_root);
        cur_pis.hash.connect(&mut builder, next_hash);
        builder.connect_hashes(cur_pis.block_root, next_block_root);

        let data = builder.build::<C>();
        assert_eq!(&data.common, common_data);
        let dummy_leaf = inner_circuit.dummy_leaf();
        let dummy_node = DummyProof::<F, C, D>::new_cyclic(&data);

        Self {
            data,
            is_not_first_step,
            leaf_proof,
            prev_left_proof,
            prev_right_proof,
            dummy_leaf,
            dummy_node,
            vd,
            _phantom: PhantomData,
        }
    }

    pub fn prove(
        &self,
        leaf_proof: Option<ProofWithPublicInputs<F, C, D>>,
        left_and_right_proof: Option<(
            ProofWithPublicInputs<F, C, D>,
            ProofWithPublicInputs<F, C, D>,
        )>,
    ) -> anyhow::Result<ProofWithPublicInputs<F, C, D>> {
        let mut pw = PartialWitness::new();
        pw.set_verifier_data_target(&self.vd, &self.data.verifier_only);
        if leaf_proof.is_some() {
            assert!(left_and_right_proof.is_none());
            pw.set_bool_target(self.is_not_first_step, false);
            pw.set_proof_with_pis_target(&self.leaf_proof, &leaf_proof.unwrap());
            pw.set_proof_with_pis_target(&self.prev_left_proof, &self.dummy_node.proof);
            pw.set_proof_with_pis_target(&self.prev_right_proof, &self.dummy_node.proof);
        } else {
            assert!(left_and_right_proof.is_some());
            pw.set_bool_target(self.is_not_first_step, true);
            let (left_proof, right_proof) = left_and_right_proof.unwrap();
            pw.set_proof_with_pis_target(&self.leaf_proof, &self.dummy_leaf.proof);
            pw.set_proof_with_pis_target(&self.prev_left_proof, &left_proof);
            pw.set_proof_with_pis_target(&self.prev_right_proof, &right_proof);
        }
        self.data.prove(pw)
    }

    pub fn verify(&self, proof_with_pis: ProofWithPublicInputs<F, C, D>) -> anyhow::Result<()> {
        check_cyclic_proof_verifier_data(
            &proof_with_pis,
            &self.data.verifier_only,
            &self.data.common,
        )?;
        self.data.verify(proof_with_pis)
    }
}

pub fn common_data_for_dynamic_tree_circuit<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
>() -> CommonCircuitData<F, D>
where
    C::Hasher: AlgebraicHasher<F>,
{
    let config = CircuitConfig::standard_recursion_config();
    let builder = CircuitBuilder::<F, D>::new(config);
    let data = builder.build::<C>();

    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);
    let proof = builder.add_virtual_proof_with_pis(&data.common);
    let verifier_data = VerifierCircuitTarget {
        constants_sigmas_cap: builder.add_virtual_cap(data.common.config.fri_config.cap_height),
        circuit_digest: builder.add_virtual_hash(),
    };
    builder.verify_proof::<C>(&proof, &verifier_data, &data.common);
    let data = builder.build::<C>();
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config.clone());
    let proof = builder.add_virtual_proof_with_pis(&data.common);
    let verifier_data = VerifierCircuitTarget {
        constants_sigmas_cap: builder.add_virtual_cap(data.common.config.fri_config.cap_height),
        circuit_digest: builder.add_virtual_hash(),
    };
    builder.verify_proof::<C>(&proof, &verifier_data, &data.common);

    let random_access_gate = RandomAccessGate::<F, D>::new_from_config(&config, 1);
    builder.add_gate(random_access_gate, vec![]);
    while builder.num_gates() < 1 << 14 {
        builder.add_gate(NoopGate, vec![]);
    }
    builder.build::<C>().common
}

#[cfg(test)]
mod tests {
    use plonky2::{
        field::{extension::Extendable, types::Field},
        hash::{
            hash_types::{HashOut, RichField},
            poseidon::PoseidonHash,
        },
        iop::{
            target::{BoolTarget, Target},
            witness::{PartialWitness, WitnessWrite},
        },
        plonk::{
            circuit_builder::CircuitBuilder,
            circuit_data::{CircuitConfig, CircuitData},
            config::{AlgebraicHasher, GenericConfig, PoseidonGoldilocksConfig},
            proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
        },
    };
    use starky_keccak::builder::CircuitBuilderWithKeccak;

    use crate::{
        tree_circuits::dynamic_leafable::DynamicLeafableCircuit,
        utils::{dummy::DummyProof, h256::H256Target},
    };

    use super::{DynamicTreeCircuit, DynamicTreePublicInputsTarget};

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    struct Acircuit<F, C, const D: usize>
    where
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
    {
        pub data: CircuitData<F, C, D>,
        pub target: Target,
    }

    impl<F, C, const D: usize> Acircuit<F, C, D>
    where
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F> + 'static,
        C::Hasher: AlgebraicHasher<F>,
    {
        pub fn new() -> Self {
            let mut builder = CircuitBuilderWithKeccak::<F, D>::new(
                CircuitConfig::standard_stark_verifier_config(),
            );
            let target = builder.add_virtual_target();
            let hash_out = builder.hash_n_to_hash_no_pad::<PoseidonHash>(vec![target]);
            let hash = H256Target::from_hash_out_target(&mut builder, hash_out);
            let block_root = builder.constant_hash(HashOut::default());
            let pis = DynamicTreePublicInputsTarget { hash, block_root };
            builder.register_public_inputs(&pis.to_vec());
            let data = builder.build::<C>();
            Self { data, target }
        }

        pub fn prove(&self, input: F) -> ProofWithPublicInputs<F, C, D> {
            let mut pw = PartialWitness::new();
            pw.set_target(self.target, input);
            self.data.prove(pw).unwrap()
        }
    }

    impl<F, C, const D: usize> DynamicLeafableCircuit<F, C, D> for Acircuit<F, C, D>
    where
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F> + 'static,
        C::Hasher: AlgebraicHasher<F>,
    {
        fn add_proof_target_and_conditionally_verify(
            &self,
            builder: &mut CircuitBuilder<F, D>,
            condition: &BoolTarget,
        ) -> ProofWithPublicInputsTarget<D> {
            let proof = builder.add_virtual_proof_with_pis(&self.data.common);
            let vd = builder.constant_verifier_data(&self.data.verifier_only);
            builder
                .conditionally_verify_proof_or_dummy::<C>(
                    *condition,
                    &proof,
                    &vd,
                    &self.data.common,
                )
                .unwrap();
            proof
        }

        fn dummy_leaf(&self) -> crate::utils::dummy::DummyProof<F, C, D> {
            DummyProof::<F, C, D>::new(&self.data.common)
        }
    }

    #[test]
    fn test_dynamic_tree_circuit() {
        let a_circuit = Acircuit::<F, C, D>::new();
        let mut common_data = super::common_data_for_dynamic_tree_circuit::<F, C, D>();
        let dynamic_tree_circuit =
            DynamicTreeCircuit::<F, C, D, _>::new(&a_circuit, &mut common_data);

        let leaf_proof0 = a_circuit.prove(F::ZERO);
        let leaf_proof1 = a_circuit.prove(F::ZERO);

        let left_proof = dynamic_tree_circuit.prove(Some(leaf_proof0), None).unwrap();
        let right_proof = dynamic_tree_circuit.prove(Some(leaf_proof1), None).unwrap();

        let root_proof = dynamic_tree_circuit
            .prove(None, Some((left_proof, right_proof)))
            .unwrap();
        dynamic_tree_circuit.verify(root_proof).unwrap();
    }
}

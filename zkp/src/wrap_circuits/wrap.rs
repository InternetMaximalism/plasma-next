use std::fmt::Display;

use plonky2::{
    field::{extension::Extendable, types::Field},
    hash::hash_types::{HashOut, RichField},
    iop::{
        target::Target,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{AlgebraicHasher, GenericConfig},
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};
use serde::{Deserialize, Serialize};
use starky_keccak::{builder::CircuitBuilderWithKeccak, keccak256_circuit::solidity_keccak256};

use crate::{
    base_circuits::{
        block_tree_circuit::{BlockTreeCircuit, BlockTreePublicInputsTarget},
        validity_circuit::{ValidityCircuit, ValidityPublicInputTargets},
    },
    tree_circuits::{
        dynamic_tree_circuit::DynamicTreePublicInputsTarget,
        settlement_tree_circuit::SettlementTreeCircuit,
    },
    utils::h256::{H256Target, H256},
};

pub struct WrapCircuit<F, C, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    pub data: CircuitData<F, C, D>,
    pub validity_proof: ProofWithPublicInputsTarget<D>,
    pub block_tree_proof: ProofWithPublicInputsTarget<D>,
    pub settlement_tree_proof: ProofWithPublicInputsTarget<D>,
}

impl<F, C, const D: usize> WrapCircuit<F, C, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
{
    pub fn new(
        config: CircuitConfig,
        validity_circuit: &ValidityCircuit<F, C, D>,
        block_tree_circuit: &BlockTreeCircuit<F, C, D>,
        settlement_tree_circuit: &SettlementTreeCircuit<F, C, D>,
    ) -> Self {
        let mut builder = CircuitBuilderWithKeccak::<F, D>::new(config);
        // builder.debug_wire_inde=Some(294711);

        let settlement_tree_proof =
            settlement_tree_circuit.add_proof_target_and_verify(&mut builder);
        let settlement_tree_pis =
            DynamicTreePublicInputsTarget::from_pis(&settlement_tree_proof.public_inputs);

        let validity_proof = validity_circuit.add_proof_target_and_verify(&mut builder);
        let block_tree_proof = block_tree_circuit.add_proof_target_and_verify(&mut builder);
        let validity_pis = ValidityPublicInputTargets::from_pis(&validity_proof.public_inputs);
        let block_tree_pis = BlockTreePublicInputsTarget::from_pis(&block_tree_proof.public_inputs);
        let block_hash = validity_pis.block_hash;
        block_hash.connect(&mut builder, block_tree_pis.block_hash);
        let block_root = block_tree_pis.block_root;

        // constraint block_root
        builder.connect_hashes(settlement_tree_pis.block_root, block_root);
        let pis = WrapPublicInputsTarget {
            block_hash,
            settlement_root: settlement_tree_pis.hash,
        };
        let pis_hash = pis.keccak_hash(&mut builder);
        let pis_hashout = pis_hash.reduce_to_hash_out_target(&mut builder);
        builder.register_public_inputs(&pis_hashout.elements);

        let data = builder.build::<C>();
        Self {
            data,
            validity_proof,
            block_tree_proof,
            settlement_tree_proof,
        }
    }

    pub fn prove(
        &self,
        validity_proof: ProofWithPublicInputs<F, C, D>,
        block_tree_proof: ProofWithPublicInputs<F, C, D>,
        settlement_tree_proof: ProofWithPublicInputs<F, C, D>,
    ) -> anyhow::Result<ProofWithPublicInputs<F, C, D>> {
        let mut pw = PartialWitness::new();
        pw.set_proof_with_pis_target(&self.validity_proof, &validity_proof);
        pw.set_proof_with_pis_target(&self.block_tree_proof, &block_tree_proof);
        pw.set_proof_with_pis_target(&self.settlement_tree_proof, &settlement_tree_proof);
        self.data.prove(pw)
    }

    pub fn add_proof_target_and_verify(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> ProofWithPublicInputsTarget<D> {
        let proof = builder.add_virtual_proof_with_pis(&self.data.common);
        let vd_target = builder.constant_verifier_data(&self.data.verifier_only);
        builder.verify_proof::<C>(&proof, &vd_target, &self.data.common);
        proof
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WrapPublicInputs {
    pub block_hash: H256,
    pub settlement_root: H256,
}

impl WrapPublicInputs {
    pub fn keccak_hash(&self) -> H256 {
        let mut u32_digits: Vec<u32> = vec![];
        u32_digits.extend(&self.block_hash.to_u32_digits());
        u32_digits.extend(&self.settlement_root.to_u32_digits());
        H256::from_u32_digits(solidity_keccak256(u32_digits).0)
    }

    pub fn to_solidity_pis<F: Field>(&self) -> HashOut<F> {
        self.keccak_hash().reduce_to_hash_out()
    }
}

impl Display for WrapPublicInputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{block_hash: {}, settlement_root: {}}}",
            self.block_hash, self.settlement_root
        )
    }
}

pub struct WrapPublicInputsTarget {
    pub block_hash: H256Target,
    pub settlement_root: H256Target,
}

impl WrapPublicInputsTarget {
    pub fn keccak_hash<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilderWithKeccak<F, D>,
    ) -> H256Target {
        let mut u32_digits: Vec<Target> = vec![];
        u32_digits.extend(&self.block_hash.to_vec());
        u32_digits.extend(&self.settlement_root.to_vec());
        H256Target::from_vec(&builder.keccak256(u32_digits))
    }
}

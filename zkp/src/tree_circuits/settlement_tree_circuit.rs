use plonky2::{
    field::extension::Extendable,
    gates::{noop::NoopGate, random_access::RandomAccessGate},
    hash::hash_types::RichField,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CommonCircuitData, VerifierCircuitTarget},
        config::{AlgebraicHasher, GenericConfig},
        proof::ProofWithPublicInputsTarget,
    },
};

use crate::constants::SETTLEMENT_TREE_PADDING_DEGREE;

use super::{
    dynamic_tree_circuit::DynamicTreeCircuit, settlement_leaf_circuit::SettlementLeafCircuit,
};

pub type SettlementTreeCircuit<F, C, const D: usize> =
    DynamicTreeCircuit<F, C, D, SettlementLeafCircuit<F, C, D>>;

impl<F, C, const D: usize> SettlementTreeCircuit<F, C, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    pub fn add_proof_target_and_verify(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> ProofWithPublicInputsTarget<D> {
        let proof = builder.add_virtual_proof_with_pis(&self.data.common);
        let vd_target = builder.constant_verifier_data(&self.data.verifier_only);
        let inner_vd_target =
            VerifierCircuitTarget::from_slice::<F, D>(&proof.public_inputs, &self.data.common)
                .unwrap();
        builder.connect_hashes(vd_target.circuit_digest, inner_vd_target.circuit_digest);
        builder.connect_merkle_caps(
            &vd_target.constants_sigmas_cap,
            &inner_vd_target.constants_sigmas_cap,
        );
        builder.verify_proof::<C>(&proof, &vd_target, &self.data.common);
        proof
    }
}

pub fn common_data_for_settlement_tree_circuit<
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
    while builder.num_gates() < 1 << SETTLEMENT_TREE_PADDING_DEGREE {
        builder.add_gate(NoopGate, vec![]);
    }
    let common_data = builder.build::<C>().common;
    // dbg!(&common_data);
    common_data
}

#[cfg(test)]
mod tests {

    use plonky2::{
        field::goldilocks_field::GoldilocksField, plonk::config::PoseidonGoldilocksConfig,
    };

    use crate::{
        base_circuits::{block_tree_circuit::BlockTreeCircuit, withdraw_circuit::WithdrawCircuit},
        common::address::Address,
        random::withdraw::generate_random_settlement,
        tree_circuits::settlement_leaf_circuit::SettlementLeafCircuit,
    };

    use super::{common_data_for_settlement_tree_circuit, SettlementTreeCircuit};

    const D: usize = 2;
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;

    #[test]
    fn test_settlemnt_tree_circuit() {
        let mut rng = rand::thread_rng();
        let num_blocks = 2;
        let num_transfers = 2;
        let recipient = Address::rand(&mut rng);

        let block_tree_circuit = BlockTreeCircuit::<F, C, D>::new();
        let withdraw_circuit = WithdrawCircuit::<F, C, D>::new(&block_tree_circuit);

        let random_settlement_proof = generate_random_settlement(
            &block_tree_circuit,
            &withdraw_circuit,
            &mut rng,
            num_blocks,
            num_transfers,
            &[recipient],
        )[0]
        .clone();

        let settlement_leaf_circuit = SettlementLeafCircuit::<F, C, D>::new(&withdraw_circuit);
        let settlement_leaf_proof = settlement_leaf_circuit
            .prove(
                &random_settlement_proof.block_root,
                &random_settlement_proof.block_merkle_proof_for_withdraw,
                &random_settlement_proof.block_merkle_proof_for_evidence,
                &random_settlement_proof.withdraw_proof,
                &random_settlement_proof.transfer_info,
            )
            .unwrap();

        let mut common_data = common_data_for_settlement_tree_circuit::<F, C, D>();
        let settlemet_tree_circuit =
            SettlementTreeCircuit::<F, C, D>::new(&settlement_leaf_circuit, &mut common_data);
        let node_proof = settlemet_tree_circuit
            .prove(Some(settlement_leaf_proof), None)
            .unwrap();
        settlemet_tree_circuit
            .prove(None, Some((node_proof.clone(), node_proof.clone())))
            .unwrap();
    }
}

use anyhow::ensure;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOut, RichField},
    plonk::{
        circuit_data::CircuitConfig,
        config::{AlgebraicHasher, GenericConfig, GenericHashOut},
        proof::ProofWithPublicInputs,
    },
};

use crate::{
    base_circuits::{
        block_tree_circuit::{BlockTreeCircuit, BlockTreePublicInputs},
        validity_circuit::{ValidityCircuit, ValidityPublicInputs},
    },
    tree_circuits::dynamic_tree_circuit::DynamicTreePublicInputs,
    wrap_circuits::{
        wrap::{WrapCircuit, WrapPublicInputs},
        wrap2::Wrap2Circuit,
    },
};

use super::settlement_processor::SettlementProcessor;

pub struct WrapProcessor<F, C, OuterC, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    OuterC: GenericConfig<D, F = F>,
{
    pub wrap_circuit: WrapCircuit<F, C, D>,
    pub wrap2_circuit: Wrap2Circuit<F, C, OuterC, D>,
}

impl<F, C, OuterC, const D: usize> WrapProcessor<F, C, OuterC, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    OuterC: GenericConfig<D, F = F>,
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    pub fn new(
        inner_config: CircuitConfig,
        outer_config: CircuitConfig,
        validity_circuit: &ValidityCircuit<F, C, D>,
        block_tree_circuit: &BlockTreeCircuit<F, C, D>,
        settlement_processor: &SettlementProcessor<F, C, D>,
    ) -> Self {
        let wrap_circuit = WrapCircuit::<F, C, D>::new(
            inner_config,
            validity_circuit,
            block_tree_circuit,
            &settlement_processor.settlement_tree_processor.node_circuit,
        );
        let wrap2_circuit = Wrap2Circuit::<F, C, OuterC, D>::new(outer_config, &wrap_circuit);
        Self {
            wrap_circuit,
            wrap2_circuit,
        }
    }

    pub fn validation(
        &self,
        validity_circuit: &ValidityCircuit<F, C, D>,
        block_tree_circuit: &BlockTreeCircuit<F, C, D>,
        settlement_processor: &SettlementProcessor<F, C, D>,
        validity_proof: &ProofWithPublicInputs<F, C, D>,
        block_tree_proof: &ProofWithPublicInputs<F, C, D>,
        settlement_tree_proof: &ProofWithPublicInputs<F, C, D>,
    ) -> anyhow::Result<()> {
        let block_root = validate_balance_block_proof(
            validity_circuit,
            block_tree_circuit,
            validity_proof,
            block_tree_proof,
        )?;

        settlement_processor
            .settlement_tree_processor
            .node_circuit
            .verify(settlement_tree_proof.clone())
            .map_err(|_| anyhow::anyhow!("settlement tree proof verification failed"))?;

        let settlement_tree_pis =
            DynamicTreePublicInputs::from_pis(&settlement_tree_proof.public_inputs);
        ensure!(
            settlement_tree_pis.block_root == block_root,
            "block root of settlement tree proof and block tree proof mismatch: {:?} != {:?}",
            settlement_tree_pis.block_root,
            block_root,
        );
        Ok(())
    }

    pub fn wrap(
        &self,
        validity_circuit: &ValidityCircuit<F, C, D>,
        block_tree_circuit: &BlockTreeCircuit<F, C, D>,
        settlement_processor: &SettlementProcessor<F, C, D>,
        validity_proof: ProofWithPublicInputs<F, C, D>,
        block_tree_proof: ProofWithPublicInputs<F, C, D>,
        settlement_tree_proof: ProofWithPublicInputs<F, C, D>,
    ) -> anyhow::Result<(WrapPublicInputs, ProofWithPublicInputs<F, OuterC, D>)> {
        self.validation(
            validity_circuit,
            block_tree_circuit,
            settlement_processor,
            &validity_proof,
            &block_tree_proof,
            &settlement_tree_proof,
        )?;
        let validity_pis = ValidityPublicInputs::from_pis(&validity_proof.public_inputs);
        let block_hash = validity_pis.block_hash;
        let settlement_tree_pis =
            DynamicTreePublicInputs::from_pis(&settlement_tree_proof.public_inputs);

        let wrap_pis = WrapPublicInputs {
            block_hash,
            settlement_root: settlement_tree_pis.hash,
        };
        let wrap_proof =
            self.wrap_circuit
                .prove(validity_proof, block_tree_proof, settlement_tree_proof)?;
        let wrap2_proof = self.wrap2_circuit.prove(&wrap_proof).unwrap(); // this should not fail
        assert!(wrap2_proof.public_inputs[0..4] == wrap_pis.to_solidity_pis::<F>().to_vec());
        Ok((wrap_pis, wrap2_proof))
    }
}

pub fn validate_balance_block_proof<F, C, const D: usize>(
    validity_circuit: &ValidityCircuit<F, C, D>,
    block_tree_circuit: &BlockTreeCircuit<F, C, D>,
    validity_proof: &ProofWithPublicInputs<F, C, D>,
    block_tree_proof: &ProofWithPublicInputs<F, C, D>,
) -> anyhow::Result<HashOut<F>>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    // validation of proofs
    ensure!(
        validity_circuit.verify(validity_proof.clone()).is_ok(),
        "balance proof verification failed"
    );
    ensure!(
        block_tree_circuit.verify(block_tree_proof.clone()).is_ok(),
        "block tree proof verification failed"
    );

    let validity_pis = ValidityPublicInputs::from_pis(&validity_proof.public_inputs);
    let block_tree_pis = BlockTreePublicInputs::from_pis(&block_tree_proof.public_inputs);
    ensure!(
        validity_pis.block_hash == block_tree_pis.block_hash,
        "block hash of validity_proof and block_tree_proof mismatch"
    );
    Ok(block_tree_pis.block_root)
}

#[cfg(test)]
mod tests {
    use plonky2::plonk::{
        circuit_data::CircuitConfig,
        config::{GenericConfig, PoseidonGoldilocksConfig},
    };
    use rand::seq::SliceRandom;
    use stark_verifier::bn254_poseidon::plonky2_config::Bn254PoseidonGoldilocksConfig;

    use crate::{
        base_circuits::{
            block_tree_circuit::BlockTreeCircuit, spent_circuit::SpentCircuit,
            validity_circuit::ValidityCircuit,
        },
        common::{address::Address, asset::Assets},
        processors::{
            block_processor::BlockProcessor,
            settlement_processor::{SettlementMerkleProof, SettlementProcessor},
        },
        random::transfers::generate_random_transfers,
        tree_circuits::tree_processor::ProofWithHash,
    };

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type OuterC = Bn254PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;
    use super::WrapProcessor;

    fn generate_settlement_root_proof() -> (
        BlockProcessor<F, C, D>,
        ValidityCircuit<F, C, D>,
        SettlementProcessor<F, C, D>,
        (ProofWithHash<F, C, D>, Vec<SettlementMerkleProof>),
    ) {
        let mut rng = rand::thread_rng();
        let recipient = Address::rand(&mut rng);
        let latest_block_number = 2;
        let transfers_vec =
            generate_random_transfers::<F, _>(&mut rng, latest_block_number, 4, &[recipient]);
        let spent_circuit = SpentCircuit::new();
        let validity_circuit = ValidityCircuit::new(&spent_circuit);
        let block_tree_circuit = BlockTreeCircuit::new();
        let mut block_processor = BlockProcessor::<F, C, D>::new();

        let mut transfer_info = vec![];
        let mut deposits = vec![Assets::rand_full(&mut rng)];
        deposits.resize(transfers_vec.len(), Assets::default());
        for (transfers, deposit) in transfers_vec.iter().zip(deposits.iter()) {
            let res = block_processor
                .generate_block(&spent_circuit, transfers, deposit)
                .unwrap();
            block_processor
                .tick(&validity_circuit, &block_tree_circuit, &res.spent_proof)
                .unwrap();
            transfer_info.extend(res.transfer_info);
        }
        transfer_info.shuffle(&mut rng);

        let block_tree_snapshot = block_processor.get_block_tree_snapshot();
        let block_tree_proof_snapshot = block_processor.get_block_tree_proof().unwrap();

        let mut settlement_processor = SettlementProcessor::<F, C, D>::new(&block_tree_circuit);
        settlement_processor.initialize(&block_tree_snapshot);

        let mut settlement_witnesses = vec![];
        for info in &transfer_info {
            let withdraw_proof = settlement_processor
                .append_withdraw_proof(
                    &block_tree_circuit,
                    &block_tree_snapshot,
                    &block_tree_proof_snapshot,
                    &[info.clone()],
                    &None,
                )
                .unwrap();
            settlement_witnesses.push((withdraw_proof, info.clone()));
        }

        for w in &settlement_witnesses {
            settlement_processor
                .add(&block_tree_snapshot, &w.0, &w.1)
                .unwrap();
        }
        let proof = settlement_processor.finalize().unwrap();
        (
            block_processor,
            validity_circuit,
            settlement_processor,
            proof,
        )
    }

    #[test]
    fn test_wrap_processor() {
        let block_tree_circuit = BlockTreeCircuit::new();
        let (block_processor, validity_circuit, settlement_processor, root_proof) =
            generate_settlement_root_proof();
        let inner_config = CircuitConfig::standard_inner_stark_verifier_config();
        let outer_config = CircuitConfig::standard_stark_verifier_config();

        let wrap_processor = WrapProcessor::<F, C, OuterC, D>::new(
            inner_config,
            outer_config,
            &validity_circuit,
            &block_tree_circuit,
            &settlement_processor,
        );
        let (pis, _proof) = wrap_processor
            .wrap(
                &validity_circuit,
                &block_tree_circuit,
                &settlement_processor,
                block_processor.get_validity_proof().unwrap(),
                block_processor.get_block_tree_proof().unwrap(),
                root_proof.0.proof,
            )
            .unwrap();
        for (i, merkle_proof) in root_proof.1.iter().enumerate() {
            println!("merkle_proof {}: {}", i, merkle_proof);
        }
        println!("wrap pis: {}", pis);
        println!("pis hash {:?}", pis.to_solidity_pis::<F>());
        dbg!(wrap_processor.wrap2_circuit.data.common.degree_bits());
    }
}

use halo2_proofs::{
    dev::MockProver,
    halo2curves::bn256::{Bn256, Fr, G1Affine},
    plonk::{keygen_pk, keygen_vk, ProvingKey},
    poly::{commitment::Params, kzg::commitment::ParamsKZG},
};
use halo2_solidity_verifier::BatchOpenScheme::Bdfg21;
use halo2_solidity_verifier::SolidityGenerator;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::PrimeField64},
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
};
use stark_verifier::{
    bn254_poseidon::plonky2_config::{
        standard_inner_stark_verifier_config, standard_stark_verifier_config,
        Bn254PoseidonGoldilocksConfig,
    },
    chip::native_chip::{test_utils::create_proof_checked, utils::goldilocks_to_fe},
    types::{common_data::CommonData, proof::ProofValues, verification_key::VerificationKeyValues},
    verifier_circuit::{ProofTuple, Verifier},
};
use std::{fs::File, io::Write};
use zkp::{
    base_circuits::{
        block_tree_circuit::BlockTreeCircuit, spent_circuit::SpentCircuit,
        validity_circuit::ValidityCircuit,
    },
    common::{asset::Assets, transfer::Transfer},
    processors::{
        block_processor::BlockProcessor, settlement_processor::SettlementProcessor,
        wrap_processor::WrapProcessor,
    },
};

const D: usize = 2;
type F = GoldilocksField;
type OuterC = Bn254PoseidonGoldilocksConfig;

lazy_static::lazy_static! {
    static ref SRS_PATH: String = std::env::var("SRS_PATH").unwrap_or_else(|_| "srs.dat".to_string());
}

pub struct SnarkProcessor {
    pub srs: ParamsKZG<Bn256>,
    pub pk: ProvingKey<G1Affine>,
    pub vk: VerificationKeyValues<Fr>,
    pub common_data: CommonData<Fr>,
}

pub struct ProofResult {
    pub proof: String,
    pub instance: Vec<String>,
}

const DEGREE: u32 = 20;

impl SnarkProcessor {
    pub fn setup_srs() {
        let mut rng = rand::rngs::OsRng;
        let srs = ParamsKZG::<Bn256>::setup(DEGREE, &mut rng);
        let mut file = File::create(SRS_PATH.as_str()).unwrap();
        srs.write(&mut file).unwrap();
    }

    pub fn setup(dummy_proof_tuple: ProofTuple<GoldilocksField, OuterC, 2>) {
        let srs = {
            let mut file = File::open(SRS_PATH.as_str()).expect("failed to open srs.dat");
            ParamsKZG::<Bn256>::read(&mut file).unwrap()
        };

        let (proof_with_public_inputs, vd, cd) = dummy_proof_tuple;
        let proof = ProofValues::<Fr, 2>::from(proof_with_public_inputs.proof);
        let instances = proof_with_public_inputs
            .public_inputs
            .iter()
            .map(|e| goldilocks_to_fe(*e))
            .collect::<Vec<Fr>>();
        let vk = VerificationKeyValues::from(vd.clone());
        let common_data = CommonData::from(cd);
        let circuit = Verifier::new(proof, instances.clone(), vk, common_data);
        let mock_prover = MockProver::run(DEGREE, &circuit, vec![instances.clone()]).unwrap();
        mock_prover.assert_satisfied();

        // generates EVM verifier
        let vk = keygen_vk(&srs, &circuit).unwrap();
        let generator = SolidityGenerator::new(&srs, &vk, Bdfg21, instances.len());
        let (verifier_solidity, vk_solidity) = generator.render_separately().unwrap();
        {
            // save verifier solidity and vk_solidity
            let mut file = File::create("Halo2Verifier.sol").unwrap();
            file.write_all(verifier_solidity.as_bytes()).unwrap();
            let mut file = File::create("Halo2VerifyingKey.sol").unwrap();
            file.write_all(vk_solidity.as_bytes()).unwrap();
        }
    }

    pub fn load(
        dummy_proof_tuple: ProofTuple<GoldilocksField, Bn254PoseidonGoldilocksConfig, 2>,
    ) -> Self {
        let (proof_with_public_inputs, vd, cd) = dummy_proof_tuple;
        let proof = ProofValues::<Fr, 2>::from(proof_with_public_inputs.proof);
        let instances = proof_with_public_inputs
            .public_inputs
            .iter()
            .map(|e| goldilocks_to_fe(*e))
            .collect::<Vec<Fr>>();
        let vk = VerificationKeyValues::from(vd.clone());
        let common_data = CommonData::from(cd);
        let circuit = Verifier::new(proof, instances.clone(), vk.clone(), common_data.clone());
        let srs = {
            let mut file = File::open(SRS_PATH.as_str()).expect("failed to open srs.dat");
            ParamsKZG::<Bn256>::read(&mut file).unwrap()
        };
        let _vk = keygen_vk(&srs, &circuit).unwrap();
        let pk = keygen_pk(&srs, _vk, &circuit).unwrap();
        Self {
            srs,
            pk,
            vk,
            common_data,
        }
    }

    pub fn prove(
        &self,
        proof_with_public_inputs: ProofWithPublicInputs<F, Bn254PoseidonGoldilocksConfig, D>,
    ) -> ProofResult {
        let proof = ProofValues::<Fr, 2>::from(proof_with_public_inputs.proof);
        let instances = proof_with_public_inputs
            .public_inputs
            .iter()
            .map(|e| goldilocks_to_fe(*e))
            .collect::<Vec<Fr>>();
        let circuit = Verifier::new(
            proof,
            instances.clone(),
            self.vk.clone(),
            self.common_data.clone(),
        );
        let mut rng = rand::thread_rng();
        let proof =
            create_proof_checked(&self.srs, &self.pk, circuit.clone(), &instances, &mut rng);
        let proof_hex = "0x".to_string() + &hex::encode(proof);
        let instance_str = proof_with_public_inputs
            .public_inputs
            .iter()
            .map(|e| format!("{}", e.to_canonical_u64()))
            .collect::<Vec<String>>();

        ProofResult {
            proof: proof_hex,
            instance: instance_str,
        }
    }
}

pub fn generate_proof_tuple_and_data() -> (
    ProofTuple<F, Bn254PoseidonGoldilocksConfig, D>,
    CircuitData<F, Bn254PoseidonGoldilocksConfig, D>,
) {
    type C = PoseidonGoldilocksConfig;
    let spent_circuit = SpentCircuit::<F, C, D>::new();
    let validity_circuit = ValidityCircuit::new(&spent_circuit);
    let block_tree_circuit = BlockTreeCircuit::new();
    let mut block_processor = BlockProcessor::<F, C, D>::new();

    let block_info = block_processor
        .generate_block(&spent_circuit, &[Transfer::default()], &Assets::default())
        .unwrap();
    block_processor
        .tick(
            &validity_circuit,
            &block_tree_circuit,
            &block_info.spent_proof,
        )
        .unwrap();
    let mut settlement_processor = SettlementProcessor::new(&block_tree_circuit);
    let withdraw_proof = settlement_processor
        .append_withdraw_proof(
            &block_tree_circuit,
            &block_processor.block_tree,
            &block_processor.get_block_tree_proof().unwrap(),
            &block_info.transfer_info,
            &None,
        )
        .unwrap();
    let evidence_transfer_info = block_info.transfer_info[0].clone();

    settlement_processor.initialize(&block_processor.get_block_tree_snapshot());
    settlement_processor
        .add(
            &block_processor.block_tree,
            &withdraw_proof,
            &evidence_transfer_info,
        )
        .unwrap();
    let (settlement_proof, _) = settlement_processor.finalize().unwrap();
    let inner_config = standard_inner_stark_verifier_config();
    let outer_config = standard_stark_verifier_config();

    let wrap_processor = WrapProcessor::<F, C, OuterC, D>::new(
        inner_config,
        outer_config,
        &validity_circuit,
        &block_tree_circuit,
        &settlement_processor,
    );
    let (_pis, proof) = wrap_processor
        .wrap(
            &validity_circuit,
            &block_tree_circuit,
            &settlement_processor,
            block_processor.get_validity_proof().unwrap(),
            block_processor.get_block_tree_proof().unwrap(),
            settlement_proof.proof,
        )
        .unwrap();
    (
        (
            proof,
            wrap_processor.wrap2_circuit.data.verifier_only.clone(),
            wrap_processor.wrap2_circuit.data.common.clone(),
        ),
        wrap_processor.wrap2_circuit.data,
    )
}

#[cfg(test)]
mod tests {
    use super::{generate_proof_tuple_and_data, SnarkProcessor};

    #[test]
    fn test_generate_proof_tuple_and_data() {
        generate_proof_tuple_and_data();
    }

    #[test]
    fn test_snark_processor_setup_srs() {
        SnarkProcessor::setup_srs();
    }

    #[test]
    fn test_snark_processor_setup() {
        SnarkProcessor::setup(generate_proof_tuple_and_data().0);
    }

    #[test]
    fn test_snark_processor_prove() {
        let (proof_tuple, _) = generate_proof_tuple_and_data();
        let snark_processor = SnarkProcessor::load(proof_tuple.clone());
        println!("start proving");
        let now = std::time::Instant::now();
        let res = snark_processor.prove(proof_tuple.0);
        let elapsed_time = now.elapsed();
        println!("proof time {:?}", elapsed_time);
        println!("{}", res.proof);
        println!("{:?}", res.instance);
    }
}

use crate::utils::logic::enforce_equal_targets_if_enabled;
use plonky2::{
    field::extension::Extendable,
    gates::noop::NoopGate,
    hash::hash_types::RichField,
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, Witness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, CommonCircuitData, VerifierCircuitTarget},
        config::{AlgebraicHasher, GenericConfig},
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
    recursion::{
        cyclic_recursion::check_cyclic_proof_verifier_data, dummy_circuit::cyclic_base_proof,
    },
};
use plonky2_u32::gadgets::arithmetic_u32::CircuitBuilderU32;
use starky_keccak::builder::CircuitBuilderWithKeccak;

use crate::{
    common::{
        asset::{Assets, AssetsTarget},
        block::Block,
    },
    constants::NUM_ASSETS,
    utils::h256::{H256Target, H256},
};

use super::spent_circuit::{SpentCircuit, SpentPublicInputsTarget};

#[derive(Clone, Debug)]
pub struct ValidityPublicInputs {
    pub block_hash: H256,      // the latest block hash
    pub total_spent: Assets,   // the total spent amount so far (including the latest block)
    pub total_deposit: Assets, // the total deposit amount so far (including the latest block)
}

// default value of prev_pis
impl Default for ValidityPublicInputs {
    fn default() -> Self {
        Self {
            block_hash: Block::default().block_hash(),
            total_spent: Assets::default(),
            total_deposit: Assets::default(),
        }
    }
}

impl ValidityPublicInputs {
    pub fn to_vec<F: RichField>(&self) -> Vec<F> {
        let mut vec = self.block_hash.to_vec();
        vec.extend(self.total_spent.to_vec::<F>());
        vec.extend(self.total_deposit.to_vec::<F>());
        vec
    }

    pub fn from_pis<F: RichField>(input: &[F]) -> Self {
        let input = input[0..8 + 8 * NUM_ASSETS + 8 * NUM_ASSETS].to_vec();
        let block_hash = H256::from_vec(&input[0..8]);
        let total_spent = Assets::from_vec(&input[8..8 + 8 * NUM_ASSETS]);
        let total_deposit = Assets::from_vec(&input[8 + 8 * NUM_ASSETS..8 + 8 * NUM_ASSETS * 2]);
        Self {
            block_hash,
            total_spent,
            total_deposit,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ValidityPublicInputTargets {
    pub block_hash: H256Target,
    pub total_spent: AssetsTarget,
    pub total_deposit: AssetsTarget,
}

impl ValidityPublicInputTargets {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        Self {
            block_hash: H256Target::new_unsafe(builder),
            total_spent: AssetsTarget::new_unsafe(builder),
            total_deposit: AssetsTarget::new_unsafe(builder),
        }
    }

    pub fn connect<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: &Self,
    ) {
        self.block_hash.connect(builder, other.block_hash);
        self.total_spent.connect(builder, &other.total_spent);
        self.total_deposit.connect(builder, &other.total_deposit);
    }

    pub fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        value: &ValidityPublicInputs,
    ) -> Self {
        let block_hash = H256Target::constant(builder, value.block_hash);
        let total_spent = AssetsTarget::constant(builder, &value.total_spent);
        let total_deposit = AssetsTarget::constant(builder, &value.total_deposit);
        Self {
            block_hash,
            total_spent,
            total_deposit,
        }
    }

    pub fn set_witness<F: RichField, W: Witness<F>>(
        self,
        pw: &mut W,
        value: &ValidityPublicInputs,
    ) {
        self.block_hash.set_witness(pw, value.block_hash);
        self.total_spent.set_witness(pw, &value.total_spent);
        self.total_deposit.set_witness(pw, &value.total_deposit);
    }

    pub fn from_pis(input: &[Target]) -> Self {
        let input = input[0..8 + 8 * NUM_ASSETS * 2].to_vec();
        let block_hash = H256Target::from_vec(&input[0..8]);
        let total_spent = AssetsTarget::from_vec(&input[8..8 + 8 * NUM_ASSETS]);
        let total_deposit =
            AssetsTarget::from_vec(&input[8 + 8 * NUM_ASSETS..8 + 8 * NUM_ASSETS * 2]);
        Self {
            block_hash,
            total_spent,
            total_deposit,
        }
    }

    pub fn to_vec(&self) -> Vec<Target> {
        let mut vec = self.block_hash.to_vec();
        vec.extend(self.total_spent.to_vec());
        vec.extend(self.total_deposit.to_vec());
        vec
    }
}

pub struct ValidityCircuit<F, C, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    pub data: CircuitData<F, C, D>,
    pub spent_proof: ProofWithPublicInputsTarget<D>,
    pub is_not_first_step: BoolTarget,
    pub prev_proof: ProofWithPublicInputsTarget<D>,
    pub verifier_data_target: VerifierCircuitTarget,
}

impl<F, C, const D: usize> ValidityCircuit<F, C, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
{
    pub fn new(spent_circuit: &SpentCircuit<F, C, D>) -> Self {
        let mut builder = CircuitBuilderWithKeccak::<F, D>::new(CircuitConfig::default());
        let spent_proof = spent_circuit.add_proof_target_and_verify(&mut builder);
        let spent_pis = SpentPublicInputsTarget::from_vec(&spent_proof.public_inputs);
        let prev_pis = ValidityPublicInputTargets::new(&mut builder);
        let total_spent = AssetsTarget::add(&mut builder, &prev_pis.total_spent, &spent_pis.spent);
        let block_hash = spent_pis.block.block_hash(&mut builder);
        let total_deposit = spent_pis.block.total_deposit;
        // assert total_spent <= total_deposit
        let _ = AssetsTarget::sub(&mut builder, &total_deposit, &total_spent);
        let cur_pis = ValidityPublicInputTargets {
            block_hash,
            total_spent,
            total_deposit,
        };
        builder.register_public_inputs(&cur_pis.to_vec());

        let is_not_first_step = builder.add_virtual_bool_target_safe();
        let is_first_step = builder.not(is_not_first_step);

        let mut common_data = common_data_for_validity::<F, C, D>();
        let verifier_data_target = builder.add_verifier_data_public_inputs();
        common_data.num_public_inputs = builder.num_public_inputs();
        let prev_proof = builder.add_virtual_proof_with_pis(&common_data);
        let prev_pis_original = ValidityPublicInputTargets::from_pis(&prev_proof.public_inputs);
        prev_pis.connect(&mut builder, &prev_pis_original);
        builder
            .conditionally_verify_cyclic_proof_or_dummy::<C>(
                is_not_first_step,
                &prev_proof,
                &common_data,
            )
            .unwrap();

        let init_pis =
            ValidityPublicInputTargets::constant(&mut builder, &ValidityPublicInputs::default());
        enforce_equal_targets_if_enabled(
            &mut builder,
            &prev_pis.to_vec(),
            &init_pis.to_vec(),
            is_first_step,
        );
        let data = builder.build();
        assert_eq!(common_data, data.common);
        Self {
            data,
            spent_proof,
            is_not_first_step,
            prev_proof,
            verifier_data_target,
        }
    }

    pub fn prove(
        &self,
        spent_proof: &ProofWithPublicInputs<F, C, D>,
        prev_proof: &Option<ProofWithPublicInputs<F, C, D>>,
    ) -> anyhow::Result<ProofWithPublicInputs<F, C, D>> {
        let mut pw = PartialWitness::<F>::new();
        pw.set_verifier_data_target(&self.verifier_data_target, &self.data.verifier_only);
        pw.set_proof_with_pis_target(&self.spent_proof, spent_proof);

        if prev_proof.is_none() {
            let dummy_proof = cyclic_base_proof(
                &self.data.common,
                &self.data.verifier_only,
                ValidityPublicInputs::default()
                    .to_vec()
                    .into_iter()
                    .enumerate()
                    .collect(),
            );
            pw.set_bool_target(self.is_not_first_step, false);
            pw.set_proof_with_pis_target(&self.prev_proof, &dummy_proof);
        } else {
            pw.set_bool_target(self.is_not_first_step, true);
            pw.set_proof_with_pis_target(&self.prev_proof, prev_proof.as_ref().unwrap());
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

pub fn common_data_for_validity<
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
    let mut builder = CircuitBuilder::<F, D>::new(config);
    let proof = builder.add_virtual_proof_with_pis(&data.common);
    let verifier_data = VerifierCircuitTarget {
        constants_sigmas_cap: builder.add_virtual_cap(data.common.config.fri_config.cap_height),
        circuit_digest: builder.add_virtual_hash(),
    };
    builder.verify_proof::<C>(&proof, &verifier_data, &data.common);
    let zero = builder.zero_u32();
    let _ = builder.add_many_u32(&[zero, zero, zero]);
    let _ = builder.sub_u32(zero, zero, zero);
    let _zero_limbs = [(); 8].map(|_| builder.zero_u32());
    while builder.num_gates() < 1 << 15 {
        builder.add_gate(NoopGate, vec![]);
    }
    builder.build::<C>().common
}

#[cfg(test)]
mod tests {
    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
    use rand::Rng;

    use crate::{
        base_circuits::spent_circuit::{SpentCircuit, SpentValue},
        common::{address::Address, asset::Assets},
        random::transfers::generate_random_transfers,
        utils::h256::H256,
    };

    use super::ValidityCircuit;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_validity_circuit() {
        let spent_circuit = SpentCircuit::<F, C, D>::new();
        let mut rng = rand::thread_rng();
        let recipients = vec![Address::rand(&mut rng)];
        let transfers = generate_random_transfers::<F, _>(&mut rng, 1, 4, &recipients)[0].clone();
        let total_deposit = Assets::rand_full(&mut rng);
        let prev_block_hash = H256::rand(&mut rng);
        let new_block_number: u32 = rng.gen();
        let value = SpentValue::new::<F>(
            &transfers,
            &total_deposit,
            &prev_block_hash,
            new_block_number,
        );
        let spent_proof = spent_circuit.prove(&value).unwrap();

        let validity_circuit = ValidityCircuit::new(&spent_circuit);
        let validity_proof = validity_circuit.prove(&spent_proof, &None).unwrap();

        let now = std::time::Instant::now();
        let validity_proof2 = validity_circuit
            .prove(&spent_proof, &Some(validity_proof.clone()))
            .unwrap();
        println!("validity circuit: {:?}", now.elapsed());
        validity_circuit.verify(validity_proof2).unwrap();
    }
}

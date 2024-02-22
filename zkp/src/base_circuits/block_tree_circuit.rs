use plonky2::{
    field::extension::Extendable,
    gates::noop::NoopGate,
    hash::hash_types::{HashOut, HashOutTarget, RichField},
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, Witness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, CommonCircuitData, VerifierCircuitTarget},
        config::{AlgebraicHasher, GenericConfig, GenericHashOut},
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
    recursion::{
        cyclic_recursion::check_cyclic_proof_verifier_data, dummy_circuit::cyclic_base_proof,
    },
};
use plonky2_u32::gadgets::arithmetic_u32::CircuitBuilderU32;
use starky_keccak::builder::CircuitBuilderWithKeccak;

use crate::{
    common::block::{Block, BlockTarget, BLOCK_VEC_LEN},
    constants::BLOCK_TREE_PADDING_DEGREE,
    utils::{
        h256::{H256Target, H256},
        leafable::{Leafable, LeafableTarget},
        logic::enforce_equal_targets_if_enabled,
        trees::merkle_tree_with_leaves::{
            MerkleProofWithLeaves, MerkleProofWithLeavesTarget, MerkleTreeWithLeaves,
        },
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockTreePublicInputs<F: RichField> {
    pub block: Block,
    pub block_hash: H256,
    pub block_root: HashOut<F>,
}

// A block tree that contains no blocks.
impl<F: RichField> Default for BlockTreePublicInputs<F> {
    fn default() -> Self {
        let mut block_tree = MerkleTreeWithLeaves::<F, Block>::new(32);
        block_tree.push(Block::default());
        let block_root = block_tree.get_root();
        Self {
            block: Block::default(),
            block_hash: Block::default().block_hash(),
            block_root,
        }
    }
}

impl<F: RichField> BlockTreePublicInputs<F> {
    pub fn from_block_tree(input: &BlockTreeValue<F>) -> Self {
        Self {
            block: input.block.clone(),
            block_hash: input.block_hash,
            block_root: input.new_block_root,
        }
    }

    pub fn from_pis(input: &[F]) -> Self {
        let input = input[0..BLOCK_VEC_LEN + 8 + 4].to_vec();
        let block = Block::from_vec(&input[0..BLOCK_VEC_LEN]);
        let block_hash = H256::from_vec(&input[BLOCK_VEC_LEN..BLOCK_VEC_LEN + 8]);
        let block_root = HashOut::from_vec(input[BLOCK_VEC_LEN + 8..BLOCK_VEC_LEN + 12].to_vec());
        Self {
            block,
            block_hash,
            block_root,
        }
    }

    pub fn to_vec(&self) -> Vec<F> {
        let mut result = vec![];
        result.extend(self.block.to_vec::<F>());
        result.extend(self.block_hash.to_vec::<F>());
        result.extend(self.block_root.to_vec());
        assert_eq!(result.len(), BLOCK_VEC_LEN + 12);
        result
    }
}

#[derive(Clone, Debug)]
pub struct BlockTreePublicInputsTarget {
    pub block: BlockTarget,
    pub block_hash: H256Target,
    pub block_root: HashOutTarget,
}

impl BlockTreePublicInputsTarget {
    pub fn assert<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) {
        self.block_hash.assert(builder);
    }

    pub fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        input: &BlockTreePublicInputs<F>,
    ) -> Self {
        Self {
            block: BlockTarget::constant(builder, &input.block),
            block_hash: H256Target::constant(builder, input.block_hash),
            block_root: builder.constant_hash(input.block_root),
        }
    }

    pub fn from_block_tree_target(input: &BlockTreeTarget) -> Self {
        Self {
            block: input.block.clone(),
            block_hash: input.block_hash,
            block_root: input.new_block_root,
        }
    }

    pub fn from_pis(input: &[Target]) -> Self {
        let input = input[0..BLOCK_VEC_LEN + 8 + 4].to_vec();
        let block = BlockTarget::from_vec(&input[0..BLOCK_VEC_LEN]);
        let block_hash = H256Target::from_vec(&input[BLOCK_VEC_LEN..BLOCK_VEC_LEN + 8]);
        let block_root =
            HashOutTarget::from_vec(input[BLOCK_VEC_LEN + 8..BLOCK_VEC_LEN + 12].to_vec());
        Self {
            block,
            block_hash,
            block_root,
        }
    }

    pub fn to_vec(&self) -> Vec<Target> {
        let mut result = vec![];
        result.extend(self.block.to_vec());
        result.extend(self.block_hash.to_vec());
        result.extend(self.block_root.elements);
        assert_eq!(result.len(), BLOCK_VEC_LEN + 12);
        result
    }
}

#[derive(Clone, Debug)]
pub struct BlockTreeValue<F: RichField> {
    pub block: Block,
    pub block_hash: H256,
    pub prev_block_root: HashOut<F>,
    pub new_block_root: HashOut<F>,
    pub merkle_proof: MerkleProofWithLeaves<F, Block>,
}

impl<F: RichField> BlockTreeValue<F> {
    pub fn new(
        block: Block,
        prev_block_root: HashOut<F>,
        new_block_root: HashOut<F>,
        merkle_proof: MerkleProofWithLeaves<F, Block>,
    ) -> Self {
        let empty_leaf = <Block as Leafable<F>>::empty_leaf();
        let block_hash = block.block_hash();
        merkle_proof
            .verify(&empty_leaf, block.block_number as usize, prev_block_root)
            .unwrap();
        merkle_proof
            .verify(&block, block.block_number as usize, new_block_root)
            .unwrap();
        Self {
            block,
            prev_block_root,
            new_block_root,
            block_hash,
            merkle_proof,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BlockTreeTarget {
    pub block: BlockTarget,
    pub prev_block_root: HashOutTarget,
    pub new_block_root: HashOutTarget,
    pub block_hash: H256Target,
    pub merkle_proof: MerkleProofWithLeavesTarget<BlockTarget>,
}

impl BlockTreeTarget {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilderWithKeccak<F, D>,
        block_hash_tree_height: usize,
    ) -> Self {
        let block = BlockTarget::new(builder);
        let merkle_proof = MerkleProofWithLeavesTarget::new(builder, block_hash_tree_height);
        let prev_block_root = builder.add_virtual_hash();
        let new_block_root = builder.add_virtual_hash();
        let empty_leaf = <BlockTarget as LeafableTarget>::empty_leaf(builder);
        let block_hash = block.block_hash(builder);
        merkle_proof.verify::<F, D>(builder, &empty_leaf, block.block_number.0, prev_block_root);
        merkle_proof.verify::<F, D>(builder, &block, block.block_number.0, new_block_root);
        Self {
            block,
            prev_block_root,
            new_block_root,
            block_hash,
            merkle_proof,
        }
    }

    pub fn set_witness<F: RichField>(&self, pw: &mut impl Witness<F>, value: &BlockTreeValue<F>) {
        self.block.set_witness(pw, &value.block);
        pw.set_hash_target(self.prev_block_root, value.prev_block_root);
        pw.set_hash_target(self.new_block_root, value.new_block_root);
        self.block_hash.set_witness(pw, value.block_hash);
        self.merkle_proof.set_witness(pw, &value.merkle_proof);
    }
}

#[derive(Debug)]
pub struct BlockTreeCircuit<F, C, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    pub data: CircuitData<F, C, D>,
    pub target: BlockTreeTarget,
    pub is_not_first_step: BoolTarget,
    pub previous_proof: ProofWithPublicInputsTarget<D>,
    pub verifier_data_target: VerifierCircuitTarget,
}

impl<F, C, const D: usize> BlockTreeCircuit<F, C, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
{
    pub fn new() -> Self {
        let mut builder = CircuitBuilderWithKeccak::<F, D>::new(CircuitConfig::default());
        let target = BlockTreeTarget::new(&mut builder, 32);
        let current_pis = BlockTreePublicInputsTarget::from_block_tree_target(&target);
        builder.register_public_inputs(&current_pis.to_vec());

        let mut common_data = common_data_for_block_tree_circuit::<F, C, D>();
        let verifier_data_target = builder.add_verifier_data_public_inputs();
        common_data.num_public_inputs = builder.num_public_inputs();

        let is_not_first_step = builder.add_virtual_bool_target_safe(); // whether this circuit uses recursive proof or not
        let is_first_step = builder.not(is_not_first_step);

        // We decare and verify the amount deposited before the last block
        let previous_proof = builder.add_virtual_proof_with_pis(&common_data);

        let previous_pis = BlockTreePublicInputsTarget::from_pis(&previous_proof.public_inputs); // cut off the verifier data pis
        previous_pis
            .block_hash
            .connect(&mut builder, target.block.prev_block_hash);

        let initial_pis = BlockTreePublicInputs::default();
        let initial_pis_target = BlockTreePublicInputsTarget::constant(&mut builder, &initial_pis);
        enforce_equal_targets_if_enabled(
            &mut builder,
            &previous_pis.to_vec(),
            &initial_pis_target.to_vec(),
            is_first_step,
        );

        let one = builder.one();
        enforce_equal_targets_if_enabled(
            &mut builder,
            &[target.block.block_number.0],
            &[one],
            is_first_step,
        );

        // Verify a cyclic proof.
        builder
            .conditionally_verify_cyclic_proof_or_dummy::<C>(
                is_not_first_step,
                &previous_proof,
                &common_data,
            )
            .unwrap();
        let circuit_data = builder.build::<C>();

        debug_assert_eq!(circuit_data.common, common_data);

        Self {
            data: circuit_data,
            target,
            is_not_first_step,
            previous_proof,
            verifier_data_target,
        }
    }

    pub fn prove(
        &self,
        value: &BlockTreeValue<F>,
        previous_proof: &Option<ProofWithPublicInputs<F, C, D>>,
    ) -> anyhow::Result<ProofWithPublicInputs<F, C, D>> {
        let mut pw = PartialWitness::<F>::new();
        pw.set_verifier_data_target(&self.verifier_data_target, &self.data.verifier_only);
        self.target.set_witness(&mut pw, value);
        if let Some(previous_proof) = previous_proof {
            pw.set_bool_target(self.is_not_first_step, true);
            pw.set_proof_with_pis_target::<C, D>(&self.previous_proof, &previous_proof);
        } else {
            let previous_pis = BlockTreePublicInputs::default();
            let dummy_proof = cyclic_base_proof(
                &self.data.common,
                &self.data.verifier_only,
                previous_pis.to_vec().into_iter().enumerate().collect(),
            );
            pw.set_bool_target(self.is_not_first_step, false);
            pw.set_proof_with_pis_target::<C, D>(&self.previous_proof, &dummy_proof);
        };
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

// Generates `CommonCircuitData` usable for recursion.
pub fn common_data_for_block_tree_circuit<
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
    while builder.num_gates() < 1 << BLOCK_TREE_PADDING_DEGREE {
        builder.add_gate(NoopGate, vec![]);
    }
    builder.build::<C>().common
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

    use crate::{
        base_circuits::block_tree_circuit::BlockTreePublicInputs,
        common::{asset::Assets, block::Block},
        utils::{h256::H256, trees::merkle_tree_with_leaves::MerkleTreeWithLeaves},
    };

    use super::{BlockTreeCircuit, BlockTreeValue};

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_block_tree_circuit() {
        let block_tree_circuit = BlockTreeCircuit::<F, C, D>::new();

        let mut block_tree = MerkleTreeWithLeaves::<F, Block>::new(32);
        let block0 = Block::default();
        block_tree.push(block0.clone());
        let block_root0 = block_tree.get_root();
        let block1 = Block {
            prev_block_hash: block0.block_hash(),
            transfer_tree_root: H256::default(),
            total_deposit: Assets::default(),
            block_number: 1,
        };
        block_tree.push(block1.clone());
        let block_root1 = block_tree.get_root();
        let block1_merkle_proof = block_tree.prove(block1.block_number as usize);
        let value1 = BlockTreeValue::new(
            block1.clone(),
            block_root0,
            block_root1,
            block1_merkle_proof,
        );

        println!("start proving: block1");
        let now = Instant::now();
        let proof_with_pis1 = block_tree_circuit.prove(&value1, &None).unwrap();
        println!("prove: {:?}", now.elapsed());

        let block2 = Block {
            prev_block_hash: block1.block_hash(),
            block_number: 2,
            transfer_tree_root: H256::default(),
            total_deposit: Assets::default(),
        };
        block_tree.push(block2.clone());
        let block_root2 = block_tree.get_root();
        let block_merkle_proof2 = block_tree.prove(block2.block_number as usize);
        let value2 = BlockTreeValue::new(block2, block_root1, block_root2, block_merkle_proof2);

        println!("start proving: block2");
        let now = Instant::now();
        let proof_with_pis2 = block_tree_circuit
            .prove(&value2, &Some(proof_with_pis1))
            .unwrap();
        println!("prove: {:?}", now.elapsed());
        let pis2 = BlockTreePublicInputs::from_pis(&proof_with_pis2.public_inputs);
        assert_eq!(block_root2, pis2.block_root);
    }
}

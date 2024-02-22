use crate::{
    common::{
        asset::{Assets, AssetsTarget},
        block::{Block, BlockTarget},
        transfer::{calc_transfer_tree, calc_transfer_tree_circuit, Transfer, TransferTarget},
    },
    constants::{NUM_ASSETS, TRANSFER_TREE_HEIGHT},
    utils::h256::{H256Target, H256},
};
use plonky2::{
    field::{extension::Extendable, types::PrimeField64},
    hash::hash_types::RichField,
    iop::{
        target::Target,
        witness::{PartialWitness, Witness},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{AlgebraicHasher, GenericConfig},
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};
use plonky2_u32::{
    gadgets::arithmetic_u32::{CircuitBuilderU32, U32Target},
    witness::WitnessU32,
};

#[derive(Debug)]
pub struct SpentPublicInputs {
    pub block: Block,
    pub spent: Assets,
}

impl SpentPublicInputs {
    pub fn to_vec<F: PrimeField64>(&self) -> Vec<F> {
        let mut vec = Vec::new();
        vec.extend(self.block.to_vec::<F>());
        vec.extend(self.spent.to_vec::<F>());
        assert_eq!(vec.len(), 8 + 8 + 8 * NUM_ASSETS + 1 + 8 * NUM_ASSETS);
        vec
    }

    pub fn from_vec<F: PrimeField64>(input: &[F]) -> Self {
        assert_eq!(input.len(), 8 + 8 + 8 * NUM_ASSETS + 1 + 8 * NUM_ASSETS);
        let block = Block::from_vec(&input[..8 + 8 + 8 * NUM_ASSETS + 1]);
        let spent = Assets::from_vec(&input[8 + 8 + 8 * NUM_ASSETS + 1..]);
        Self { block, spent }
    }
}

pub struct SpentPublicInputsTarget {
    pub block: BlockTarget,
    pub spent: AssetsTarget,
}

impl SpentPublicInputsTarget {
    pub fn to_vec(&self) -> Vec<Target> {
        let mut vec = Vec::new();
        vec.extend(self.block.to_vec());
        vec.extend(self.spent.to_vec());
        assert_eq!(vec.len(), 8 + 8 + 8 * NUM_ASSETS + 1 + 8 * NUM_ASSETS);
        vec
    }

    pub fn from_vec(input: &[Target]) -> Self {
        assert_eq!(input.len(), 8 + 8 + 8 * NUM_ASSETS + 1 + 8 * NUM_ASSETS);
        let block = BlockTarget::from_vec(&input[..8 + 8 + 8 * NUM_ASSETS + 1]);
        let spent = AssetsTarget::from_vec(&input[8 + 8 + 8 * NUM_ASSETS + 1..]);
        Self { block, spent }
    }
}

pub struct SpentValue {
    pub transfers: Vec<Transfer>,  // padded transfers
    pub prev_block_hash: H256,     // previous block hash
    pub new_total_deposit: Assets, // total deposit amount
    pub new_block_number: u32,     // block number
    pub new_block: Block,          // new block
    pub spent: Assets,             // spent amount in the new block
}

impl SpentValue {
    pub fn new<F: RichField>(
        transfers: &[Transfer],
        new_total_deposit: &Assets,
        prev_block_hash: &H256,
        new_block_number: u32,
    ) -> Self {
        assert!(
            transfers.len() <= 1 << TRANSFER_TREE_HEIGHT,
            "too many transfers"
        );
        let (transfer_tree_root, spent) = calc_transfer_tree::<F>(transfers);
        let mut transfers = transfers.to_vec();
        transfers.resize(1 << TRANSFER_TREE_HEIGHT, Transfer::default());
        let new_block = Block {
            prev_block_hash: prev_block_hash.clone(),
            transfer_tree_root: transfer_tree_root.into(),
            total_deposit: new_total_deposit.clone(),
            block_number: new_block_number,
        };
        Self {
            transfers,
            prev_block_hash: prev_block_hash.clone(),
            new_total_deposit: new_total_deposit.clone(),
            new_block_number,
            new_block,
            spent,
        }
    }
}

pub struct SpentTaret {
    pub transfers: Vec<TransferTarget>,  // padded transfers
    pub prev_block_hash: H256Target,     // previous block hash
    pub new_total_deposit: AssetsTarget, // total deposit amount
    pub new_block_number: U32Target,     // block number
    pub new_block: BlockTarget,          // new block
    pub spent: AssetsTarget,             // spent amount in the new block
}

impl SpentTaret {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        let transfers =
            [(); 1 << TRANSFER_TREE_HEIGHT].map(|_| TransferTarget::new_unsafe(builder));
        let (transfer_tree_root, spent) = calc_transfer_tree_circuit(builder, &transfers);
        let new_block = BlockTarget {
            prev_block_hash: H256Target::new_unsafe(builder),
            transfer_tree_root: H256Target::from_hash_out_target(builder, transfer_tree_root),
            total_deposit: AssetsTarget::new_unsafe(builder),
            block_number: builder.add_virtual_u32_target(),
        };
        Self {
            transfers: transfers.to_vec(),
            prev_block_hash: new_block.prev_block_hash.clone(),
            new_total_deposit: new_block.total_deposit.clone(),
            new_block_number: new_block.block_number.clone(),
            new_block,
            spent,
        }
    }

    pub fn set_witness<F: RichField, W: Witness<F>>(&self, pw: &mut W, value: &SpentValue) {
        assert_eq!(self.transfers.len(), value.transfers.len());
        for (target, transfer) in self.transfers.iter().zip(value.transfers.iter()) {
            target.set_witness(pw, transfer);
        }
        self.prev_block_hash.set_witness(pw, value.prev_block_hash);
        self.new_total_deposit
            .set_witness(pw, &value.new_total_deposit);
        pw.set_u32_target(self.new_block_number, value.new_block_number);
        self.new_block.set_witness(pw, &value.new_block);
        self.spent.set_witness(pw, &value.spent);
    }
}

pub struct SpentCircuit<F, C, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    pub data: CircuitData<F, C, D>,
    pub target: SpentTaret,
}

impl<F, C, const D: usize> SpentCircuit<F, C, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
{
    pub fn new() -> Self {
        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::default());
        let target = SpentTaret::new::<F, D>(&mut builder);
        let pis = SpentPublicInputsTarget {
            block: target.new_block.clone(),
            spent: target.spent.clone(),
        };
        builder.register_public_inputs(&pis.to_vec());
        let data = builder.build();
        Self { data, target }
    }

    pub fn prove(&self, value: &SpentValue) -> anyhow::Result<ProofWithPublicInputs<F, C, D>> {
        let mut pw = PartialWitness::<F>::new();
        self.target.set_witness(&mut pw, value);
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

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
    use rand::Rng;

    use crate::{
        common::{address::Address, asset::Assets},
        random::transfers::generate_random_transfers,
        utils::h256::H256,
    };

    use super::{SpentCircuit, SpentValue};

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_spent_circuit() {
        let circuit = SpentCircuit::<F, C, D>::new();
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
        let now = Instant::now();
        let _proof = circuit.prove(&value).unwrap();
        println!("spent circuit: {:?}", now.elapsed());
    }
}

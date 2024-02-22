use plonky2::{
    field::{extension::Extendable, types::PrimeField64},
    hash::{
        hash_types::{HashOut, HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::{target::Target, witness::Witness},
    plonk::{circuit_builder::CircuitBuilder, config::Hasher},
};
use plonky2_u32::{
    gadgets::arithmetic_u32::{CircuitBuilderU32, U32Target},
    witness::WitnessU32,
};
use serde::{Deserialize, Serialize, Serializer};
use starky_keccak::{builder::CircuitBuilderWithKeccak, keccak256_circuit::solidity_keccak256};

use crate::{
    constants::NUM_ASSETS,
    utils::{
        h256::{H256Target, H256},
        leafable::{poseidon_two_to_one, poseidon_two_to_one_swapped, Leafable, LeafableTarget},
        u256::assert_u32_target,
    },
};

use super::asset::{Assets, AssetsTarget};

pub const BLOCK_VEC_LEN: usize = 8 + 8 + 8 * NUM_ASSETS + 1;

fn serialize_block_number<S: Serializer>(x: &u32, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // serialize to decimal string
    s.serialize_str(x.to_string().as_str())
}

fn deserialize_block_number<'de, D>(d: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // deserialize from decimal string
    let s = String::deserialize(d)?;
    s.parse().map_err(serde::de::Error::custom)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub prev_block_hash: H256,
    pub transfer_tree_root: H256,
    pub total_deposit: Assets,
    #[serde(serialize_with = "serialize_block_number")]
    #[serde(deserialize_with = "deserialize_block_number")]
    pub block_number: u32,
}

// genesis block
impl Default for Block {
    fn default() -> Self {
        Self {
            prev_block_hash: H256::default(),
            transfer_tree_root: H256::default(),
            total_deposit: Assets::default(),
            block_number: 0,
        }
    }
}

impl Block {
    /// Returns the block hash.
    pub fn block_hash(&self) -> H256 {
        let input = vec![
            &self.prev_block_hash.to_u32_digits()[..],
            &self.transfer_tree_root.to_u32_digits()[..],
            &self.total_deposit.to_u32_digits()[..],
            &[self.block_number][..],
        ]
        .concat();
        H256::from_u32_digits(solidity_keccak256(input).0)
    }

    pub fn to_vec<F: PrimeField64>(&self) -> Vec<F> {
        let mut result = self.prev_block_hash.to_vec();
        result.extend(&self.transfer_tree_root.to_vec());
        result.extend(&self.total_deposit.to_vec());
        result.extend([F::from_canonical_u32(self.block_number)]);
        assert_eq!(result.len(), BLOCK_VEC_LEN);
        result
    }

    pub fn from_vec<F: PrimeField64>(input: &[F]) -> Self {
        assert_eq!(input.len(), BLOCK_VEC_LEN);
        let prev_block_hash = H256::from_vec(&input[..8]);
        let transfer_tree_root = H256::from_vec(&input[8..16]);
        let total_deposit = Assets::from_vec(&input[16..16 + 8 * NUM_ASSETS]);
        let block_number = input[16 + 8 * NUM_ASSETS].to_canonical_u64() as u32;
        Self {
            prev_block_hash,
            transfer_tree_root,
            total_deposit,
            block_number,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BlockTarget {
    pub prev_block_hash: H256Target,
    pub transfer_tree_root: H256Target,
    pub total_deposit: AssetsTarget,
    pub block_number: U32Target,
}

impl BlockTarget {
    pub fn new_unsafe<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        let prev_block_hash = H256Target::new_unsafe(builder);
        let transfer_tree_root = H256Target::new_unsafe(builder);
        let total_deposit = AssetsTarget::new_unsafe(builder);
        let block_number = builder.add_virtual_u32_target();

        Self {
            prev_block_hash,
            transfer_tree_root,
            total_deposit,
            block_number,
        }
    }

    pub fn assert<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) {
        self.prev_block_hash.assert(builder);
        assert_u32_target(builder, self.block_number.0);
        self.transfer_tree_root.assert(builder);
    }

    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        let target = Self::new_unsafe(builder);
        target.assert(builder);

        target
    }

    pub fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        input: &Block,
    ) -> Self {
        let prev_block_hash = H256Target::constant(builder, input.prev_block_hash);
        let transfer_tree_root = H256Target::constant(builder, input.transfer_tree_root);
        let total_deposit = AssetsTarget::constant(builder, &input.total_deposit);
        let block_number = builder.constant_u32(input.block_number);
        Self {
            prev_block_hash,
            transfer_tree_root,
            total_deposit,
            block_number,
        }
    }

    pub fn connect<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: &Self,
    ) {
        self.prev_block_hash.connect(builder, other.prev_block_hash);
        self.transfer_tree_root
            .connect(builder, other.transfer_tree_root);
        self.total_deposit.connect(builder, &other.total_deposit);
        builder.connect_u32(self.block_number, other.block_number);
    }

    pub fn set_witness<F: RichField>(&self, pw: &mut impl Witness<F>, input: &Block) {
        self.prev_block_hash.set_witness(pw, input.prev_block_hash);
        self.transfer_tree_root
            .set_witness(pw, input.transfer_tree_root);
        self.total_deposit.set_witness(pw, &input.total_deposit);
        pw.set_u32_target(self.block_number, input.block_number);
    }

    /// Returns the block hash.
    pub fn block_hash<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilderWithKeccak<F, D>,
    ) -> H256Target {
        let mut result = self.prev_block_hash.to_vec();
        result.extend(self.transfer_tree_root.to_vec());
        result.extend(self.total_deposit.to_vec());
        result.extend([self.block_number.0]);
        H256Target::wrap_unsafe(builder.keccak256(result))
    }

    pub fn to_vec(&self) -> Vec<Target> {
        let mut result = self.prev_block_hash.to_vec();
        result.extend(self.transfer_tree_root.to_vec());
        result.extend(self.total_deposit.to_vec());
        result.extend([self.block_number.0]);
        assert_eq!(result.len(), BLOCK_VEC_LEN);
        result
    }

    pub fn from_vec(input: &[Target]) -> Self {
        assert_eq!(input.len(), BLOCK_VEC_LEN);
        let prev_block_hash = H256Target::from_vec(&input[..8]);
        let transfer_tree_root = H256Target::from_vec(&input[8..16]);
        let total_deposit = AssetsTarget::from_vec(&input[16..16 + 8 * NUM_ASSETS]);
        let block_number = U32Target(input[16 + 8 * NUM_ASSETS]);
        Self {
            prev_block_hash,
            transfer_tree_root,
            total_deposit,
            block_number,
        }
    }
}

impl<F: RichField> Leafable<F> for Block {
    type HashOut = HashOut<F>;

    fn empty_leaf() -> Self {
        Self::default()
    }

    fn hash(&self) -> HashOut<F> {
        PoseidonHash::hash_no_pad(&self.to_vec())
    }

    fn two_to_one(left: &Self::HashOut, right: &Self::HashOut) -> Self::HashOut {
        PoseidonHash::two_to_one(*left, *right)
    }
}

impl LeafableTarget for BlockTarget {
    type HashOutTarget = HashOutTarget;

    fn empty_leaf<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        Self::constant(builder, &Block::default())
    }

    fn hash<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> HashOutTarget {
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(self.to_vec())
    }

    fn connect_hash<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        x: &Self::HashOutTarget,
        y: &Self::HashOutTarget,
    ) {
        builder.connect_hashes(*x, *y)
    }

    fn two_to_one<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        left: &Self::HashOutTarget,
        right: &Self::HashOutTarget,
    ) -> Self::HashOutTarget {
        poseidon_two_to_one(builder, left, right)
    }

    fn two_to_one_swapped<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        left: &Self::HashOutTarget,
        right: &Self::HashOutTarget,
        swap: plonky2::iop::target::BoolTarget,
    ) -> Self::HashOutTarget {
        poseidon_two_to_one_swapped(builder, left, right, swap)
    }
}

#[cfg(test)]
mod tests {
    use plonky2::{
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::{
            circuit_data::CircuitConfig,
            config::{GenericConfig, PoseidonGoldilocksConfig},
        },
    };
    use starky_keccak::builder::CircuitBuilderWithKeccak;

    use crate::{
        common::{
            asset::Assets,
            block::{Block, BlockTarget},
        },
        utils::{
            h256::H256,
            leafable::{Leafable, LeafableTarget},
        },
    };

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_calc_block_hash_circuit() {
        let block0 = Block::default();
        let block1 = Block {
            block_number: 1,
            prev_block_hash: block0.block_hash(),
            transfer_tree_root: H256::default(),
            total_deposit: Assets::default(),
        };
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilderWithKeccak::<F, D>::new(config);
        let block_header_target = BlockTarget::new(&mut builder);
        let block_hash_target = block_header_target.block_hash(&mut builder);
        let data = builder.build::<C>();

        let mut pw = PartialWitness::new();
        block_header_target.set_witness(&mut pw, &block1);
        block_hash_target.set_witness(&mut pw, block1.block_hash());
        data.prove(pw).unwrap();
    }

    #[test]
    fn test_block_header_leable() {
        let block = Block::default();
        let hash = block.hash();
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilderWithKeccak::<F, D>::new(config);
        let block_header_t = BlockTarget::constant(&mut builder, &block);
        let hash_t = block_header_t.hash(&mut builder);

        let data = builder.build::<C>();
        let mut pw = PartialWitness::new();
        pw.set_hash_target(hash_t, hash);
        data.prove(pw).unwrap();
    }
}

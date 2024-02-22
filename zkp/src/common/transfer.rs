use std::vec;

use std::ops::Add;

use plonky2::{
    field::{extension::Extendable, types::PrimeField64},
    hash::{
        hash_types::{HashOut, HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::{target::Target, witness::Witness},
    plonk::{circuit_builder::CircuitBuilder, config::Hasher},
    util::log2_ceil,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use starky_keccak::{builder::CircuitBuilderWithKeccak, keccak256_circuit::solidity_keccak256};

use crate::{
    constants::TRANSFER_TREE_HEIGHT,
    utils::{
        h256::{H256Target, H256},
        leafable::{poseidon_two_to_one, poseidon_two_to_one_swapped, Leafable, LeafableTarget},
        trees::{
            get_root::get_merkle_root_from_leaves_circuit,
            merkle_tree_with_leaves::MerkleTreeWithLeaves,
        },
        u256::U256,
    },
};

use super::{
    address::{Address, AddressTarget},
    asset::{Asset, AssetTarget, Assets, AssetsTarget},
};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Transfer {
    pub recipient: Address,
    pub asset: Asset,
}

impl Transfer {
    pub fn rand<T: Rng>(rng: &mut T) -> Self {
        let mut amount_not_full = U256::rand(rng);
        amount_not_full.0[0] = 0;
        Self {
            recipient: Address::rand(rng),
            asset: Asset::rand(rng),
        }
    }

    pub fn to_vec<F: PrimeField64>(&self) -> Vec<F> {
        let mut result = vec![];
        result.extend(self.recipient.to_vec::<F>());
        result.extend(self.asset.to_vec::<F>());
        assert_eq!(result.len(), 14);
        result
    }

    pub fn from_vec<F: PrimeField64>(input: &[F]) -> Self {
        assert!(input.len() == 14);
        let recipient = Address::from_vec(&input[0..5]);
        let amount = Asset::from_vec(&input[5..14]);
        Self {
            recipient,
            asset: amount,
        }
    }

    pub fn to_u32_digits(&self) -> Vec<u32> {
        let mut result = Vec::new();
        result.extend(self.recipient.to_u32_digits());
        result.extend(self.asset.amount.to_u32_digits());
        result.extend([self.asset.asset_id as u32]);
        result
    }

    pub fn keccak_hash(&self) -> H256 {
        H256::from_u32_digits(solidity_keccak256(self.to_u32_digits()).0)
    }
}

impl<F: RichField> Leafable<F> for Transfer {
    type HashOut = HashOut<F>;

    fn hash(&self) -> HashOut<F> {
        PoseidonHash::hash_no_pad(&self.to_vec())
    }

    fn empty_leaf() -> Self {
        Self::default()
    }

    fn two_to_one(left: &Self::HashOut, right: &Self::HashOut) -> Self::HashOut {
        PoseidonHash::two_to_one(*left, *right)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TransferTarget {
    pub recipient: AddressTarget,
    pub amount: AssetTarget,
}

impl TransferTarget {
    pub fn to_vec(&self) -> Vec<Target> {
        let mut result = Vec::new();
        result.extend(self.recipient.to_vec());
        result.extend(self.amount.to_vec());
        result
    }

    pub fn from_vec(input: &[Target]) -> Self {
        assert!(input.len() == 14);
        let recipient = AddressTarget::from_vec(&input[0..5]);
        let amount = AssetTarget::from_vec(&input[5..14]);
        Self { recipient, amount }
    }
}

impl LeafableTarget for TransferTarget {
    type HashOutTarget = HashOutTarget;

    fn hash<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> HashOutTarget {
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(self.to_vec())
    }

    fn empty_leaf<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        let recipient = AddressTarget::constant(builder, Address::default());
        let amount = AssetTarget::constant(builder, Asset::default());

        Self { recipient, amount }
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

impl TransferTarget {
    pub fn new_unsafe<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        let recipient = AddressTarget::new_unsafe(builder);
        let amount = AssetTarget::new_unsafe(builder);

        Self { recipient, amount }
    }

    pub fn assert<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) {
        self.recipient.assert(builder);
        self.amount.assert(builder);
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
        value: Transfer,
    ) -> Self {
        let recipient = AddressTarget::constant(builder, value.recipient);
        let amount = AssetTarget::constant(builder, value.asset);

        Self { recipient, amount }
    }

    pub fn set_witness<F: PrimeField64>(&self, pw: &mut impl Witness<F>, input: &Transfer) {
        self.recipient.set_witness::<F>(pw, input.recipient);
        self.amount.set_witness::<F>(pw, &input.asset);
    }

    pub fn connect<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: &Self,
    ) {
        self.recipient.connect(builder, &other.recipient);
        self.amount.connect(builder, &other.amount);
    }

    pub fn to_u32_digits(&self) -> Vec<Target> {
        let mut result = Vec::new();
        result.extend(self.recipient.to_u32_digits());
        result.extend(self.amount.amount.to_vec());
        result.extend([self.amount.asset_id.0]);
        result
    }

    pub fn keccak_hash<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilderWithKeccak<F, D>,
    ) -> H256Target {
        H256Target::from_vec(&builder.keccak256(self.to_u32_digits()))
    }
}

pub fn get_total_transfer_amount(transfers: &[Transfer]) -> Assets {
    transfers.iter().fold(Assets::default(), |acc, transfer| {
        let transfer_amount = Assets::from_asset(&transfer.asset);
        Assets::add(acc, transfer_amount)
    })
}

pub fn calc_transfer_tree<F: RichField>(transfers: &[Transfer]) -> (HashOut<F>, Assets) {
    assert!(transfers.len() <= 1 << TRANSFER_TREE_HEIGHT);
    let mut transfer_tree = MerkleTreeWithLeaves::<F, Transfer>::new(TRANSFER_TREE_HEIGHT);
    for transfer in transfers {
        transfer_tree.push(*transfer);
    }
    let transfer_tree_root = transfer_tree.get_root();
    let total_amount = get_total_transfer_amount(transfers);
    (transfer_tree_root, total_amount)
}

pub fn calc_transfer_tree_circuit<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    transfers: &[TransferTarget],
) -> (HashOutTarget, AssetsTarget) {
    let transfer_hashes = transfers
        .iter()
        .map(|v| v.hash(builder))
        .collect::<Vec<_>>();
    let mut transfer_tree_root = get_merkle_root_from_leaves_circuit::<F, _, D>(
        builder,
        TRANSFER_TREE_HEIGHT,
        &transfer_hashes,
    );
    let log_n_transfers = log2_ceil(transfers.len());

    let mut default_hash = <Transfer as Leafable<F>>::empty_leaf().hash();
    for _ in 0..log_n_transfers {
        default_hash = Transfer::two_to_one(&default_hash, &default_hash);
    }
    for _ in log_n_transfers..TRANSFER_TREE_HEIGHT {
        let default_hash_target = builder.constant_hash(default_hash);
        transfer_tree_root =
            TransferTarget::two_to_one(builder, &transfer_tree_root, &default_hash_target);
        default_hash = Transfer::two_to_one(&default_hash, &default_hash);
    }

    let total_amount_sent = transfers.iter().fold(
        AssetsTarget::constant(builder, &Assets::default()),
        |acc, transfer| {
            let transfer_amount = AssetsTarget::from_asset(builder, &transfer.amount);
            AssetsTarget::add(builder, &acc, &transfer_amount)
        },
    );

    (transfer_tree_root, total_amount_sent)
}

#[cfg(test)]
mod tests {

    use std::time::Instant;

    use plonky2::{
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::{
            circuit_builder::CircuitBuilder,
            circuit_data::CircuitConfig,
            config::{GenericConfig, PoseidonGoldilocksConfig},
        },
    };
    use starky_keccak::builder::CircuitBuilderWithKeccak;

    use crate::{
        common::{address::Address, asset::Asset},
        constants::TRANSFER_TREE_HEIGHT,
        utils::u256::U256,
    };

    use super::{calc_transfer_tree, calc_transfer_tree_circuit, Transfer, TransferTarget};

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_calc_transfer_tree() {
        let mut rng = rand::thread_rng();
        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::default());

        let transfers: Vec<Transfer> = (0..1 << TRANSFER_TREE_HEIGHT)
            .map(|_| Transfer::rand(&mut rng))
            .collect();

        let now = Instant::now();
        let (transfer_tree_root, total_amount) = calc_transfer_tree::<F>(&transfers);
        println!("calc_transfer_tree: {:?}", now.elapsed());

        let transfers_t = transfers
            .iter()
            .map(|v| TransferTarget::constant(&mut builder, *v))
            .collect::<Vec<_>>();
        let (transfer_tree_root_t, total_amount_t) =
            calc_transfer_tree_circuit(&mut builder, &transfers_t);

        let data = builder.build::<C>();
        let mut pw = PartialWitness::<F>::new();
        total_amount_t.set_witness(&mut pw, &total_amount);
        pw.set_hash_target(transfer_tree_root_t, transfer_tree_root);
        data.prove(pw).unwrap();
    }

    #[test]
    fn test_keccak_hash() {
        let mut rng = rand::thread_rng();
        let mut builder = CircuitBuilderWithKeccak::<F, D>::new(CircuitConfig::default());

        let transfer = Transfer::rand(&mut rng);
        let transfer_hash = transfer.keccak_hash();
        let transfer_t = TransferTarget::constant(&mut builder, transfer);
        let transfer_hash_t = transfer_t.keccak_hash(&mut builder);

        let data = builder.build::<C>();
        let mut pw = PartialWitness::<F>::new();
        transfer_t.set_witness(&mut pw, &transfer);

        transfer_hash_t.set_witness(&mut pw, transfer_hash);
        data.prove(pw).unwrap();
    }

    #[test]
    fn test_transfer_solidity_hash() {
        let transfer = Transfer {
            recipient: Address::from_hex("70997970c51812dc3A010c7d01b50e0d17dc79C8"),
            asset: Asset {
                asset_id: 0,
                amount: U256::from(10),
            },
        };
        let transfer_hash = transfer.keccak_hash();
        println!("transfer_hash: {}", transfer_hash);
    }
}

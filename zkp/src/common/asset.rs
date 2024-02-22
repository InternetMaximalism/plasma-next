use std::{
    fmt::Display,
    iter::Sum,
    ops::{Add, AddAssign},
};

use plonky2::{
    field::{
        extension::Extendable,
        types::{Field, Field64, PrimeField64},
    },
    hash::hash_types::RichField,
    iop::{
        target::{BoolTarget, Target},
        witness::WitnessWrite,
    },
    plonk::{circuit_builder::CircuitBuilder, config::AlgebraicHasher},
};
use plonky2_u32::{
    gadgets::arithmetic_u32::{CircuitBuilderU32, U32Target},
    witness::WitnessU32,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::utils::{
    logic::{connect_targets, is_equal_targets},
    u256::{U256Target, U256},
};

use crate::constants::NUM_ASSETS;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub asset_id: u32,
    pub amount: U256,
}

impl PartialOrd for Asset {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.asset_id != other.asset_id {
            return None;
        }

        self.amount.partial_cmp(&other.amount)
    }
}

impl Asset {
    pub fn rand<T: Rng>(rng: &mut T) -> Self {
        let mut amount_not_full = U256::rand(rng);
        amount_not_full.0[0] = 0;
        amount_not_full.0[1] = 0;
        Self {
            asset_id: rng.gen_range(0..NUM_ASSETS) as u32,
            amount: amount_not_full,
        }
    }

    pub fn to_vec<F: Field>(&self) -> Vec<F> {
        let mut result: Vec<F> = Vec::new();
        result.extend(self.amount.to_vec::<F>());
        result.push(F::from_canonical_u32(self.asset_id));
        assert_eq!(result.len(), 9);
        result
    }

    pub fn from_vec<F: PrimeField64>(input: &[F]) -> Self {
        assert!(input.len() == 9);
        let amount = U256::from_vec(&input[0..8]);
        let asset_id = input[8].to_canonical_u64() as u32;
        Self { asset_id, amount }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AssetTarget {
    pub asset_id: U32Target,
    pub amount: U256Target,
}

impl AssetTarget {
    pub fn new_unsafe<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        let asset_id = builder.add_virtual_u32_target();
        let amount = U256Target::new_unsafe(builder);

        Self { asset_id, amount }
    }

    pub fn assert<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) {
        self.amount.assert(builder);
    }

    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        let target = Self::new_unsafe(builder);
        target.assert(builder);

        target
    }

    pub fn set_witness<F: PrimeField64>(&self, pw: &mut impl WitnessU32<F>, asset: &Asset) {
        pw.set_u32_target(self.asset_id, asset.asset_id);
        self.amount.set_witness(pw, asset.amount);
    }

    pub fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        asset: Asset,
    ) -> Self {
        Self {
            asset_id: builder.constant_u32(asset.asset_id),
            amount: U256Target::constant(builder, asset.amount),
        }
    }

    pub fn connect<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: &Self,
    ) {
        connect_targets(builder, &self.to_vec(), &other.to_vec())
    }

    pub fn is_equal<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        x: &Self,
        y: &Self,
    ) -> BoolTarget {
        is_equal_targets(builder, &x.to_vec(), &y.to_vec())
    }

    pub fn to_vec(self) -> Vec<Target> {
        let mut result = self.amount.to_vec();
        result.push(self.asset_id.0);
        assert_eq!(result.len(), 9);
        result
    }

    pub fn from_vec(input: &[Target]) -> Self {
        assert!(input.len() == 9);
        let amount = U256Target::from_vec(&input[0..8]);
        let asset_id = U32Target(input[8]);
        Self { amount, asset_id }
    }
}

pub const ASSETS_VEC_LEN: usize = 8 * NUM_ASSETS;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assets(pub [U256; NUM_ASSETS]);

impl Assets {
    pub fn to_vec<F: Field64>(&self) -> Vec<F> {
        let result = self.0.iter().flat_map(|v| v.to_vec()).collect::<Vec<_>>();
        assert_eq!(result.len(), ASSETS_VEC_LEN);
        result
    }

    pub fn from_vec<F: PrimeField64>(input: &[F]) -> Self {
        assert!(input.len() == ASSETS_VEC_LEN);
        let input = input.chunks(8).map(U256::from_vec).collect::<Vec<_>>();
        Self(input.try_into().unwrap())
    }

    pub fn to_u32_digits(&self) -> [u32; 8 * NUM_ASSETS] {
        self.0
            .iter()
            .flat_map(|v| v.to_u32_digits())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    pub fn from_asset(asset: &Asset) -> Self {
        let mut result = Self::default();
        result.0[asset.asset_id as usize] = asset.amount;
        result
    }

    pub fn rand<T: Rng>(rng: &mut T) -> Self {
        let amounts = (0..NUM_ASSETS)
            .map(|_| {
                let mut amount_not_full = U256::rand(rng);
                amount_not_full.0[0] = 0;
                amount_not_full
            })
            .collect::<Vec<_>>();
        Self(amounts.try_into().unwrap())
    }

    pub fn rand_full<T: Rng>(rng: &mut T) -> Self {
        let amounts = (0..NUM_ASSETS).map(|_| U256::rand(rng)).collect::<Vec<_>>();
        Self(amounts.try_into().unwrap())
    }
}

impl PartialOrd for Assets {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let mut result = std::cmp::Ordering::Equal;
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            match a.partial_cmp(b) {
                Some(std::cmp::Ordering::Equal) => {}
                Some(std::cmp::Ordering::Greater) => {
                    if result == std::cmp::Ordering::Less {
                        return None;
                    }
                    result = std::cmp::Ordering::Greater;
                }
                Some(std::cmp::Ordering::Less) => {
                    if result == std::cmp::Ordering::Greater {
                        return None;
                    }
                    result = std::cmp::Ordering::Less;
                }
                None => return None,
            }
        }
        Some(result)
    }
}

impl AddAssign<&Assets> for Assets {
    fn add_assign(&mut self, rhs: &Assets) {
        for (a, b) in self.0.iter_mut().zip(rhs.0.iter()) {
            *a += *b;
        }
    }
}

impl Add for &Assets {
    type Output = Assets;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result += rhs;

        result
    }
}

impl AddAssign for Assets {
    fn add_assign(&mut self, rhs: Self) {
        *self += &rhs;
    }
}

impl Add for Assets {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}

impl AddAssign<Asset> for Assets {
    fn add_assign(&mut self, rhs: Asset) {
        self.0[rhs.asset_id as usize] += rhs.amount;
    }
}

impl Add<Asset> for Assets {
    type Output = Self;

    fn add(mut self, rhs: Asset) -> Self::Output {
        self += rhs;

        self
    }
}

impl Add<Asset> for &Assets {
    type Output = Assets;

    fn add(self, rhs: Asset) -> Self::Output {
        self.clone() + rhs
    }
}

impl Sum for Assets {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |a, b| a + b)
    }
}

impl Display for Assets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, v) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", v)?;
        }
        write!(f, "]")
    }
}

#[derive(Clone, Debug)]
pub struct AssetsTarget(pub [U256Target; NUM_ASSETS]);

impl AssetsTarget {
    pub fn new_unsafe<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        Self([(); NUM_ASSETS].map(|_| U256Target::new_unsafe(builder)))
    }

    pub fn assert<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) {
        for v in self.0.iter() {
            v.assert(builder);
        }
    }

    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        let target = Self::new_unsafe(builder);
        target.assert(builder);

        target
    }

    pub fn set_witness<F: RichField>(&self, pw: &mut impl WitnessWrite<F>, assets: &Assets) {
        for (a, b) in self.0.iter().zip(assets.0.iter()) {
            a.set_witness(pw, *b);
        }
    }

    pub fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        assets: &Assets,
    ) -> Self {
        Self(
            assets
                .0
                .iter()
                .map(|amount| U256Target::constant(builder, *amount))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    pub fn connect<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: &Self,
    ) {
        connect_targets(builder, &self.to_vec(), &other.to_vec())
    }

    pub fn is_equal<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        x: &Self,
        y: &Self,
    ) -> BoolTarget {
        is_equal_targets(builder, &x.to_vec(), &y.to_vec())
    }

    pub fn to_vec(&self) -> Vec<Target> {
        let result = self.0.iter().flat_map(|v| v.to_vec()).collect::<Vec<_>>();
        assert_eq!(result.len(), ASSETS_VEC_LEN);
        result
    }

    pub fn from_vec(input: &[Target]) -> Self {
        assert!(input.len() == ASSETS_VEC_LEN);
        let input = input
            .chunks(8)
            .map(U256Target::from_vec)
            .collect::<Vec<_>>();
        Self(input.try_into().unwrap())
    }

    /// Returns `x + y`
    pub fn add<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        x: &Self,
        y: &Self,
    ) -> Self {
        Self(
            x.0.iter()
                .zip(y.0.iter())
                .map(|(l, r)| l.add(builder, r))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    /// Returns `x - y`
    pub fn sub<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        x: &Self,
        y: &Self,
    ) -> Self {
        Self(
            x.0.iter()
                .zip(y.0.iter())
                .map(|(l, r)| l.sub(builder, r))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    pub fn from_asset<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        asset: &AssetTarget,
    ) -> Self {
        Self(
            (0..NUM_ASSETS)
                .map(|i| {
                    let i_t = builder.constant(F::from_canonical_usize(i));
                    let selected = builder.is_equal(asset.asset_id.0, i_t);
                    let zero = U256Target::constant(builder, U256::default());
                    U256Target::conditionally_select(builder, asset.amount, zero, selected)
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    /// less than or equal
    /// Returns `self <= other`
    pub fn le<F: RichField + Extendable<D>, H: AlgebraicHasher<F>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: &AssetsTarget,
    ) -> BoolTarget {
        let mut result = builder._true();
        for (l, r) in self.0.iter().zip(other.0.iter()) {
            let tmp = l.le(builder, r);
            result = builder.and(result, tmp);
        }
        result
    }
}

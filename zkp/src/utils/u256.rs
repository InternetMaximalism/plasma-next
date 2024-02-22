use num_bigint::BigUint;
use num_traits::Num;
use plonky2::{
    field::{
        extension::Extendable,
        types::{Field, PrimeField64},
    },
    hash::hash_types::RichField,
    iop::{
        target::{BoolTarget, Target},
        witness::{Witness, WitnessWrite},
    },
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_u32::gadgets::arithmetic_u32::{CircuitBuilderU32, U32Target};
use plonky2_u32::gadgets::multiple_comparison::list_le_circuit;
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::h256::H256;

/// Store 32bit per one field.
/// Represent the number as big endian.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct U256(pub [u32; 8]);

impl std::fmt::Display for U256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b: BigUint = (*self).into();
        let s = b.to_str_radix(10);
        write!(f, "{}", s)
    }
}

impl Serialize for U256 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let b: BigUint = (*self).into();
        let s = b.to_str_radix(10);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for U256 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let b = BigUint::from_str_radix(&s, 10).map_err(serde::de::Error::custom)?;
        let u: U256 = b.try_into().unwrap();
        Ok(u)
    }
}

impl PartialOrd for U256 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for U256 {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Iterator::cmp(self.0.iter(), other.0.iter())
    }
}

impl From<u64> for U256 {
    fn from(v: u64) -> Self {
        Self([0, 0, 0, 0, 0, 0, (v >> 32) as u32, v as u32])
    }
}

impl From<H256> for U256 {
    fn from(value: H256) -> Self {
        Self::from_be_bytes(value.0)
    }
}

impl From<U256> for BigUint {
    fn from(value: U256) -> Self {
        BigUint::from_bytes_be(&value.to_be_bytes())
    }
}

impl TryFrom<BigUint> for U256 {
    type Error = anyhow::Error;
    fn try_from(value: BigUint) -> anyhow::Result<Self> {
        let mut limbs = value.to_bytes_le();
        anyhow::ensure!(limbs.len() <= 32);
        limbs.extend(vec![0; 32 - limbs.len()]);
        Ok(U256::from_le_bytes(limbs.try_into().unwrap()))
    }
}

impl U256 {
    pub fn max() -> Self {
        Self([u32::MAX; 8])
    }

    pub fn does_overflow_after_add(&self, other: &Self) -> bool {
        let max_minus_self = Self::max() - *self;
        max_minus_self < *other
    }

    /// NOTICE: without 0x-prefix
    pub fn from_hex(hex: &str) -> Self {
        let mut result = [0u8; 32];
        hex::decode_to_slice(hex, &mut result).unwrap();

        Self::from_be_bytes(result)
    }

    pub fn from_be_bytes(bytes: [u8; 32]) -> U256 {
        let result = bytes
            .chunks(4)
            .map(|c| u32::from_be_bytes(c.try_into().unwrap()))
            .collect::<Vec<_>>();

        Self(result.try_into().unwrap())
    }

    pub fn from_le_bytes(bytes: [u8; 32]) -> U256 {
        let result = bytes
            .chunks(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
            .rev()
            .collect::<Vec<_>>();
        Self(result.try_into().unwrap())
    }

    pub fn to_be_bytes(self) -> [u8; 32] {
        let mut result = vec![];
        for limb in self.0.iter() {
            result.extend_from_slice(&limb.to_be_bytes());
        }

        result.try_into().unwrap()
    }

    pub fn to_le_bytes(self) -> [u8; 32] {
        let mut result = self.to_be_bytes();
        result.reverse();

        result
    }

    pub fn to_u32_digits(self) -> [u32; 8] {
        self.0
    }

    pub fn from_u32_digits(input: [u32; 8]) -> Self {
        Self(input)
    }

    pub fn to_vec<F: Field>(self) -> Vec<F> {
        let result = self.0.map(F::from_canonical_u32).to_vec();
        assert_eq!(result.len(), 8);
        result
    }

    pub fn from_vec<F: PrimeField64>(input: &[F]) -> Self {
        let raw: [F; 8] = input.try_into().unwrap();

        Self(raw.map(|v| v.to_canonical_u64() as u32))
    }

    pub fn rand<T: Rng>(rng: &mut T) -> Self {
        Self(rng.gen())
    }
}

impl num::Zero for U256 {
    fn zero() -> Self {
        Self::default()
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|v| *v == 0)
    }
}

impl std::ops::Add for U256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result_limbs = vec![];
        let mut carry = 0u64;
        for (a, b) in self.0.iter().rev().zip(rhs.0.iter().rev()) {
            let c = carry + *a as u64 + *b as u64;
            let result = c as u32;
            carry = c >> 32;
            result_limbs.push(result);
        }

        // Carry should be zero here.
        assert_eq!(carry, 0, "U256 addition overflow occured");

        result_limbs.reverse();

        Self(result_limbs.try_into().unwrap())
    }
}

impl std::ops::AddAssign for U256 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub for U256 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result_limbs = vec![];

        let mut borrow = 0i64;
        for (a, b) in self.0.iter().rev().zip(rhs.0.iter().rev()) {
            let c = *a as i64 - *b as i64 + borrow;
            let result = c as u32;
            borrow = (c >> 32) as i32 as i64;
            result_limbs.push(result);
        }

        // Borrow should be zero here.
        assert_eq!(borrow, 0, "U256 sub underflow occured");

        result_limbs.reverse();

        Self(result_limbs.try_into().unwrap())
    }
}

impl std::ops::SubAssign for U256 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

/// Solidity uint256
/// big endian
#[derive(Copy, Clone, Debug)]
pub struct U256Target(pub [U32Target; 8]);

pub fn assert_u32_target<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    target: Target,
) {
    let _ = builder.split_le(target, 32);
}

impl U256Target {
    pub fn new_unsafe<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        Self([(); 8].map(|_| builder.add_virtual_u32_target()))
    }

    pub fn assert<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) {
        for target in self.0.iter() {
            assert_u32_target(builder, target.0);
        }
    }

    pub fn set_witness<F: Field>(&self, pw: &mut impl WitnessWrite<F>, witness: U256) {
        for (target, value) in self.0.iter().zip(witness.0.iter()) {
            pw.set_target(target.0, F::from_canonical_u32(*value));
        }
    }

    pub fn get_witness<F: PrimeField64>(&self, pw: &impl Witness<F>) -> U256 {
        let mut result = vec![];
        for target in self.0.iter() {
            let value = pw.get_target(target.0);
            result.push(value.to_canonical_u64() as u32);
        }

        U256(result.try_into().unwrap())
    }

    pub fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        value: U256,
    ) -> Self {
        Self(value.0.map(|v| builder.constant_u32(v)))
    }

    pub fn connect<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: Self,
    ) {
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            builder.connect_u32(*a, *b);
        }
    }

    pub fn is_equal<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: Self,
    ) -> BoolTarget {
        let mut result = builder._true();
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            let eq = builder.is_equal(a.0, b.0);
            result = builder.and(result, eq);
        }

        result
    }

    pub fn select<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        b: BoolTarget,
        x: Self,
        y: Self,
    ) -> Self {
        Self(
            x.0.iter()
                .zip(y.0.iter())
                .map(|(x, y)| U32Target(builder.select(b, x.0, y.0)))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    /// if condition { lhs } else { rhs }
    pub fn conditionally_select<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        lhs: Self,
        rhs: Self,
        condition: BoolTarget,
    ) -> Self {
        Self(
            lhs.0
                .iter()
                .zip(rhs.0.iter())
                .map(|(a, b)| U32Target(builder._if(condition, a.0, b.0)))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    pub fn to_vec(&self) -> Vec<Target> {
        self.0.iter().map(|t| t.0).collect()
    }

    pub fn from_vec(input: &[Target]) -> Self {
        assert_eq!(input.len(), 8);

        Self(
            input
                .iter()
                .map(|v| U32Target(*v))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    pub fn add<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: &Self,
    ) -> Self {
        let zero = builder.zero_u32();

        let mut combined_limbs = vec![];
        let mut carry = zero;
        for (a, b) in self.0.iter().rev().zip(other.0.iter().rev()) {
            let (new_limb, new_carry) = builder.add_many_u32(&[carry, *a, *b]);
            carry = new_carry;
            combined_limbs.push(new_limb);
        }

        // Carry should be zero here.
        builder.connect_u32(carry, zero);

        combined_limbs.reverse();

        Self(combined_limbs.try_into().unwrap())
    }

    pub fn sub<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: &Self,
    ) -> Self {
        let zero = builder.zero_u32();
        let mut result_limbs = vec![];

        let mut borrow = zero;
        for (a, b) in self.0.iter().rev().zip(other.0.iter().rev()) {
            let (result, new_borrow) = builder.sub_u32(*a, *b, borrow);
            result_limbs.push(result);
            borrow = new_borrow;
        }

        // Borrow should be zero here.
        builder.connect_u32(borrow, zero);

        Self(result_limbs.try_into().unwrap())
    }

    pub fn split_le<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Vec<BoolTarget> {
        self.0
            .iter()
            .rev()
            .flat_map(|e| builder.split_le(e.0, 32))
            .collect::<Vec<_>>()
    }

    /// less than or equal
    /// Returns `self <= other`
    pub fn le<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: &Self,
    ) -> BoolTarget {
        list_le_circuit(
            builder,
            self.0.iter().map(|t| t.0).collect(),
            other.0.iter().map(|t| t.0).collect(),
            32,
        )
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use plonky2::{
        field::types::{Field, Sample},
        hash::{hashing::hash_n_to_hash_no_pad, poseidon::PoseidonPermutation},
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::{
            circuit_builder::CircuitBuilder,
            circuit_data::CircuitConfig,
            config::{GenericConfig, PoseidonGoldilocksConfig},
        },
    };
    use rand::thread_rng;

    use crate::utils::{
        h256::{H256Target, H256},
        u256::{assert_u32_target, U256},
    };

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_display() {
        let u = U256::from(123u64);
        let h: H256 = u.into();
        println!("u: {}", u);
        println!("h: {}", h);
    }

    #[test]
    fn test_to_biguint() {
        let u = U256::from(123u64);
        let u_b: BigUint = u.into();
        let u_r: U256 = u_b.try_into().unwrap();
        assert_eq!(u, u_r);
    }

    #[test]
    fn test_u256_order() {
        let a = U256([0, 0, 0, 0, 2, 0, 0, 0]);
        let b = U256([0, 0, 0, 1, 1, 0, 0, 0]);
        assert!(a < b);
    }

    #[test]
    fn test_convert_both_h256_and_hash_out() {
        let hash_out = hash_n_to_hash_no_pad::<F, PoseidonPermutation<F>>(&[F::rand()]);
        let h256 = H256::from(hash_out);
        let hash_out2 = h256.reduce_to_hash_out::<F>();
        let h256_2 = H256::from(hash_out2);
        assert_eq!(hash_out, hash_out2);
        assert_eq!(h256, h256_2);
    }

    #[test]
    fn test_convert_hash_out_to_h256() {
        let hash_out = hash_n_to_hash_no_pad::<F, PoseidonPermutation<F>>(&[F::rand()]);
        let h256 = H256::from(hash_out);
        let mut builder = CircuitBuilder::new(CircuitConfig::standard_recursion_config());
        let hash_out_t = builder.constant_hash(hash_out);
        let h256_t = H256Target::from_hash_out_target(&mut builder, hash_out_t);
        let mut pw = PartialWitness::new();
        h256_t.set_witness(&mut pw, h256);
        let data = builder.build::<C>();
        data.prove(pw).unwrap();
    }

    #[test]
    fn test_convert_h256_to_hash_out() {
        let mut rng = rand::thread_rng();
        let h256 = H256::rand(&mut rng);
        let hash_out = h256.reduce_to_hash_out::<F>();
        let mut builder = CircuitBuilder::new(CircuitConfig::standard_recursion_config());
        let h256_t = H256Target::constant(&mut builder, h256);
        let hash_out_t = h256_t.reduce_to_hash_out_target(&mut builder);
        let mut pw = PartialWitness::new();
        pw.set_hash_target(hash_out_t, hash_out);
        let data = builder.build::<C>();
        data.prove(pw).unwrap();
    }

    #[test]
    fn test_u256_add_sub() {
        let a = U256([0, 0, 0, 1, 2, 0, 0, 0]);
        let b = U256([0, 0, 0, 0, u32::MAX, 0, 0, 0]);
        let c = U256([0, 0, 0, 2, 1, 0, 0, 0]);
        let d = U256([0, 0, 0, 0, 3, 0, 0, 0]);
        assert_eq!(a + b, c);
        assert_eq!(a - b, d);
    }

    #[test]
    #[should_panic]
    fn test_u256_sub_underflow() {
        let a = U256([0, 0, 0, 1, 2, 0, 0, 0]);
        let b = U256([0, 0, 0, 0, u32::MAX, 0, 0, 0]);

        _ = b - a;
    }

    #[test]
    fn test_assert_u32_target() {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;

        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::new(config);
        let target = builder.add_virtual_target();
        assert_u32_target(&mut builder, target);
        let data = builder.build::<C>();

        // NOTICE: `u64::MAX` is equivalent to 2^32 - 2 on the Goldilocks field.
        // Hence `witness` is a 32-bit value.
        let witness = F::from_noncanonical_u64(u64::MAX);

        let mut pw = PartialWitness::new();
        pw.set_target(target, witness);

        let _ = data.prove(pw).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_panic_assert_u32_target_case1() {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;

        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::new(config);
        let target = builder.add_virtual_target();
        assert_u32_target(&mut builder, target);
        let data = builder.build::<C>();

        let witness = F::from_canonical_u64(1 << 32);

        let mut pw = PartialWitness::new();
        pw.set_target(target, witness);

        let _ = data.prove(pw).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_panic_assert_u32_target_case2() {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;

        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::new(config);
        let target = builder.add_virtual_target();
        assert_u32_target(&mut builder, target);
        let data = builder.build::<C>();

        let witness = F::NEG_ONE;

        let mut pw = PartialWitness::new();
        pw.set_target(target, witness);

        let _ = data.prove(pw).unwrap();
    }

    #[test]
    fn test_serialize_u256() {
        let mut rng = thread_rng();
        let x = U256::rand(&mut rng);
        let x_str = serde_json::to_string(&x).unwrap();
        let x_recovered: U256 = serde_json::from_str(&x_str).unwrap();
        assert_eq!(x, x_recovered);
        println!("{}", x_str);
    }
}

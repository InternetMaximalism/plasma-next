use plonky2::{
    field::{
        extension::Extendable,
        types::{Field, PrimeField64},
    },
    hash::hash_types::{HashOut, HashOutTarget, RichField},
    iop::{
        target::{BoolTarget, Target},
        witness::{Witness, WitnessWrite},
    },
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_u32::gadgets::arithmetic_u32::{CircuitBuilderU32, U32Target};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::u256::{assert_u32_target, U256};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct H256(pub [u8; 32]);

impl std::fmt::Display for H256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let h = hex::encode(self.0);
        write!(f, "{}", h)
    }
}

impl Serialize for H256 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let h = hex::encode(self.0);
        serializer.serialize_str(&h)
    }
}

impl<'de> Deserialize<'de> for H256 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let mut result = [0u8; 32];
        hex::decode_to_slice(s, &mut result).map_err(serde::de::Error::custom)?;
        Ok(Self(result))
    }
}

impl H256 {
    pub fn rand<R: Rng>(rng: &mut R) -> Self {
        Self(rng.gen())
    }

    /// NOTICE: without 0x-prefix
    pub fn from_hex(hex: &str) -> Self {
        let mut result = [0u8; 32];
        hex::decode_to_slice(hex, &mut result).unwrap();

        Self(result)
    }

    pub fn to_vec<F: Field>(&self) -> Vec<F> {
        self.to_u32_digits()
            .map(|c| F::from_canonical_u32(c))
            .to_vec()
    }

    pub fn from_vec<F: PrimeField64>(input: &[F]) -> Self {
        let raw: [F; 8] = input.try_into().unwrap();

        Self::from_u32_digits(raw.map(|v| v.to_canonical_u64() as u32))
    }

    pub fn to_u32_digits(&self) -> [u32; 8] {
        self.0
            .chunks(4)
            .map(|c| u32::from_be_bytes(c.try_into().unwrap()))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    pub fn from_u32_digits(input: [u32; 8]) -> Self {
        let mut result = vec![];
        for limb in input.iter() {
            result.extend_from_slice(&limb.to_be_bytes());
        }

        Self(result.try_into().unwrap())
    }
}

impl<F: PrimeField64> From<HashOut<F>> for H256 {
    fn from(value: HashOut<F>) -> Self {
        let value = value
            .elements
            .iter()
            .flat_map(|&e| {
                let e = e.to_canonical_u64();
                let low = e as u32;
                let high = (e >> 32) as u32;
                [high, low]
            })
            .collect::<Vec<_>>();
        Self::from_u32_digits(value.try_into().unwrap())
    }
}

// H256 -> HashOut<F> is destructive and does not necessarily match HashOut<F> -> H256.
impl H256 {
    pub fn reduce_to_hash_out<F: Field>(&self) -> HashOut<F> {
        let elements = self
            .to_u32_digits()
            .chunks(2)
            .map(|chunk| {
                let low = chunk[1];
                let high = chunk[0];
                let e = ((high as u64) << 32) + (low as u64);
                F::from_noncanonical_u64(e)
            })
            .collect::<Vec<_>>();
        HashOut {
            elements: elements.try_into().unwrap(),
        }
    }
}

impl From<U256> for H256 {
    fn from(value: U256) -> Self {
        Self(value.to_be_bytes())
    }
}

/// Solidity bytes32
#[derive(Copy, Clone, Debug)]
pub struct H256Target(pub [U32Target; 8]);

impl H256Target {
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

    pub fn wrap_unsafe(target: [Target; 8]) -> Self {
        Self(target.map(U32Target))
    }

    pub fn set_witness<F: Field>(&self, pw: &mut impl WitnessWrite<F>, witness: H256) {
        for (target, value) in self.0.iter().zip(witness.0.chunks(4)) {
            let mut tmp = [0u8; 4];
            tmp.copy_from_slice(value);
            pw.set_target(target.0, F::from_canonical_u32(u32::from_be_bytes(tmp)));
        }
    }

    pub fn get_witness<F: PrimeField64>(&self, pw: &impl Witness<F>) -> H256 {
        let mut result = vec![];
        for target in self.0.iter() {
            let value = pw.get_target(target.0);
            result.push(value.to_canonical_u64() as u32);
        }

        H256(
            result
                .iter()
                .flat_map(|v| v.to_be_bytes())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    pub fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        value: H256,
    ) -> Self {
        let result = value
            .0
            .chunks(4)
            .map(|value| {
                let mut tmp = [0u8; 4];
                tmp.copy_from_slice(value);

                builder.constant_u32(u32::from_be_bytes(tmp))
            })
            .collect::<Vec<_>>();

        Self(result.try_into().unwrap())
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

    pub fn from_hash_out_target<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        input: HashOutTarget,
    ) -> Self {
        let x = input
            .elements
            .iter()
            .flat_map(|e| {
                let (low, high) = builder.split_low_high(*e, 32, 32);
                [U32Target(high), U32Target(low)]
            })
            .collect::<Vec<_>>();
        Self(x.try_into().unwrap())
    }

    pub fn reduce_to_hash_out_target<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> HashOutTarget {
        let mut result = vec![];
        for chunk in self.0.chunks(2) {
            let low = chunk[1];
            let high = chunk[0];
            result.push(builder.mul_const_add(F::from_canonical_u64(1 << 32), high.0, low.0));
        }
        HashOutTarget {
            elements: result.try_into().unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use plonky2::{
        field::types::Sample,
        hash::{hashing::hash_n_to_hash_no_pad, poseidon::PoseidonPermutation},
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::{
            circuit_builder::CircuitBuilder,
            circuit_data::CircuitConfig,
            config::{GenericConfig, PoseidonGoldilocksConfig},
        },
    };

    use crate::utils::h256::H256;

    use super::H256Target;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

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
    fn test_serialize_h256() {
        let mut rng = rand::thread_rng();
        let x = H256::rand(&mut rng);
        let x_str = serde_json::to_string(&x).unwrap();
        let x_recovered: H256 = serde_json::from_str(&x_str).unwrap();
        assert_eq!(x, x_recovered);
        println!("{}", x_str);
    }
}

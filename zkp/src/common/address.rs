use plonky2::{
    field::{
        extension::Extendable,
        types::{Field64, PrimeField64},
    },
    hash::hash_types::RichField,
    iop::{
        target::Target,
        witness::{Witness, WitnessWrite},
    },
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_u32::gadgets::arithmetic_u32::{CircuitBuilderU32, U32Target};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::utils::u256::assert_u32_target;

pub const ADDRESS_VEC_LEN: usize = 5;

/// Address of user account. This corresponds to the index of the world state tree.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Address([u32; 5]);

impl Address {
    pub fn rand<T: Rng>(rng: &mut T) -> Self {
        Self(rng.gen())
    }

    pub fn to_vec<F: Field64>(self) -> Vec<F> {
        let result = self.0.map(|x| F::from_canonical_u32(x)).to_vec();
        assert_eq!(result.len(), ADDRESS_VEC_LEN);
        result
    }

    pub fn from_vec<F: PrimeField64>(input: &[F]) -> Self {
        assert!(input.len() == ADDRESS_VEC_LEN);
        let input = input
            .iter()
            .map(|x| x.to_canonical_u64() as u32)
            .collect::<Vec<_>>();
        Self(input.try_into().unwrap())
    }

    pub fn to_be_bytes(self) -> [u8; 20] {
        let mut result = vec![];
        for limb in self.0.iter() {
            result.extend_from_slice(&limb.to_be_bytes());
        }
        result.try_into().unwrap()
    }

    pub fn from_be_bytes(bytes: [u8; 20]) -> Self {
        let result = bytes
            .chunks(4)
            .map(|c| u32::from_be_bytes(c.try_into().unwrap()))
            .collect::<Vec<_>>();
        Self(result.try_into().unwrap())
    }

    pub fn from_hex(hex: &str) -> Self {
        let bytes = hex::decode(hex).unwrap();
        Self::from_be_bytes(bytes.try_into().unwrap())
    }

    /// big endian
    pub fn to_u32_digits(self) -> [u32; 5] {
        self.0
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let h = hex::encode(self.to_be_bytes());
        write!(f, "{}", h)
    }
}

impl Serialize for Address {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let h = hex::encode(self.to_be_bytes());
        serializer.serialize_str(&h)
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let mut result = [0u8; 20];
        hex::decode_to_slice(s, &mut result).map_err(serde::de::Error::custom)?;
        Ok(Self::from_be_bytes(result))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AddressTarget([U32Target; 5]); // TODO: Convert from a public key of BLS.

impl AddressTarget {
    pub fn new_unsafe<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        Self([(); 5].map(|_| builder.add_virtual_u32_target()))
    }

    pub fn assert<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) {
        self.0.iter().for_each(|x| assert_u32_target(builder, x.0))
    }

    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        let target = Self::new_unsafe(builder);
        target.assert(builder);
        target
    }

    pub fn set_witness<F: Field64>(&self, pw: &mut impl WitnessWrite<F>, address: Address) {
        self.0
            .iter()
            .zip(address.0)
            .for_each(|(x, y)| pw.set_target(x.0, F::from_canonical_u32(y)));
    }

    pub fn get_witness<F: PrimeField64>(&self, pw: &impl Witness<F>) -> Address {
        let mut result = vec![];
        for target in self.0.iter() {
            let value = pw.get_target(target.0);
            result.push(value.to_canonical_u64() as u32);
        }
        Address(result.try_into().unwrap())
    }

    pub fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        address: Address,
    ) -> Self {
        Self(
            address
                .0
                .iter()
                .map(|x| builder.constant_u32(*x))
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
        self.0
            .iter()
            .zip(other.0)
            .for_each(|(x, y)| builder.connect_u32(*x, y));
    }

    pub fn to_vec(self) -> Vec<Target> {
        let result = self.0.iter().map(|x| x.0).collect::<Vec<_>>();
        assert_eq!(result.len(), ADDRESS_VEC_LEN);
        result
    }

    pub fn from_vec(input: &[Target]) -> Self {
        assert!(input.len() == ADDRESS_VEC_LEN);
        let input = input.iter().map(|x| U32Target(*x)).collect::<Vec<_>>();
        Self(input.try_into().unwrap())
    }

    /// big endian
    pub fn to_u32_digits(self) -> [Target; 5] {
        self.to_vec().try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    #[test]
    fn test_display_address() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let address = super::Address::rand(&mut rng);
        let address_str = address.to_string();
        let recovered_address = super::Address::from_hex(&address_str);
        assert_eq!(address, recovered_address);
    }
}

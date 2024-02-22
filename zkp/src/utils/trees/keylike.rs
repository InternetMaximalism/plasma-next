use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOut, HashOutTarget, RichField},
    iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_u32::gadgets::arithmetic_u32::U32Target;

use crate::utils::u256::{U256Target, U256};

pub trait KeyLike: Copy + Eq + Default + std::hash::Hash {
    // little endian
    fn to_bits(&self) -> Vec<bool>;

    fn to_bits_with_length(&self, length: usize) -> Vec<bool> {
        let bits = self.to_bits();
        assert!(bits.len() >= length);
        bits[..length].to_vec()
    }
}

impl KeyLike for u32 {
    fn to_bits(&self) -> Vec<bool> {
        self.to_le_bytes()
            .iter()
            .flat_map(|v| u8_to_le_bits(*v))
            .collect()
    }
}

impl KeyLike for U256 {
    fn to_bits(&self) -> Vec<bool> {
        self.to_le_bytes()
            .iter()
            .flat_map(|v| u8_to_le_bits(*v))
            .collect::<Vec<_>>()
    }
}

impl<F: RichField> KeyLike for HashOut<F> {
    fn to_bits(&self) -> Vec<bool> {
        let limbs = self.elements; // little endian

        limbs
            .iter()
            .flat_map(|v| {
                v.to_canonical_u64()
                    .to_le_bytes()
                    .iter()
                    .flat_map(|v| u8_to_le_bits(*v))
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

pub trait KeyLikeTarget {
    /// little endian
    fn to_bits<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Vec<BoolTarget>;

    /// big endian
    fn to_bits_with_length<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        length: usize,
    ) -> Vec<BoolTarget> {
        let bits = self.to_bits(builder);
        assert!(bits.len() >= length);
        bits[..length].to_vec()
    }
}

impl KeyLikeTarget for U32Target {
    fn to_bits<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Vec<BoolTarget> {
        builder.split_le(self.0, 32)
    }
}

impl KeyLikeTarget for U256Target {
    fn to_bits<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Vec<BoolTarget> {
        self.split_le(builder)
    }
}

impl KeyLikeTarget for HashOutTarget {
    fn to_bits<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Vec<BoolTarget> {
        self.elements
            .iter()
            .flat_map(|e| builder.split_le(*e, 64))
            .collect::<Vec<_>>()
    }
}

fn u8_to_le_bits(num: u8) -> Vec<bool> {
    let mut result = Vec::with_capacity(8);
    let mut n = num;
    for _ in 0..8 {
        result.push(n & 1 == 1);
        n >>= 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use plonky2::{
        field::types::Sample,
        hash::poseidon::PoseidonHash,
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::{
            circuit_builder::CircuitBuilder,
            circuit_data::CircuitConfig,
            config::{GenericConfig, Hasher, PoseidonGoldilocksConfig},
        },
    };
    use plonky2_u32::{gadgets::arithmetic_u32::U32Target, witness::WitnessU32};

    use crate::utils::u256::{U256Target, U256};

    use super::{KeyLike, KeyLikeTarget};

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_keylike_u32() {
        let index = 0x12345678u32;
        let length = 10;
        let bits = index.to_bits_with_length(length);

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::default());
        let index_t = U32Target(builder.add_virtual_target());
        let bits_t = index_t.to_bits_with_length(&mut builder, length);

        let data = builder.build::<C>();
        let mut pw = PartialWitness::<F>::new();

        pw.set_u32_target(index_t, index);
        for (&bit_t, &bit) in bits_t.iter().zip(bits.iter()) {
            pw.set_bool_target(bit_t, bit);
        }
        data.prove(pw).unwrap();
    }

    #[test]
    fn test_keylike_u256() {
        let rng = &mut rand::thread_rng();
        let index = U256::rand(rng);
        let length = 10;
        let bits = index.to_bits_with_length(length);

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::default());
        let index_t = U256Target::new_unsafe(&mut builder);
        let bits_t = index_t.to_bits_with_length(&mut builder, length);

        let data = builder.build::<C>();
        let mut pw = PartialWitness::<F>::new();

        index_t.set_witness(&mut pw, index);
        for (&bit_t, &bit) in bits_t.iter().zip(bits.iter()) {
            pw.set_bool_target(bit_t, bit);
        }
        data.prove(pw).unwrap();
    }

    #[test]
    fn test_keylike_hash_out() {
        type H = PoseidonHash;
        let index = H::hash_no_pad(&[F::rand()]);
        let length = 10;
        let bits = index.to_bits_with_length(length);

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::default());
        let index_t = builder.add_virtual_hash();
        let bits_t = index_t.to_bits_with_length(&mut builder, length);

        let data = builder.build::<C>();
        let mut pw = PartialWitness::<F>::new();

        pw.set_hash_target(index_t, index);
        for (&bit_t, &bit) in bits_t.iter().zip(bits.iter()) {
            pw.set_bool_target(bit_t, bit);
        }
        data.prove(pw).unwrap();
    }
}

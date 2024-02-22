use std::fmt::Display;

use plonky2::{
    field::{
        extension::Extendable,
        types::{Field, PrimeField64},
    },
    hash::hash_types::RichField,
    iop::{
        target::{BoolTarget, Target},
        witness::WitnessWrite,
    },
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_u32::gadgets::arithmetic_u32::U32Target;
use serde::{Deserialize, Serialize};

// To prevent double spent of Withdraw transaction, we only allow withdrawals
// with ExtendedBlockNumber in increasing order.
// ExtendedBlockNumber is created by combining block_number and transfer_index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ExtendedBlockNumber(u64);

impl ExtendedBlockNumber {
    pub fn new(input: u64) -> Self {
        Self(input)
    }

    pub fn sub_one(&self) -> Self {
        Self(self.0 - 1)
    }

    pub fn construct(block_number: u32, transfer_index: usize) -> Self {
        Self(((block_number as u64) << 32) + (transfer_index as u64))
    }

    pub fn less_than(&self, other: &Self) {
        assert!(self.0 < other.0);
    }

    pub fn to_vec<F: PrimeField64>(&self) -> Vec<F> {
        vec![F::from_canonical_u64(self.0)]
    }

    pub fn from_vec<F: PrimeField64>(vec: &[F]) -> Self {
        assert_eq!(vec.len(), 1);
        Self(vec[0].to_canonical_u64())
    }

    pub fn to_u32_digits(&self) -> [u32; 2] {
        [(self.0 >> 32) as u32, self.0 as u32] // big endian
    }
}

impl Display for ExtendedBlockNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct ExtendedBlockNumberTarget(Target);

impl ExtendedBlockNumberTarget {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        Self(builder.add_virtual_target())
    }

    pub fn construct<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        block_number: U32Target,
        transfer_index: Target,
    ) -> Self {
        Self(builder.mul_const_add(
            F::from_canonical_u64(1 << 32),
            block_number.0,
            transfer_index,
        ))
    }

    pub fn connect<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: &Self,
    ) {
        builder.connect(self.0, other.0);
    }

    /// select a if flag is true, otherwise b
    pub fn select<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        b: BoolTarget,
        x: Self,
        y: Self,
    ) -> Self {
        Self(builder.select(b, x.0, y.0))
    }

    pub fn less_than<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: &Self,
    ) {
        let diff = builder.sub(other.0, self.0);

        // check that 0 < diff < (1<<62)
        builder.range_check(diff, 62);
        let zero = builder.zero();
        let is_zero = builder.is_equal(diff, zero);
        builder.assert_zero(is_zero.target);
    }

    pub fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        value: &ExtendedBlockNumber,
    ) -> Self {
        Self(builder.constant(F::from_canonical_u64(value.0)))
    }

    pub fn to_vec(&self) -> Vec<Target> {
        vec![self.0]
    }

    pub fn from_vec(vec: &[Target]) -> Self {
        assert_eq!(vec.len(), 1);
        Self(vec[0])
    }

    pub fn to_u32_digits<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> [Target; 2] {
        let (low, high) = builder.split_low_high(self.0, 32, 64);
        [high, low]
    }

    pub fn set_witness<F: Field>(
        &self,
        pw: &mut impl WitnessWrite<F>,
        value: &ExtendedBlockNumber,
    ) {
        pw.set_target(self.0, F::from_canonical_u64(value.0));
    }
}

#[cfg(test)]
mod tests {
    use plonky2::{
        iop::witness::PartialWitness,
        plonk::{
            circuit_builder::CircuitBuilder,
            circuit_data::CircuitConfig,
            config::{GenericConfig, PoseidonGoldilocksConfig},
        },
    };

    use super::{ExtendedBlockNumber, ExtendedBlockNumberTarget};

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_extended_block_number_case0() {
        let x = ExtendedBlockNumber::construct(3, 5);
        let y = ExtendedBlockNumber::construct(5, 4);
        x.less_than(&y);
        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::default());
        let x = ExtendedBlockNumberTarget::constant(&mut builder, &x);
        let y = ExtendedBlockNumberTarget::constant(&mut builder, &y);
        x.less_than(&mut builder, &y);
        let data = builder.build::<C>();
        let _proof = data.prove(PartialWitness::new()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_extended_block_number_case1() {
        let x = ExtendedBlockNumber::construct(3, 5);
        let y = ExtendedBlockNumber::construct(3, 5);
        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::default());
        let x = ExtendedBlockNumberTarget::constant(&mut builder, &x);
        let y = ExtendedBlockNumberTarget::constant(&mut builder, &y);
        x.less_than(&mut builder, &y);
        let data = builder.build::<C>();
        let _proof = data.prove(PartialWitness::new()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_extended_block_number_case2() {
        let x = ExtendedBlockNumber::construct(3, 5);
        let y = ExtendedBlockNumber::construct(2, 5);
        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::default());
        let x = ExtendedBlockNumberTarget::constant(&mut builder, &x);
        let y = ExtendedBlockNumberTarget::constant(&mut builder, &y);
        x.less_than(&mut builder, &y);
        let data = builder.build::<C>();
        let _proof = data.prove(PartialWitness::new()).unwrap();
    }
}

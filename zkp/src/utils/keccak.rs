use core::fmt::Debug;
use plonky2::{field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget};
use starky_keccak::builder::CircuitBuilderWithKeccak;

pub trait LeafableTargetKeccak: Clone {
    type HashOutTarget: Clone + Debug;

    /// Default constant hash target which indicates empty value.
    fn empty_leaf<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilderWithKeccak<F, D>,
    ) -> Self;

    /// Hash target of its value.
    fn hash<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilderWithKeccak<F, D>,
    ) -> Self::HashOutTarget;

    fn connect_hash<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilderWithKeccak<F, D>,
        x: &Self::HashOutTarget,
        y: &Self::HashOutTarget,
    );

    fn two_to_one<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilderWithKeccak<F, D>,
        left: &Self::HashOutTarget,
        right: &Self::HashOutTarget,
    ) -> Self::HashOutTarget;

    fn two_to_one_swapped<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilderWithKeccak<F, D>,
        left: &Self::HashOutTarget,
        right: &Self::HashOutTarget,
        swap: BoolTarget,
    ) -> Self::HashOutTarget;
}

pub fn get_merkle_root_from_leaves_keccak_circuit<
    F: RichField + Extendable<D>,
    VT: LeafableTargetKeccak,
    const D: usize,
>(
    builder: &mut CircuitBuilderWithKeccak<F, D>,
    height: usize,
    leaves: &[VT],
) -> VT::HashOutTarget {
    assert_eq!(leaves.len(), 1 << height);
    let mut layer = leaves.iter().map(|v| v.hash(builder)).collect::<Vec<_>>();
    assert_ne!(layer.len(), 0);
    while layer.len() > 1 {
        if layer.len() % 2 == 1 {
            panic!("leaves is not power of 2");
        }
        layer = (0..(layer.len() / 2))
            .map(|i| VT::two_to_one(builder, &layer[2 * i], &layer[2 * i + 1]))
            .collect::<Vec<_>>();
    }
    layer[0].clone()
}

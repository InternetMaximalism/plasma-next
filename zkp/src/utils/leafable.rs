use std::fmt::Debug;

use plonky2::{
    field::extension::Extendable,
    hash::{
        hash_types::{HashOut, HashOutTarget, RichField, NUM_HASH_OUT_ELTS},
        hashing::PlonkyPermutation,
        poseidon::PoseidonHash,
    },
    iop::target::{BoolTarget, Target},
    plonk::{
        circuit_builder::CircuitBuilder,
        config::{AlgebraicHasher, Hasher},
    },
};
use serde::{Deserialize, Serialize};

use super::h256::{H256Target, H256};

/// Can be a leaf of Merkle trees.
pub trait Leafable<F>: Clone {
    type HashOut: PartialEq + Clone + Debug + Serialize + Deserialize<'static>;

    /// Default hash which indicates empty value.
    fn empty_leaf() -> Self;

    /// Hash of its value.
    fn hash(&self) -> Self::HashOut;

    fn two_to_one(left: &Self::HashOut, right: &Self::HashOut) -> Self::HashOut;
}

impl<F: RichField> Leafable<F> for HashOut<F> {
    type HashOut = HashOut<F>;

    fn empty_leaf() -> Self {
        Self::default()
    }

    // Output as is in the case of a hash.
    fn hash(&self) -> Self {
        *self
    }

    fn two_to_one(left: &HashOut<F>, right: &HashOut<F>) -> Self {
        PoseidonHash::two_to_one(*left, *right)
    }
}

impl<F: RichField> Leafable<F> for Vec<F> {
    type HashOut = HashOut<F>;

    fn empty_leaf() -> Self {
        vec![]
    }

    fn hash(&self) -> Self::HashOut {
        PoseidonHash::hash_no_pad(&self)
    }

    fn two_to_one(left: &Self::HashOut, right: &Self::HashOut) -> Self::HashOut {
        PoseidonHash::two_to_one(*left, *right)
    }
}

impl<F: RichField> Leafable<F> for H256 {
    type HashOut = HashOut<F>;

    fn empty_leaf() -> Self {
        H256::default()
    }

    fn hash(&self) -> HashOut<F> {
        PoseidonHash::hash_no_pad(&self.to_vec())
    }

    fn two_to_one(left: &Self::HashOut, right: &Self::HashOut) -> Self::HashOut {
        PoseidonHash::two_to_one(*left, *right)
    }
}

/// Can be a leaf target of Merkle trees.
pub trait LeafableTarget: Clone {
    type HashOutTarget: Clone + Debug;

    /// Default constant hash target which indicates empty value.
    fn empty_leaf<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self;

    /// Hash target of its value.
    fn hash<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self::HashOutTarget;

    fn connect_hash<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        x: &Self::HashOutTarget,
        y: &Self::HashOutTarget,
    );

    fn two_to_one<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        left: &Self::HashOutTarget,
        right: &Self::HashOutTarget,
    ) -> Self::HashOutTarget;

    fn two_to_one_swapped<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        left: &Self::HashOutTarget,
        right: &Self::HashOutTarget,
        swap: BoolTarget,
    ) -> Self::HashOutTarget;
}

impl LeafableTarget for HashOutTarget {
    type HashOutTarget = HashOutTarget;

    fn empty_leaf<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        let empty_leaf = Leafable::<F>::empty_leaf();
        builder.constant_hash(empty_leaf)
    }

    fn hash<F: RichField + Extendable<D>, const D: usize>(
        &self,
        _builder: &mut CircuitBuilder<F, D>,
    ) -> HashOutTarget {
        *self
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
    ) -> HashOutTarget {
        poseidon_two_to_one(builder, left, right)
    }

    fn two_to_one_swapped<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        left: &Self::HashOutTarget,
        right: &Self::HashOutTarget,
        swap: BoolTarget,
    ) -> Self::HashOutTarget {
        poseidon_two_to_one_swapped(builder, left, right, swap)
    }
}

impl LeafableTarget for Vec<Target> {
    type HashOutTarget = HashOutTarget;

    fn empty_leaf<F: RichField + Extendable<D>, const D: usize>(
        _builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        vec![]
    }

    fn hash<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> HashOutTarget {
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(self.clone())
    }

    fn connect_hash<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        x: &Self::HashOutTarget,
        y: &Self::HashOutTarget,
    ) {
        builder.connect_hashes(*x, *y);
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
        swap: BoolTarget,
    ) -> Self::HashOutTarget {
        poseidon_two_to_one_swapped(builder, left, right, swap)
    }
}

impl LeafableTarget for H256Target {
    type HashOutTarget = HashOutTarget;

    fn empty_leaf<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        H256Target::constant(builder, H256::default())
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
        swap: BoolTarget,
    ) -> Self::HashOutTarget {
        poseidon_two_to_one_swapped(builder, left, right, swap)
    }
}

pub(crate) fn poseidon_two_to_one<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left: &HashOutTarget,
    right: &HashOutTarget,
) -> HashOutTarget {
    builder.hash_n_to_hash_no_pad::<PoseidonHash>(vec![
        left.elements[0],
        left.elements[1],
        left.elements[2],
        left.elements[3],
        right.elements[0],
        right.elements[1],
        right.elements[2],
        right.elements[3],
    ])
}

pub(crate) fn poseidon_two_to_one_swapped<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left: &HashOutTarget,
    right: &HashOutTarget,
    swap: BoolTarget,
) -> HashOutTarget {
    let zero = builder.zero();
    let mut perm_inputs = <PoseidonHash as AlgebraicHasher<F>>::AlgebraicPermutation::default();
    perm_inputs.set_from_slice(&left.elements, 0);
    perm_inputs.set_from_slice(&right.elements, NUM_HASH_OUT_ELTS);
    // Ensure the rest of the state, if any, is zero:
    perm_inputs.set_from_iter(std::iter::repeat(zero), 2 * NUM_HASH_OUT_ELTS);
    let perm_outs = PoseidonHash::permute_swapped(perm_inputs, swap, builder);
    let hash_outs = perm_outs.squeeze()[0..NUM_HASH_OUT_ELTS]
        .try_into()
        .unwrap();
    HashOutTarget {
        elements: hash_outs,
    }
}

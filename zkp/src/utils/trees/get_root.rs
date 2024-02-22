use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField,
    plonk::circuit_builder::CircuitBuilder,
};

use crate::utils::leafable::{Leafable, LeafableTarget};

pub fn get_merkle_root_from_leaves<F: RichField, V: Leafable<F>>(
    height: usize,
    leaves: &[V],
) -> V::HashOut {
    assert_eq!(leaves.len(), 1 << height);
    let mut layer = leaves.iter().map(|v| v.hash()).collect::<Vec<_>>();
    assert_ne!(layer.len(), 0);
    while layer.len() > 1 {
        if layer.len() % 2 == 1 {
            panic!("leaves is not power of 2");
        }
        layer = (0..(layer.len() / 2))
            .map(|i| V::two_to_one(&layer[2 * i], &layer[2 * i + 1]))
            .collect::<Vec<_>>();
    }
    layer[0].clone()
}

pub fn get_merkle_root_from_leaves_circuit<
    F: RichField + Extendable<D>,
    VT: LeafableTarget,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
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

#[cfg(test)]
mod tests {
    use plonky2::{
        field::types::Sample,
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::{
            circuit_builder::CircuitBuilder,
            circuit_data::CircuitConfig,
            config::{GenericConfig, PoseidonGoldilocksConfig},
        },
    };

    use crate::utils::{
        leafable::{Leafable, LeafableTarget},
        trees::{
            get_root::{get_merkle_root_from_leaves, get_merkle_root_from_leaves_circuit},
            merkle_tree_with_leaves::MerkleTreeWithLeaves,
        },
    };

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_get_merkle_root_target_from_leaves() {
        let height = 15;
        let num_leaves = 800;
        type V = Vec<F>;
        let mut tree = MerkleTreeWithLeaves::<F, V>::new(height);
        let leaves = (0..num_leaves).map(|_| vec![F::rand()]).collect::<Vec<_>>();
        for leaf in &leaves {
            tree.push(leaf.clone());
        }
        let root_expected = tree.get_root();

        let mut padded_leaves = leaves.clone();
        padded_leaves.resize(1 << height, V::empty_leaf());
        let root = get_merkle_root_from_leaves(height, &padded_leaves);
        assert_eq!(root, root_expected);
    }

    #[test]
    fn test_get_merkle_root_target_from_leaves_circuit() {
        let height = 10;
        let leaves = (0..1 << height)
            .map(|_| vec![F::rand()])
            .collect::<Vec<_>>();
        let root = get_merkle_root_from_leaves(height, &leaves);

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::default());
        let mut leaves_t = leaves
            .into_iter()
            .map(|leaf| {
                leaf.into_iter()
                    .map(|e| builder.constant(e))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        leaves_t.resize(1 << height, LeafableTarget::empty_leaf(&mut builder));

        let root_t = get_merkle_root_from_leaves_circuit(&mut builder, height, &leaves_t);
        let data = builder.build::<C>();

        let mut pw = PartialWitness::<F>::new();
        pw.set_hash_target(root_t, root);
        data.prove(pw).unwrap();
    }
}

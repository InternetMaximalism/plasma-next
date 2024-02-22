use std::collections::HashMap;

use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOut, HashOutTarget, RichField},
    iop::{target::BoolTarget, witness::Witness},
    plonk::circuit_builder::CircuitBuilder,
};

use crate::utils::leafable::{Leafable, LeafableTarget};

// MekleTree is a structure of Merkle Tree used for `MerkleTreeWithLeaves`
// and `SparseMerkleTreeWithLeaves`. It only holds non-zero nodes.
// All nodes are specified by path: Vec<bool>. The path is big endian.
// Note that this is different from the original plonky2 Merkle Tree which
// uses little endian path.
#[derive(Clone, Debug)]
pub(crate) struct MerkleTree<F: RichField, V: Leafable<F>> {
    height: usize,
    node_hashes: HashMap<Vec<bool>, V::HashOut>,
    zero_hashes: Vec<V::HashOut>,
}

impl<F: RichField, V: Leafable<F>> MerkleTree<F, V> {
    pub(crate) fn new(height: usize, empty_leaf_hash: V::HashOut) -> Self {
        // zero_hashes = reverse([H(zero_leaf), H(H(zero_leaf), H(zero_leaf)), ...])
        let mut zero_hashes = vec![];
        let mut h = empty_leaf_hash;
        zero_hashes.push(h.clone());
        for _ in 0..height {
            h = V::two_to_one(&h, &h);
            zero_hashes.push(h.clone());
        }
        zero_hashes.reverse();

        let node_hashes: HashMap<Vec<bool>, V::HashOut> = HashMap::new();

        Self {
            height,
            node_hashes,
            zero_hashes,
        }
    }

    pub(crate) fn height(&self) -> usize {
        self.height
    }

    pub(crate) fn get_node_hash(&self, path: &Vec<bool>) -> V::HashOut {
        assert!(path.len() <= self.height);
        match self.node_hashes.get(path) {
            Some(h) => h.clone(),
            None => self.zero_hashes[path.len()].clone(),
        }
    }

    pub(crate) fn get_root(&self) -> V::HashOut {
        self.get_node_hash(&vec![])
    }

    fn get_sibling_hash(&self, path: &Vec<bool>) -> V::HashOut {
        assert!(!path.is_empty());
        let mut path = path.clone();
        let last = path.len() - 1;
        path[last] = !path[last];
        self.get_node_hash(&path)
    }

    // index_bits is little endian
    pub(crate) fn update_leaf(&mut self, index_bits: Vec<bool>, leaf_hash: V::HashOut) {
        assert_eq!(index_bits.len(), self.height);
        let mut path = index_bits;
        path.reverse(); // path is big endian

        let mut h = leaf_hash;
        self.node_hashes.insert(path.clone(), h.clone());

        while !path.is_empty() {
            let sibling = self.get_sibling_hash(&path);
            h = if path.pop().unwrap() {
                V::two_to_one(&sibling, &h)
            } else {
                V::two_to_one(&h, &sibling)
            };
            self.node_hashes.insert(path.clone(), h.clone());
        }
    }

    pub(crate) fn prove(&self, index_bits: Vec<bool>) -> MerkleProof<F, V> {
        assert_eq!(index_bits.len(), self.height);
        let mut path = index_bits;
        path.reverse(); // path is big endian

        let mut siblings = vec![];
        while !path.is_empty() {
            siblings.push(self.get_sibling_hash(&path));
            path.pop();
        }
        MerkleProof { siblings }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct MerkleProof<F: RichField, V: Leafable<F>> {
    pub(crate) siblings: Vec<V::HashOut>,
}

impl<F: RichField, V: Leafable<F>> MerkleProof<F, V> {
    pub fn height(&self) -> usize {
        self.siblings.len()
    }

    pub fn verify(
        &self,
        leaf_data: &V,
        index_bits: Vec<bool>, // little endian
        merkle_root: V::HashOut,
    ) -> anyhow::Result<()> {
        let mut state = leaf_data.hash();
        for (&bit, sibling) in index_bits.iter().zip(self.siblings.iter()) {
            state = if bit {
                V::two_to_one(&sibling, &state)
            } else {
                V::two_to_one(&state, &sibling)
            }
        }
        anyhow::ensure!(state == merkle_root, "Merkle proof verification failed");
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct MerkleProofTarget<VT: LeafableTarget> {
    siblings: Vec<VT::HashOutTarget>,
}

impl<VT: LeafableTarget<HashOutTarget = HashOutTarget>> MerkleProofTarget<VT> {
    pub(crate) fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        height: usize,
    ) -> Self {
        Self {
            siblings: builder.add_virtual_hashes(height),
        }
    }

    pub(crate) fn constant<
        F: RichField + Extendable<D>,
        const D: usize,
        V: Leafable<F, HashOut = HashOut<F>>,
    >(
        builder: &mut CircuitBuilder<F, D>,
        input: &MerkleProof<F, V>,
    ) -> Self {
        Self {
            siblings: input
                .siblings
                .iter()
                .map(|sibling| builder.constant_hash(*sibling))
                .collect(),
        }
    }

    pub(crate) fn set_witness<F: RichField, V: Leafable<F, HashOut = HashOut<F>>>(
        &self,
        pw: &mut impl Witness<F>,
        merkle_proof: &MerkleProof<F, V>,
    ) {
        assert_eq!(self.siblings.len(), merkle_proof.siblings.len());
        for (sibling_t, sibling) in self.siblings.iter().zip(merkle_proof.siblings.iter()) {
            pw.set_hash_target(*sibling_t, *sibling);
        }
    }
}

impl<VT: LeafableTarget> MerkleProofTarget<VT> {
    pub(crate) fn get_root<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        leaf_data: &VT,
        index_bits: Vec<BoolTarget>,
    ) -> VT::HashOutTarget {
        let mut state = leaf_data.hash(builder);
        assert_eq!(index_bits.len(), self.siblings.len());
        for (bit, sibling) in index_bits.iter().zip(&self.siblings) {
            state = VT::two_to_one_swapped(builder, &state, sibling, *bit);
        }
        state
    }

    pub(crate) fn verify<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        leaf_data: &VT,
        index_bits: Vec<BoolTarget>,
        merkle_root: VT::HashOutTarget,
    ) {
        let state = self.get_root(builder, leaf_data, index_bits);
        VT::connect_hash(builder, &state, &merkle_root);
    }

    pub(crate) fn height(&self) -> usize {
        self.siblings.len()
    }
}

pub fn usize_le_bits(num: usize, length: usize) -> Vec<bool> {
    let mut result = Vec::with_capacity(length);
    let mut n = num;
    for _ in 0..length {
        result.push(n & 1 == 1);
        n >>= 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use plonky2::{
        field::types::{Field, Sample},
        hash::poseidon::PoseidonHash,
        iop::{
            target::Target,
            witness::{PartialWitness, WitnessWrite},
        },
        plonk::{
            circuit_data::CircuitConfig,
            config::{GenericConfig, Hasher, PoseidonGoldilocksConfig},
        },
    };

    use rand::Rng;

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_merkle_tree_update_prove_verify() {
        let mut rng = rand::thread_rng();
        let height = 10;

        let mut tree = MerkleTree::<F, Vec<F>>::new(height, PoseidonHash::hash_no_pad(&vec![]));

        for _ in 0..100 {
            let index = rng.gen_range(0..1 << height);
            let new_leaf = vec![F::rand()];
            let leaf_hash = PoseidonHash::hash_no_pad(&new_leaf);
            let index_bits = usize_le_bits(index, height);
            tree.update_leaf(index_bits.clone(), leaf_hash);
            let proof = tree.prove(index_bits.clone());
            proof
                .verify(&new_leaf, index_bits, tree.get_root())
                .unwrap();
        }
    }

    #[test]
    fn test_merkle_proof_target() {
        let mut rng = rand::thread_rng();
        let height = 10;

        type H = PoseidonHash;
        let mut tree = MerkleTree::<F, Vec<F>>::new(height, PoseidonHash::hash_no_pad(&vec![]));

        let index = rng.gen_range(0..1 << height);
        let leaf = vec![F::rand()];
        let leaf_hash = H::hash_no_pad(&leaf);
        let index_bits = usize_le_bits(index, height);
        tree.update_leaf(index_bits.clone(), leaf_hash);
        let proof = tree.prove(index_bits.clone());

        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::default());
        let proof_t = MerkleProofTarget::<Vec<Target>>::new(&mut builder, height);
        let leaf_t = vec![builder.add_virtual_target()];
        let root_t = builder.add_virtual_hash();
        let index_t = builder.add_virtual_target();
        let index_bits_t = builder.split_le(index_t, height);
        proof_t.verify(&mut builder, &leaf_t, index_bits_t, root_t);

        let data = builder.build::<C>();
        let mut pw = PartialWitness::<F>::new();
        pw.set_target_arr(&leaf_t, &leaf);
        pw.set_hash_target(root_t, tree.get_root());
        pw.set_target(index_t, F::from_canonical_usize(index));
        proof_t.set_witness(&mut pw, &proof);

        data.prove(pw).unwrap();
    }
}

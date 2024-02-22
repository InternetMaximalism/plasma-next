use std::fmt::Display;

use anyhow::{ensure, Ok};
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    plonk::{
        circuit_data::CommonCircuitData,
        config::{AlgebraicHasher, GenericConfig},
        proof::ProofWithPublicInputs,
    },
};
use serde::{Deserialize, Serialize};
use starky_keccak::keccak256_circuit::solidity_keccak256;

use crate::{
    tree_circuits::{
        dynamic_leafable::{DynamicLeafable, DynamicLeafableCircuit},
        dynamic_tree_circuit::{DynamicTreeCircuit, DynamicTreePublicInputs},
    },
    utils::{display::join_str_with_separator, h256::H256},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = ""))]
pub struct ProofWithHash<F, C, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    pub proof: ProofWithPublicInputs<F, C, D>,
    pub hash: H256,
}

pub struct TreeProcessor<F, C, const D: usize, Leaf, LeafCircuit>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    Leaf: DynamicLeafable,
    LeafCircuit: DynamicLeafableCircuit<F, C, D>,
{
    pub nodes: Vec<Vec<ProofWithHash<F, C, D>>>,
    pub leaves: Vec<Leaf>,
    pub node_circuit: DynamicTreeCircuit<F, C, D, LeafCircuit>,
}

impl<F, C, const D: usize, Leaf, LeafCircuit> TreeProcessor<F, C, D, Leaf, LeafCircuit>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
    Leaf: DynamicLeafable,
    LeafCircuit: DynamicLeafableCircuit<F, C, D>,
{
    pub fn new(leaf_circuit: &LeafCircuit, common_data: &mut CommonCircuitData<F, D>) -> Self {
        let node_circuit = DynamicTreeCircuit::new(leaf_circuit, common_data);
        Self {
            nodes: vec![],
            leaves: vec![],
            node_circuit,
        }
    }

    pub fn initialize(&mut self) {
        self.nodes = vec![];
        self.leaves = vec![];
    }

    pub fn add(
        &mut self,
        leaf: Leaf,
        leaf_proof: ProofWithPublicInputs<F, C, D>,
    ) -> anyhow::Result<()> {
        let proof = self.node_circuit.prove(Some(leaf_proof), None)?;
        let pis = DynamicTreePublicInputs::from_pis(&proof.public_inputs);
        let proof_with_hash = ProofWithHash {
            proof: proof.clone(),
            hash: pis.hash,
        };
        fill_node(&mut self.nodes, &self.node_circuit, 0, proof_with_hash)?;
        self.leaves.push(leaf);
        Ok(())
    }

    pub fn get(&self) -> Vec<Leaf> {
        self.leaves.clone()
    }

    pub fn finalize(
        &mut self,
    ) -> Option<(
        ProofWithHash<F, C, D>,
        Vec<DynamicMerkleProofWithLeaf<Leaf>>,
    )> {
        if self.leaves.len() == 0 {
            return None;
        }
        let dummy_proof = self.nodes[0][0].clone();
        let mut level = 0;
        loop {
            if self.nodes.len() - 1 == level {
                break;
            }
            if self.nodes[level].len() % 2 == 1 {
                fill_node(
                    &mut self.nodes,
                    &self.node_circuit,
                    level,
                    dummy_proof.clone(),
                )
                .unwrap();
            }
            level += 1;
        }
        let root_proof = self.nodes[self.nodes.len() - 1][0].clone();
        let mut merkle_proofs = vec![];
        for (i, leaf) in self.leaves.iter().enumerate() {
            let merkle_proof = generate_merkle_proof(&self.nodes, i, leaf.clone());
            merkle_proof.verify(root_proof.hash).unwrap();
            merkle_proofs.push(merkle_proof);
        }
        Some((root_proof, merkle_proofs))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicMerkleProofWithLeaf<Leaf: DynamicLeafable> {
    index: usize,
    siblings: Vec<H256>,
    leaf: Leaf,
}

impl<Leaf: DynamicLeafable> Display for DynamicMerkleProofWithLeaf<Leaf> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{index: {}, siblings: [{}], leaf: {}}}",
            self.index,
            join_str_with_separator(&self.siblings, ", "),
            self.leaf
        )
    }
}

impl<Leaf: DynamicLeafable> DynamicMerkleProofWithLeaf<Leaf> {
    pub fn verify(&self, root: H256) -> anyhow::Result<()> {
        let mut hash = self.leaf.hash();
        let mut index = self.index;
        for sibling in self.siblings.iter() {
            hash = if index % 2 == 0 {
                H256::from_u32_digits(
                    solidity_keccak256(
                        vec![hash.to_u32_digits(), sibling.to_u32_digits()].concat(),
                    )
                    .0,
                )
            } else {
                H256::from_u32_digits(
                    solidity_keccak256(
                        vec![sibling.to_u32_digits(), hash.to_u32_digits()].concat(),
                    )
                    .0,
                )
            };
            index >>= 1;
        }
        ensure!(hash == root, "merkle proof verification failed");
        Ok(())
    }
}

fn generate_merkle_proof<F, C, const D: usize, Leaf>(
    nodes: &Vec<Vec<ProofWithHash<F, C, D>>>,
    index: usize,
    leaf: Leaf,
) -> DynamicMerkleProofWithLeaf<Leaf>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
    Leaf: DynamicLeafable,
{
    let height = nodes.len() - 1;
    assert!(index <= 1 << height, "index out of range");
    let mut siblings = vec![];
    for level in 0..height {
        let cur_level_index = index >> level;
        let sibling_index = if cur_level_index % 2 == 0 {
            cur_level_index + 1
        } else {
            cur_level_index - 1
        };
        let sibling = nodes[level][sibling_index].hash;
        siblings.push(sibling);
    }
    DynamicMerkleProofWithLeaf {
        index,
        siblings,
        leaf,
    }
}

fn fill_node<F, C, const D: usize>(
    nodes: &mut Vec<Vec<ProofWithHash<F, C, D>>>,
    node_circuit: &DynamicTreeCircuit<F, C, D, impl DynamicLeafableCircuit<F, C, D>>,
    start_level: usize,
    input: ProofWithHash<F, C, D>,
) -> anyhow::Result<()>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let mut level = start_level;
    let mut proof_with_hash = input;
    loop {
        if nodes.len() <= level {
            nodes.push(vec![]);
        }
        nodes[level].push(proof_with_hash.clone());
        if nodes[level].len() % 2 == 1 {
            break;
        }
        let left = nodes[level][nodes[level].len() - 2].proof.clone();
        let right = nodes[level][nodes[level].len() - 1].proof.clone();
        let next_proof = node_circuit.prove(None, Some((left, right)))?;
        let next_pis = DynamicTreePublicInputs::from_pis(&next_proof.public_inputs);
        proof_with_hash = ProofWithHash {
            proof: next_proof,
            hash: next_pis.hash,
        };
        level += 1;
    }
    Ok(())
}

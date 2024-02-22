use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOut, HashOutTarget, RichField},
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, Witness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{AlgebraicHasher, GenericConfig},
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use starky_keccak::{builder::CircuitBuilderWithKeccak, keccak256_circuit::solidity_keccak256};

use std::fmt::Display;

use crate::{
    base_circuits::withdraw_circuit::{WithdrawCircuit, WithdrawPublicInputsTarget},
    common::{
        block::{Block, BlockTarget},
        transfer_info::{TransferInfo, TransferInfoTarget},
    },
    tree_circuits::{evidence_leaf::EVIDENCE_LEAF_LEN, withdraw_leaf::WITHDRAW_LEAF_LEN},
    utils::{
        dummy::DummyProof,
        h256::{H256Target, H256},
        trees::merkle_tree_with_leaves::{MerkleProofWithLeaves, MerkleProofWithLeavesTarget},
    },
};

use super::{
    dynamic_leafable::{DynamicLeafable, DynamicLeafableCircuit},
    dynamic_tree_circuit::DynamicTreePublicInputsTarget,
    evidence_leaf::{EvidenceLeaf, EvidenceLeafTarget},
    withdraw_leaf::{WithdrawLeaf, WithdrawLeafTarget},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementLeaf {
    pub withdraw_leaf: WithdrawLeaf,
    pub evidence_leaf: EvidenceLeaf,
}

impl Display for SettlementLeaf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(withdraw_leaf: {}, evidence_leaf: {})",
            self.withdraw_leaf, self.evidence_leaf
        )
    }
}

impl SettlementLeaf {
    pub fn new<F, C, const D: usize>(
        withdraw_circuit: &WithdrawCircuit<F, C, D>,
        block_root: &HashOut<F>,
        block_merkle_proof_for_withdraw: &MerkleProofWithLeaves<F, Block>,
        block_merkle_proof_for_evidence: &MerkleProofWithLeaves<F, Block>,
        withdraw_proof: &ProofWithPublicInputs<F, C, D>,
        evidence_transfer_info: &TransferInfo<F>,
    ) -> anyhow::Result<Self>
    where
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F> + 'static,
        <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
    {
        let withdraw_leaf = WithdrawLeaf::new(
            withdraw_circuit,
            block_root,
            block_merkle_proof_for_withdraw,
            withdraw_proof,
        )?;
        let evidence_leaf = EvidenceLeaf::new(
            block_root,
            block_merkle_proof_for_evidence,
            evidence_transfer_info,
        )?;
        Ok(Self {
            withdraw_leaf,
            evidence_leaf,
        })
    }

    pub fn rand<R: Rng>(rng: &mut R) -> Self {
        Self {
            withdraw_leaf: WithdrawLeaf::rand(rng),
            evidence_leaf: EvidenceLeaf::rand(rng),
        }
    }

    pub fn to_u32_digits(&self) -> Vec<u32> {
        let mut u32_digits = self.withdraw_leaf.to_u32_digits().to_vec();
        u32_digits.extend(self.evidence_leaf.to_u32_digits().to_vec());
        assert_eq!(u32_digits.len(), WITHDRAW_LEAF_LEN + EVIDENCE_LEAF_LEN);
        u32_digits
    }

    pub fn hash(&self) -> H256 {
        H256::from_u32_digits(solidity_keccak256(self.to_u32_digits()).0)
    }
}

#[derive(Debug, Clone)]
pub struct SettlementLeafTarget {
    pub withdraw_leaf: WithdrawLeafTarget,
    pub evidence_leaf: EvidenceLeafTarget,
}

impl SettlementLeafTarget {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        Self {
            withdraw_leaf: WithdrawLeafTarget::new(builder),
            evidence_leaf: EvidenceLeafTarget::new(builder),
        }
    }

    pub fn to_u32_digits<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Vec<Target> {
        let mut vec = Vec::new();
        vec.extend(self.withdraw_leaf.to_u32_digits(builder).to_vec());
        vec.extend(self.evidence_leaf.to_u32_digits(builder).to_vec());
        assert_eq!(vec.len(), WITHDRAW_LEAF_LEN + EVIDENCE_LEAF_LEN);
        vec
    }

    pub fn set_witness<F: RichField, W: Witness<F>>(&self, pw: &mut W, input: &SettlementLeaf) {
        self.withdraw_leaf.set_witness(pw, &input.withdraw_leaf);
        self.evidence_leaf.set_witness(pw, &input.evidence_leaf);
    }

    pub fn hash<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilderWithKeccak<F, D>,
    ) -> H256Target {
        let digits = self.to_u32_digits(builder);
        H256Target::from_vec(&builder.keccak256(digits))
    }
}

impl DynamicLeafable for SettlementLeaf {
    fn hash(&self) -> H256 {
        self.hash()
    }
}

pub struct SettlementLeafCircuit<F, C, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    pub data: CircuitData<F, C, D>,
    pub block_root: HashOutTarget,
    pub block_merkle_proof_for_withdraw: MerkleProofWithLeavesTarget<BlockTarget>,
    pub block_merkle_proof_for_evidence: MerkleProofWithLeavesTarget<BlockTarget>,
    pub withdraw_proof: ProofWithPublicInputsTarget<D>,
    pub transfer_info: TransferInfoTarget,
}

impl<F, C, const D: usize> SettlementLeafCircuit<F, C, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
{
    pub fn new(withdraw_circuit: &WithdrawCircuit<F, C, D>) -> Self {
        let mut builder = CircuitBuilderWithKeccak::<F, D>::new(CircuitConfig::default());
        let block_root = builder.add_virtual_hash();

        // verify withdraw
        let withdraw_proof = withdraw_circuit.add_proof_target_and_verify(&mut builder);
        let withdraw_pis = WithdrawPublicInputsTarget::from_pis(&withdraw_proof.public_inputs);
        let withdraw_leaf = WithdrawLeafTarget {
            recipient: withdraw_pis.recipient,
            amount: withdraw_pis.total_amount,
            start_ebn: withdraw_pis.start_ebn,
            end_ebn: withdraw_pis.end_ebn,
        };
        let block_for_withdraw = withdraw_pis.block.clone();
        let block_merkle_proof_for_withdraw = MerkleProofWithLeavesTarget::new(&mut builder, 32);
        block_merkle_proof_for_withdraw.verify(
            &mut builder,
            &block_for_withdraw,
            block_for_withdraw.block_number.0,
            block_root,
        );

        // verify evidence
        let transfer_info = TransferInfoTarget::new(&mut builder);
        transfer_info.verify(&mut builder);
        let block_merkle_proof_for_evidence = MerkleProofWithLeavesTarget::new(&mut builder, 32);
        block_merkle_proof_for_evidence.verify(
            &mut builder,
            &transfer_info.block,
            transfer_info.block.block_number.0,
            block_root,
        );
        let transfer_commitment = transfer_info.transfer.keccak_hash(&mut builder);
        let ebn = transfer_info.ebn(&mut builder);
        let evidence_leaf = EvidenceLeafTarget {
            transfer_commitment,
            ebn,
        };

        let settlement_leaf = SettlementLeafTarget {
            withdraw_leaf,
            evidence_leaf,
        };
        let hash = settlement_leaf.hash(&mut builder);
        let pis = DynamicTreePublicInputsTarget { hash, block_root };
        builder.register_public_inputs(&pis.to_vec());

        // add ConstantGate
        for i in 0..1 << 10 {
            builder.constant(F::from_canonical_usize(i));
        }

        let data = builder.build::<C>();
        Self {
            data,
            block_root,
            block_merkle_proof_for_withdraw,
            block_merkle_proof_for_evidence,
            withdraw_proof,
            transfer_info,
        }
    }

    pub fn prove(
        &self,
        block_root: &HashOut<F>,
        block_merkle_proof_for_withdraw: &MerkleProofWithLeaves<F, Block>,
        block_merkle_proof_for_evidence: &MerkleProofWithLeaves<F, Block>,
        withdraw_proof: &ProofWithPublicInputs<F, C, D>,
        transfer_info: &TransferInfo<F>,
    ) -> anyhow::Result<ProofWithPublicInputs<F, C, D>> {
        let mut pw = PartialWitness::new();
        pw.set_hash_target(self.block_root, block_root.clone());
        self.block_merkle_proof_for_withdraw
            .set_witness(&mut pw, block_merkle_proof_for_withdraw);
        self.block_merkle_proof_for_evidence
            .set_witness(&mut pw, block_merkle_proof_for_evidence);
        pw.set_proof_with_pis_target(&self.withdraw_proof, withdraw_proof);
        self.transfer_info.set_witness(&mut pw, transfer_info);
        self.data.prove(pw)
    }
}

impl<F, C, const D: usize> DynamicLeafableCircuit<F, C, D> for SettlementLeafCircuit<F, C, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
{
    fn add_proof_target_and_conditionally_verify(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        condition: &BoolTarget,
    ) -> ProofWithPublicInputsTarget<D> {
        let proof = builder.add_virtual_proof_with_pis(&self.data.common);
        let vd = builder.constant_verifier_data(&self.data.verifier_only);
        builder
            .conditionally_verify_proof_or_dummy::<C>(*condition, &proof, &vd, &self.data.common)
            .unwrap();
        proof
    }

    fn dummy_leaf(&self) -> crate::utils::dummy::DummyProof<F, C, D> {
        DummyProof::<F, C, D>::new(&self.data.common)
    }
}

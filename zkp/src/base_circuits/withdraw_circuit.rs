use anyhow::Ok;
use plonky2::{
    field::{extension::Extendable, types::PrimeField64},
    gates::noop::NoopGate,
    hash::hash_types::RichField,
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, Witness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, CommonCircuitData, VerifierCircuitTarget},
        config::{AlgebraicHasher, GenericConfig},
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
    recursion::{
        cyclic_recursion::check_cyclic_proof_verifier_data, dummy_circuit::cyclic_base_proof,
    },
};
use plonky2_u32::gadgets::arithmetic_u32::CircuitBuilderU32;
use serde::{Deserialize, Serialize};

use crate::{
    base_circuits::block_tree_circuit::BlockTreePublicInputs,
    common::{
        address::{Address, AddressTarget, ADDRESS_VEC_LEN},
        asset::{Assets, AssetsTarget, ASSETS_VEC_LEN},
        block::{Block, BlockTarget, BLOCK_VEC_LEN},
        extended_block_number::{ExtendedBlockNumber, ExtendedBlockNumberTarget},
        transfer_info::{TransferInfo, TransferInfoTarget},
    },
    constants::WITHDRAW_PADDING_DEGREE,
    utils::{
        logic::enforce_equal_targets_if_enabled,
        trees::merkle_tree_with_leaves::{MerkleProofWithLeaves, MerkleProofWithLeavesTarget},
    },
};

use super::block_tree_circuit::{BlockTreeCircuit, BlockTreePublicInputsTarget};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawPublicInputs {
    pub recipient: Address,
    pub total_amount: Assets,
    pub start_ebn: ExtendedBlockNumber,
    pub end_ebn: ExtendedBlockNumber,
    pub block: Block,
}

impl WithdrawPublicInputs {
    pub fn to_vec<F: PrimeField64>(&self) -> Vec<F> {
        let mut result = Vec::new();
        result.extend(self.recipient.to_vec::<F>());
        result.extend(self.total_amount.to_vec::<F>());
        result.extend(self.start_ebn.to_vec::<F>());
        result.extend(self.end_ebn.to_vec::<F>());
        result.extend(self.block.to_vec::<F>());
        assert_eq!(
            result.len(),
            ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1 + 1 + BLOCK_VEC_LEN
        );
        result
    }

    pub fn from_pis<F: PrimeField64>(input: &[F]) -> Self {
        let input = input[0..ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1 + 1 + BLOCK_VEC_LEN].to_vec();
        let recipient = Address::from_vec(&input[0..ADDRESS_VEC_LEN]);
        let total_amount =
            Assets::from_vec(&input[ADDRESS_VEC_LEN..ADDRESS_VEC_LEN + ASSETS_VEC_LEN]);
        let start_ebn = ExtendedBlockNumber::from_vec(
            &input[ADDRESS_VEC_LEN + ASSETS_VEC_LEN..ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1],
        );
        let end_ebn = ExtendedBlockNumber::from_vec(
            &input[ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1..ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1 + 1],
        );
        let block = Block::from_vec(
            &input[ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1 + 1
                ..ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1 + 1 + BLOCK_VEC_LEN],
        );
        Self {
            recipient,
            total_amount,
            start_ebn,
            end_ebn,
            block,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WithdrawPublicInputsTarget {
    pub recipient: AddressTarget,
    pub total_amount: AssetsTarget,
    pub start_ebn: ExtendedBlockNumberTarget,
    pub end_ebn: ExtendedBlockNumberTarget,
    pub block: BlockTarget,
}

impl WithdrawPublicInputsTarget {
    pub fn new<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self {
        Self {
            recipient: AddressTarget::new(builder),
            total_amount: AssetsTarget::new(builder),
            start_ebn: ExtendedBlockNumberTarget::new(builder),
            end_ebn: ExtendedBlockNumberTarget::new(builder),
            block: BlockTarget::new(builder),
        }
    }

    pub fn constant<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
        input: &WithdrawPublicInputs,
    ) -> Self {
        Self {
            recipient: AddressTarget::constant(builder, input.recipient),
            total_amount: AssetsTarget::constant(builder, &input.total_amount),
            start_ebn: ExtendedBlockNumberTarget::constant(builder, &input.start_ebn),
            end_ebn: ExtendedBlockNumberTarget::constant(builder, &input.end_ebn),
            block: BlockTarget::constant(builder, &input.block),
        }
    }

    pub fn connect<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
        other: &Self,
    ) {
        self.recipient.connect(builder, &other.recipient);
        self.total_amount.connect(builder, &other.total_amount);
        self.start_ebn.connect(builder, &other.start_ebn);
        self.end_ebn.connect(builder, &other.end_ebn);
        self.block.connect(builder, &other.block);
    }

    pub fn to_vec(&self) -> Vec<Target> {
        let mut result = Vec::new();
        result.extend(self.recipient.to_vec());
        result.extend(self.total_amount.to_vec());
        result.extend(self.start_ebn.to_vec());
        result.extend(self.end_ebn.to_vec());
        result.extend(self.block.to_vec());
        assert_eq!(
            result.len(),
            ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1 + 1 + BLOCK_VEC_LEN
        );
        result
    }

    pub fn from_pis(input: &[Target]) -> Self {
        let input = input[0..ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1 + 1 + BLOCK_VEC_LEN].to_vec();
        let recipient = AddressTarget::from_vec(&input[0..ADDRESS_VEC_LEN]);
        let total_amount =
            AssetsTarget::from_vec(&input[ADDRESS_VEC_LEN..ADDRESS_VEC_LEN + ASSETS_VEC_LEN]);
        let start_ebn = ExtendedBlockNumberTarget::from_vec(
            &input[ADDRESS_VEC_LEN + ASSETS_VEC_LEN..ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1],
        );
        let end_ebn = ExtendedBlockNumberTarget::from_vec(
            &input[ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1..ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1 + 1],
        );
        let block = BlockTarget::from_vec(
            &input[ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1 + 1
                ..ADDRESS_VEC_LEN + ASSETS_VEC_LEN + 1 + 1 + BLOCK_VEC_LEN],
        );
        Self {
            recipient,
            total_amount,
            start_ebn,
            end_ebn,
            block,
        }
    }

    pub fn set_witness<F: RichField>(
        &self,
        pw: &mut impl Witness<F>,
        value: &WithdrawPublicInputs,
    ) {
        self.recipient.set_witness(pw, value.recipient);
        self.total_amount.set_witness(pw, &value.total_amount);
        self.start_ebn.set_witness(pw, &value.start_ebn);
        self.end_ebn.set_witness(pw, &value.end_ebn);
        self.block.set_witness(pw, &value.block);
    }
}

#[derive(Debug, Clone)]
pub struct WithdrawValue<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    pub is_first_step: bool,
    pub prev_pis: WithdrawPublicInputs,
    pub new_pis: WithdrawPublicInputs,
    pub transfer_info: TransferInfo<F>,
    pub block_tree_proof: ProofWithPublicInputs<F, C, D>,
    pub block_merkle_proof_prev: MerkleProofWithLeaves<F, Block>, // merkle proof for the prev_block
    pub block_merkle_proof_transfer: MerkleProofWithLeaves<F, Block>, // merkle proof for the transfer info
}

impl<F: RichField + Extendable<D>, const D: usize, C: GenericConfig<D, F = F> + 'static>
    WithdrawValue<F, C, D>
where
    C::Hasher: AlgebraicHasher<F>,
{
    pub fn new(
        block_tree_circuit: &BlockTreeCircuit<F, C, D>,
        is_first_step: bool,
        prev_pis: WithdrawPublicInputs,
        transfer_info: TransferInfo<F>,
        block_tree_proof: ProofWithPublicInputs<F, C, D>,
        block_merkle_proof_prev: MerkleProofWithLeaves<F, Block>,
        block_merkle_proof_transfer: MerkleProofWithLeaves<F, Block>,
    ) -> anyhow::Result<Self> {
        let new_recipient = prev_pis.recipient;
        transfer_info
            .verify()
            .map_err(|_| anyhow::anyhow!("transfer_info is invalid"))?;
        assert!(
            transfer_info.transfer.recipient == new_recipient,
            "recipient mismatch"
        );
        let (new_start_ebn, new_end_ebn, new_total_amount) = if is_first_step {
            assert_eq!(
                prev_pis.total_amount,
                Assets::default(),
                "prev_total_amount must be zero"
            );
            let new_total_amount = prev_pis.total_amount.clone() + transfer_info.transfer.asset;
            let new_start_ebn = transfer_info.ebn();
            let new_end_ebn = new_start_ebn;
            (new_start_ebn, new_end_ebn, new_total_amount)
        } else {
            let new_total_amount = prev_pis.total_amount.clone() + transfer_info.transfer.asset;
            let new_start_ebn = prev_pis.start_ebn;
            let new_end_ebn = transfer_info.ebn();
            assert!(
                prev_pis.end_ebn < new_end_ebn,
                "prev_end_ebn must be less than new_end_ebn"
            );
            (new_start_ebn, new_end_ebn, new_total_amount)
        };

        block_tree_circuit.verify(block_tree_proof.clone())?;
        let block_tree_pis = BlockTreePublicInputs::from_pis(&block_tree_proof.public_inputs);
        block_merkle_proof_prev.verify(
            &prev_pis.block,
            prev_pis.block.block_number as usize,
            block_tree_pis.block_root,
        )?;
        block_merkle_proof_transfer.verify(
            &transfer_info.block,
            transfer_info.block.block_number as usize,
            block_tree_pis.block_root,
        )?;
        let new_block = block_tree_pis.block.clone();
        let new_pis = WithdrawPublicInputs {
            recipient: new_recipient,
            total_amount: new_total_amount,
            start_ebn: new_start_ebn,
            end_ebn: new_end_ebn,
            block: new_block,
        };
        Ok(Self {
            is_first_step,
            prev_pis,
            new_pis,
            transfer_info,
            block_tree_proof,
            block_merkle_proof_prev,
            block_merkle_proof_transfer,
        })
    }
}

#[derive(Debug, Clone)]
pub struct WithdrawTarget<const D: usize> {
    is_first_step: BoolTarget,
    prev_pis: WithdrawPublicInputsTarget,
    new_pis: WithdrawPublicInputsTarget,
    transfer_info: TransferInfoTarget,
    block_tree_proof: ProofWithPublicInputsTarget<D>,
    block_merkle_proof_prev: MerkleProofWithLeavesTarget<BlockTarget>,
    block_merkle_proof_transfer: MerkleProofWithLeavesTarget<BlockTarget>,
}

impl<const D: usize> WithdrawTarget<D> {
    pub fn new<F: RichField + Extendable<D>, C: GenericConfig<D, F = F> + 'static>(
        block_tree_circuit: &BlockTreeCircuit<F, C, D>,
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self
    where
        C::Hasher: AlgebraicHasher<F>,
    {
        let is_first_step = builder.add_virtual_bool_target_safe();
        let is_not_first_step = builder.not(is_first_step);

        let transfer_info = TransferInfoTarget::new(builder);
        let prev_pis = WithdrawPublicInputsTarget::new(builder);

        // new_recipient = prev_recipient = transfer_info.transfer.recipient
        prev_pis
            .recipient
            .connect(builder, &transfer_info.transfer.recipient);
        let new_recipient = prev_pis.recipient.clone();

        // prev_total_amount == 0 if is_first_step == true
        let zero_assets = AssetsTarget::constant(builder, &Assets::default());
        enforce_equal_targets_if_enabled(
            builder,
            &prev_pis.total_amount.to_vec(),
            &zero_assets.to_vec(),
            is_first_step,
        );

        // new_total_amount = prev_total_amount + transfer_info.transfer.amount
        let transfer_amount = AssetsTarget::from_asset(builder, &transfer_info.transfer.amount);
        let new_total_amount = AssetsTarget::add(builder, &prev_pis.total_amount, &transfer_amount);

        // new_start_ebn == prev_start_ebn if is_not_first_step == true
        let new_start_ebn = ExtendedBlockNumberTarget::new(builder);
        enforce_equal_targets_if_enabled(
            builder,
            &prev_pis.start_ebn.to_vec(),
            &new_start_ebn.to_vec(),
            is_not_first_step,
        );

        // constrain prev_end_ebn < new_end_ebn if is_first_step == false
        let new_end_ebn = transfer_info.ebn(builder);
        let zero_ebn =
            ExtendedBlockNumberTarget::constant(builder, &ExtendedBlockNumber::default());
        let prev_end_or_zero_ebn = ExtendedBlockNumberTarget::select(
            builder,
            is_first_step,
            zero_ebn,
            prev_pis.end_ebn.clone(),
        );
        prev_end_or_zero_ebn.less_than(builder, &new_end_ebn);

        // verify block merkle proof
        let block_tree_proof = block_tree_circuit.add_proof_target_and_verify(builder);
        let block_tree_pis = BlockTreePublicInputsTarget::from_pis(&block_tree_proof.public_inputs);
        let block_merkle_proof_prev = MerkleProofWithLeavesTarget::new(builder, 32);
        let block_merkle_proof_transfer = MerkleProofWithLeavesTarget::new(builder, 32);
        block_merkle_proof_prev.verify(
            builder,
            &prev_pis.block,
            prev_pis.block.block_number.0,
            block_tree_pis.block_root,
        );
        block_merkle_proof_transfer.verify(
            builder,
            &transfer_info.block,
            transfer_info.block.block_number.0,
            block_tree_pis.block_root,
        );
        let new_block = block_tree_pis.block.clone();

        let new_pis = WithdrawPublicInputsTarget {
            recipient: new_recipient,
            total_amount: new_total_amount,
            start_ebn: new_start_ebn,
            end_ebn: new_end_ebn,
            block: new_block,
        };
        Self {
            is_first_step,
            prev_pis,
            new_pis,
            transfer_info,
            block_tree_proof,
            block_merkle_proof_prev,
            block_merkle_proof_transfer,
        }
    }

    pub fn set_witness<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>>(
        &self,
        pw: &mut impl Witness<F>,
        value: &WithdrawValue<F, C, D>,
    ) where
        C::Hasher: AlgebraicHasher<F>,
    {
        pw.set_bool_target(self.is_first_step, value.is_first_step);
        self.prev_pis.set_witness(pw, &value.prev_pis);
        self.new_pis.set_witness(pw, &value.new_pis);
        self.transfer_info.set_witness(pw, &value.transfer_info);
        pw.set_proof_with_pis_target(&self.block_tree_proof, &value.block_tree_proof);
        self.block_merkle_proof_prev
            .set_witness(pw, &value.block_merkle_proof_prev);
        self.block_merkle_proof_transfer
            .set_witness(pw, &value.block_merkle_proof_transfer);
    }
}

#[derive(Debug)]
pub struct WithdrawCircuit<F, C, const D: usize>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
{
    pub data: CircuitData<F, C, D>,
    pub target: WithdrawTarget<D>,
    pub is_first_step: BoolTarget,
    pub prev_proof: ProofWithPublicInputsTarget<D>,
    pub verifier_data_target: VerifierCircuitTarget,
}

impl<F, C, const D: usize> WithdrawCircuit<F, C, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
{
    pub fn new(block_tree_circuit: &BlockTreeCircuit<F, C, D>) -> Self {
        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::default());
        let target = WithdrawTarget::new(block_tree_circuit, &mut builder);
        builder.register_public_inputs(&target.new_pis.to_vec());

        let mut common_data = common_data_for_withdraw::<F, C, D>();
        let verifier_data_target = builder.add_verifier_data_public_inputs();
        common_data.num_public_inputs = builder.num_public_inputs();

        let is_first_step = builder.add_virtual_bool_target_safe(); // whether this circuit uses recursive proof or not
        let is_not_first_step = builder.not(is_first_step);

        let prev_proof = builder.add_virtual_proof_with_pis(&common_data);
        let prev_pis = WithdrawPublicInputsTarget::from_pis(&prev_proof.public_inputs);

        target.prev_pis.connect(&mut builder, &prev_pis);

        // Verify a cyclic proof.
        builder
            .conditionally_verify_cyclic_proof_or_dummy::<C>(
                is_not_first_step,
                &prev_proof,
                &common_data,
            )
            .unwrap();
        let circuit_data = builder.build::<C>();
        debug_assert_eq!(circuit_data.common, common_data);
        Self {
            data: circuit_data,
            target,
            is_first_step,
            prev_proof,
            verifier_data_target,
        }
    }

    pub fn prove(
        &self,
        value: &WithdrawValue<F, C, D>,
        prev_proof: Option<ProofWithPublicInputs<F, C, D>>,
    ) -> anyhow::Result<ProofWithPublicInputs<F, C, D>> {
        let mut pw = PartialWitness::<F>::new();
        pw.set_verifier_data_target(&self.verifier_data_target, &self.data.verifier_only);
        self.target.set_witness(&mut pw, value);
        if let Some(prev_proof) = prev_proof {
            pw.set_bool_target(self.is_first_step, false);
            pw.set_proof_with_pis_target::<C, D>(&self.prev_proof, &prev_proof);
        } else {
            let init_pis = WithdrawPublicInputs {
                recipient: value.transfer_info.transfer.recipient,
                total_amount: Assets::default(),
                start_ebn: ExtendedBlockNumber::default(),
                end_ebn: ExtendedBlockNumber::default(),
                block: Block::default(),
            };
            let dummy_proof = cyclic_base_proof(
                &self.data.common,
                &self.data.verifier_only,
                init_pis.to_vec().into_iter().enumerate().collect(),
            );
            pw.set_bool_target(self.is_first_step, true);
            pw.set_proof_with_pis_target::<C, D>(&self.prev_proof, &dummy_proof);
        };
        self.data.prove(pw)
    }

    pub fn verify(&self, proof_with_pis: &ProofWithPublicInputs<F, C, D>) -> anyhow::Result<()> {
        check_cyclic_proof_verifier_data(
            &proof_with_pis,
            &self.data.verifier_only,
            &self.data.common,
        )?;
        self.data.verify(proof_with_pis.clone())
    }

    pub fn add_proof_target_and_verify(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    ) -> ProofWithPublicInputsTarget<D> {
        let proof = builder.add_virtual_proof_with_pis(&self.data.common);
        let vd_target = builder.constant_verifier_data(&self.data.verifier_only);
        let inner_vd_target =
            VerifierCircuitTarget::from_slice::<F, D>(&proof.public_inputs, &self.data.common)
                .unwrap();
        builder.connect_hashes(vd_target.circuit_digest, inner_vd_target.circuit_digest);
        builder.connect_merkle_caps(
            &vd_target.constants_sigmas_cap,
            &inner_vd_target.constants_sigmas_cap,
        );
        builder.verify_proof::<C>(&proof, &vd_target, &self.data.common);
        proof
    }
}

// Generates `CommonCircuitData` usable for recursion.
pub fn common_data_for_withdraw<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
>() -> CommonCircuitData<F, D>
where
    C::Hasher: AlgebraicHasher<F>,
{
    let config = CircuitConfig::standard_recursion_config();
    let builder = CircuitBuilder::<F, D>::new(config);
    let data = builder.build::<C>();

    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);
    let proof = builder.add_virtual_proof_with_pis(&data.common);
    let verifier_data = VerifierCircuitTarget {
        constants_sigmas_cap: builder.add_virtual_cap(data.common.config.fri_config.cap_height),
        circuit_digest: builder.add_virtual_hash(),
    };
    builder.verify_proof::<C>(&proof, &verifier_data, &data.common);
    let data = builder.build::<C>();

    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);
    let proof = builder.add_virtual_proof_with_pis(&data.common);
    let verifier_data = VerifierCircuitTarget {
        constants_sigmas_cap: builder.add_virtual_cap(data.common.config.fri_config.cap_height),
        circuit_digest: builder.add_virtual_hash(),
    };
    builder.verify_proof::<C>(&proof, &verifier_data, &data.common);
    let zero = builder.zero_u32();
    let _ = builder.add_many_u32(&[zero, zero, zero]);
    let _ = builder.sub_u32(zero, zero, zero);
    let _zero_limbs = [(); 8].map(|_| builder.zero_u32());
    while builder.num_gates() < 1 << WITHDRAW_PADDING_DEGREE {
        builder.add_gate(NoopGate, vec![]);
    }
    builder.build::<C>().common
}

#[cfg(test)]
mod tests {
    use crate::{
        base_circuits::block_tree_circuit::{BlockTreeCircuit, BlockTreeValue},
        common::{
            address::Address, asset::Assets, block::Block,
            extended_block_number::ExtendedBlockNumber, transfer::Transfer,
            transfer_info::TransferInfo,
        },
        constants::TRANSFER_TREE_HEIGHT,
        random::transfers::generate_random_transfers,
        utils::trees::merkle_tree_with_leaves::MerkleTreeWithLeaves,
    };
    use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

    use super::{WithdrawCircuit, WithdrawPublicInputs, WithdrawValue};

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_withdraw_circuit() {
        let mut rng = rand::thread_rng();
        let recipient = Address::rand(&mut rng);
        let latest_block_number = 2;
        let transfers_vec =
            generate_random_transfers::<F, _>(&mut rng, latest_block_number, 4, &[recipient]);

        let block_tree_circuit = BlockTreeCircuit::<F, C, D>::new();
        let mut block_tree = MerkleTreeWithLeaves::<F, Block>::new(32);
        block_tree.push(Block::default());
        let mut block_tree_proof = None;

        let mut prev_block = Block::default();
        let mut transfer_info_vec = Vec::new();

        // generate block and block tree proof
        for transfers in transfers_vec {
            let mut transfer_tree = MerkleTreeWithLeaves::<F, Transfer>::new(TRANSFER_TREE_HEIGHT);
            for transfer in transfers.iter() {
                transfer_tree.push(*transfer);
            }
            let transfer_tree_root = transfer_tree.get_root();
            let block = Block {
                prev_block_hash: prev_block.block_hash(),
                transfer_tree_root: transfer_tree_root.into(),
                total_deposit: Assets::default(),
                block_number: prev_block.block_number + 1,
            };
            let transfer_info: Vec<TransferInfo<F>> = transfers
                .iter()
                .enumerate()
                .map(|(transfer_index, transfer)| {
                    let transfer_merkle_proof = transfer_tree.prove(transfer_index);
                    TransferInfo {
                        transfer: *transfer,
                        transfer_index,
                        transfer_merkle_proof,
                        block: block.clone(),
                    }
                })
                .collect();
            transfer_info_vec.extend(transfer_info.clone());

            // block tree transition proof
            let block_merkle_proof = block_tree.prove(block.block_number as usize);
            let prev_block_root = block_tree.get_root();
            block_tree.push(block.clone());
            let new_block_root = block_tree.get_root();
            let block_tree_value = BlockTreeValue::new(
                block.clone(),
                prev_block_root,
                new_block_root,
                block_merkle_proof,
            );
            block_tree_proof = Some(
                block_tree_circuit
                    .prove(&block_tree_value, &block_tree_proof)
                    .unwrap(),
            );
            prev_block = block;
        }

        let withdraw_circuit = WithdrawCircuit::<F, C, D>::new(&block_tree_circuit);

        let mut withdraw_proof = None;
        for transfer_info in transfer_info_vec {
            if withdraw_proof.is_none() {
                let withdraw_pis = WithdrawPublicInputs {
                    recipient: transfer_info.transfer.recipient,
                    total_amount: Assets::default(),
                    start_ebn: ExtendedBlockNumber::default(),
                    end_ebn: ExtendedBlockNumber::default(),
                    block: Block::default(),
                };
                let block_merkle_proof_prev = block_tree.prove(0);
                let block_merkle_proof_transfer =
                    block_tree.prove(transfer_info.block.block_number as usize);
                let withdraw_value = WithdrawValue::new(
                    &block_tree_circuit,
                    true,
                    withdraw_pis,
                    transfer_info,
                    block_tree_proof.clone().unwrap(),
                    block_merkle_proof_prev,
                    block_merkle_proof_transfer,
                )
                .unwrap();
                withdraw_proof = Some(
                    withdraw_circuit
                        .prove(&withdraw_value, withdraw_proof)
                        .unwrap(),
                );
            } else {
                let withdraw_pis =
                    WithdrawPublicInputs::from_pis(&withdraw_proof.clone().unwrap().public_inputs);
                let block_merkle_proof_prev =
                    block_tree.prove(withdraw_pis.block.block_number as usize);
                let block_merkle_proof_transfer =
                    block_tree.prove(transfer_info.block.block_number as usize);
                let withdraw_value = WithdrawValue::new(
                    &block_tree_circuit,
                    false,
                    withdraw_pis,
                    transfer_info,
                    block_tree_proof.clone().unwrap(),
                    block_merkle_proof_prev,
                    block_merkle_proof_transfer,
                );
                withdraw_proof = Some(
                    withdraw_circuit
                        .prove(&withdraw_value.unwrap(), withdraw_proof)
                        .unwrap(),
                );
            }
        }
    }
}

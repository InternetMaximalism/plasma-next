use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOut, RichField},
    plonk::{
        config::{AlgebraicHasher, GenericConfig},
        proof::ProofWithPublicInputs,
    },
};
use rand::Rng;

use crate::{
    base_circuits::{
        block_tree_circuit::{BlockTreeCircuit, BlockTreeValue},
        spent_circuit::SpentValue,
        withdraw_circuit::{WithdrawCircuit, WithdrawPublicInputs, WithdrawValue},
    },
    common::{
        address::Address, asset::Assets, block::Block, extended_block_number::ExtendedBlockNumber,
        transfer::Transfer, transfer_info::TransferInfo,
    },
    constants::TRANSFER_TREE_HEIGHT,
    utils::trees::merkle_tree_with_leaves::{MerkleProofWithLeaves, MerkleTreeWithLeaves},
};

use super::transfers::generate_random_transfers;

#[derive(Debug, Clone)]
pub struct RandomSettlementProof<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
> {
    pub block_root: HashOut<F>,
    pub block_merkle_proof_for_withdraw: MerkleProofWithLeaves<F, Block>,
    pub block_merkle_proof_for_evidence: MerkleProofWithLeaves<F, Block>,
    pub withdraw_proof: ProofWithPublicInputs<F, C, D>,
    pub transfer_info: TransferInfo<F>,
}

pub fn generate_random_settlement<F, C, const D: usize, R: Rng>(
    block_tree_circuit: &BlockTreeCircuit<F, C, D>,
    withdraw_circuit: &WithdrawCircuit<F, C, D>,
    rng: &mut R,
    num_blocks: usize,
    num_transfers: usize,
    recipients: &[Address],
) -> Vec<RandomSettlementProof<F, C, D>>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
{
    let transfers_vec =
        generate_random_transfers::<F, _>(rng, num_blocks, num_transfers, recipients);

    let mut block = Block::default();
    let total_deposit = Assets::rand_full(rng);
    let mut block_tree_proof = None;
    let mut block_tree = MerkleTreeWithLeaves::<F, Block>::new(32);

    block_tree.push(block.clone());

    let mut transfer_info_vec = Vec::new();

    for transfers in &transfers_vec {
        let spent_value = SpentValue::new::<F>(
            &transfers,
            &total_deposit,
            &block.block_hash(),
            block.block_number + 1,
        );

        let new_block = spent_value.new_block.clone();

        // generate transfer_info
        {
            let mut transfer_tree = MerkleTreeWithLeaves::<F, Transfer>::new(TRANSFER_TREE_HEIGHT);
            for transfer in transfers.iter() {
                transfer_tree.push(*transfer);
            }

            let transfer_info: Vec<TransferInfo<F>> = transfers
                .iter()
                .enumerate()
                .map(|(transfer_index, transfer)| {
                    let transfer_merkle_proof = transfer_tree.prove(transfer_index);
                    TransferInfo {
                        transfer: *transfer,
                        transfer_index,
                        transfer_merkle_proof,
                        block: new_block.clone(),
                    }
                })
                .collect();
            transfer_info_vec.extend(transfer_info);
        }

        // generate block tree proof
        {
            let prev_block_root = block_tree.get_root();
            let block_merkle_proof = block_tree.prove(new_block.block_number as usize);
            block_tree.push(new_block.clone());
            let new_block_root = block_tree.get_root();
            let block_value = BlockTreeValue::new(
                new_block.clone(),
                prev_block_root,
                new_block_root,
                block_merkle_proof,
            );
            block_tree_proof = Some(
                block_tree_circuit
                    .prove(&block_value, &block_tree_proof)
                    .unwrap(),
            );
            block = new_block;
        }
    }

    // generate settlement proofs
    let mut settlement_proofs = Vec::new();

    for recipient in recipients {
        let transfer_info = transfer_info_vec
            .iter()
            .filter(|transfer_info| transfer_info.transfer.recipient == *recipient)
            .cloned()
            .collect::<Vec<_>>();

        let mut is_first_step = true;
        let mut pis = WithdrawPublicInputs {
            recipient: recipient.clone(),
            total_amount: Assets::default(),
            start_ebn: ExtendedBlockNumber::default(),
            end_ebn: ExtendedBlockNumber::default(),
            block: Block::default(),
        };
        let mut withdraw_proof = None;
        for t in transfer_info.iter() {
            let block_merkle_proof_prev = block_tree.prove(pis.block.block_number as usize);
            let block_merkle_proof_transfer = block_tree.prove(t.block.block_number as usize);
            let withdraw_value = WithdrawValue::new(
                block_tree_circuit,
                is_first_step,
                pis.clone(),
                t.clone(),
                block_tree_proof.clone().unwrap(),
                block_merkle_proof_prev,
                block_merkle_proof_transfer,
            )
            .unwrap();

            withdraw_proof = Some(
                withdraw_circuit
                    .prove(&withdraw_value, withdraw_proof.clone())
                    .unwrap(),
            );

            pis = withdraw_value.new_pis.clone();
            if is_first_step {
                is_first_step = false;
            }
        }

        let random_transfer_info = transfer_info[rng.gen_range(0..transfer_info.len())].clone();
        let block_merkle_proof_for_withdraw = block_tree.prove(pis.block.block_number as usize);
        let block_merkle_proof_for_evidence =
            block_tree.prove(random_transfer_info.block.block_number as usize);
        let block_root = block_tree.get_root();
        let withdraw_proof = withdraw_proof.unwrap();
        let random_settlement_proof = RandomSettlementProof {
            block_root,
            block_merkle_proof_for_withdraw,
            block_merkle_proof_for_evidence,
            withdraw_proof,
            transfer_info: random_transfer_info,
        };
        settlement_proofs.push(random_settlement_proof);
    }
    settlement_proofs
}

#[cfg(test)]
mod tests {
    use crate::{
        base_circuits::{
            block_tree_circuit::BlockTreeCircuit,
            withdraw_circuit::{WithdrawCircuit, WithdrawPublicInputs},
        },
        common::address::Address,
        random::withdraw::generate_random_settlement,
    };
    use plonky2::{
        field::goldilocks_field::GoldilocksField, plonk::config::PoseidonGoldilocksConfig,
    };

    const D: usize = 2;
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;

    #[test]
    fn test_random_withdraw() {
        let mut rng = rand::thread_rng();
        let num_blocks = 4;
        let num_transfers = 3;
        let recipient = Address::rand(&mut rng);
        let block_tree_circuit = BlockTreeCircuit::new();
        let withdraw_circuit = WithdrawCircuit::<F, C, D>::new(&block_tree_circuit);

        let random_settlement_proofs = generate_random_settlement::<F, C, D, _>(
            &block_tree_circuit,
            &withdraw_circuit,
            &mut rng,
            num_blocks,
            num_transfers,
            &[recipient],
        );
        let pis = WithdrawPublicInputs::from_pis(
            &random_settlement_proofs[0].withdraw_proof.public_inputs,
        );
        println!(
            "start_ebn: {}, end_ebn: {}, total_amount: {}",
            pis.start_ebn, pis.end_ebn, pis.total_amount
        );
    }
}

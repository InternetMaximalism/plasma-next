use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    plonk::{
        config::{AlgebraicHasher, GenericConfig},
        proof::ProofWithPublicInputs,
    },
};
use rand::Rng;

use crate::{
    base_circuits::withdraw_circuit::{WithdrawCircuit, WithdrawPublicInputs, WithdrawValue},
    common::{address::Address, asset::Assets},
};

use super::etmp::generate_random_etmps;

pub fn generate_random_withdraw_proofs<F, C, const D: usize, R: Rng>(
    rng: &mut R,
    num_blocks: usize,
    num_transfers_per_recipient: usize,
    recipients: &[Address],
    withdraw_circuit: &WithdrawCircuit<F, C, D>,
) -> Vec<ProofWithPublicInputs<F, C, D>>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
{
    let etmps = generate_random_etmps::<F, C, D, _>(
        rng,
        num_blocks,
        num_transfers_per_recipient,
        recipients,
    );

    let mut recipient_etmps_vec = vec![];
    for recipient in recipients.iter() {
        let mut recipient_etmps = etmps
            .iter()
            .filter(|etmp| &etmp.transfer.recipient == recipient)
            .collect::<Vec<_>>();
        recipient_etmps.sort_by_key(|w| w.extended_block_number());
        recipient_etmps_vec.push(recipient_etmps);
    }
    let mut proofs = vec![];
    for recipient_etmps in recipient_etmps_vec.iter() {
        if recipient_etmps.is_empty() {
            continue;
        }
        let init_ebn = recipient_etmps[0].extended_block_number().sub_one();
        let recipient = recipient_etmps[0].transfer.recipient;
        let block_root = recipient_etmps[0].block_root;
        let init_pis = WithdrawPublicInputs {
            block_root,
            recipient,
            total_amount: Assets::default(),
            init_extended_block_number: init_ebn,
            extended_block_number: init_ebn,
        };
        let mut prev_ebn = init_pis.init_extended_block_number;
        let mut prev_total_amount = init_pis.total_amount.clone();
        let mut proof = None;
        for etmp in etmps.iter() {
            let w = WithdrawValue::new(etmp.clone(), prev_total_amount.clone(), init_ebn, prev_ebn);
            proof = if proof.is_none() {
                Some(withdraw_circuit.prove(&w, None, &init_pis).unwrap())
            } else {
                Some(withdraw_circuit.prove(&w, proof, &init_pis).unwrap())
            };
            prev_ebn = etmp.extended_block_number();
            prev_total_amount += etmp.transfer.asset.clone();
        }
        proofs.push(proof.unwrap());
    }
    proofs
}

#[cfg(test)]
mod tests {
    use crate::{base_circuits::withdraw_circuit::WithdrawCircuit, common::address::Address};
    use plonky2::{
        field::goldilocks_field::GoldilocksField, plonk::config::PoseidonGoldilocksConfig,
    };

    use super::generate_random_withdraw_proofs;
    const D: usize = 2;
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;

    #[test]
    fn test_generate_random_withdraw_proofs() {
        let mut rng = rand::thread_rng();
        let recipients = vec![Address::rand(&mut rng)];
        let withdraw_circuit = WithdrawCircuit::<F, C, D>::new();
        generate_random_withdraw_proofs(&mut rng, 1, 1, &recipients, &withdraw_circuit);
    }
}

use plonky2::hash::hash_types::RichField;
use rand::{seq::SliceRandom, Rng};

use crate::common::{address::Address, asset::Asset, transfer::Transfer};

use super::utils::generate_random_vec;

pub fn generate_random_transfers<F: RichField, R: Rng>(
    rng: &mut R,
    num_blocks: usize,
    num_transfers_per_recipient: usize,
    recipients: &[Address],
) -> Vec<Vec<Transfer>> {
    let mut transfers = recipients
        .iter()
        .flat_map(|r| {
            (0..num_transfers_per_recipient)
                .map(|_| Transfer {
                    recipient: *r,
                    asset: Asset::rand(rng),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    transfers.shuffle(rng);
    let num_transfers_vec = generate_random_vec(
        rng,
        num_blocks,
        num_transfers_per_recipient * recipients.len(),
    );
    let mut result = vec![];
    for num_transfer in num_transfers_vec {
        result.push(transfers.drain(0..num_transfer).collect::<Vec<_>>());
    }
    result
}

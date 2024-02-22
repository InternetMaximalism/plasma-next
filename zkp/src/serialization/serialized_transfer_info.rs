use plonky2::field::goldilocks_field::GoldilocksField;
use serde::{Deserialize, Serialize};

use crate::{
    common::{block::Block, transfer::Transfer, transfer_info::TransferInfo},
    utils::trees::{merkle_tree::MerkleProof, merkle_tree_with_leaves::MerkleProofWithLeaves},
};

use super::serialized_hashout::SerializedHashOut;

type F = GoldilocksField;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializedTransferInfo {
    pub transfer: Transfer,
    pub transfer_index: usize,
    pub transfer_merkle_proof: Vec<SerializedHashOut>,
    pub block: Block,
}

impl From<TransferInfo<F>> for SerializedTransferInfo {
    fn from(value: TransferInfo<F>) -> Self {
        let transfer_merkle_proof = value
            .transfer_merkle_proof
            .0
            .siblings
            .iter()
            .map(|h| SerializedHashOut(*h))
            .collect::<Vec<_>>();
        Self {
            transfer: value.transfer,
            transfer_index: value.transfer_index,
            transfer_merkle_proof,
            block: value.block,
        }
    }
}

impl From<SerializedTransferInfo> for TransferInfo<F> {
    fn from(value: SerializedTransferInfo) -> Self {
        let siblings = value
            .transfer_merkle_proof
            .iter()
            .map(|h| h.0)
            .collect::<Vec<_>>();
        let transfer_merkle_proof = MerkleProofWithLeaves(MerkleProof { siblings });
        Self {
            transfer: value.transfer,
            transfer_index: value.transfer_index,
            transfer_merkle_proof,
            block: value.block,
        }
    }
}

#[cfg(test)]
mod tests {
    use plonky2::{
        field::goldilocks_field::GoldilocksField, plonk::config::PoseidonGoldilocksConfig,
    };

    use crate::{
        common::{address::Address, transfer_info::TransferInfo},
        random::etmp::generate_random_etmps,
    };

    use super::SerializedTransferInfo;

    const D: usize = 2;
    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;

    #[test]
    fn test_convert_transfer_info() {
        let mut rng = rand::thread_rng();

        let recipient = Address::rand(&mut rng);
        let etmp = generate_random_etmps::<F, C, D, _>(&mut rng, 1, 1, &[recipient])[0].clone();
        let serialized_transfer_info: SerializedTransferInfo = etmp.transfer_info.into();
        let transfer_info_recovered: TransferInfo<F> = serialized_transfer_info.clone().into();
        let serialized_transfer_info2: SerializedTransferInfo = transfer_info_recovered.into();
        assert_eq!(serialized_transfer_info, serialized_transfer_info2);
    }
}

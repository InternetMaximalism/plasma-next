/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {ITransfer} from "../common-interface/ITransfer.sol";

library TransferLib {
    function transferCommitment(
        ITransfer.Transfer memory transfer
    ) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    transfer.recipient,
                    transfer.amount,
                    transfer.assetId,
                    transfer.nonce
                )
            );
    }
}

// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {ITransfer} from "../common-interface/ITransfer.sol";
import {TransferLib} from "../utils/TransferLib.sol";

contract TestTransferLib {
    using TransferLib for ITransfer.Transfer;

    function transferCommitment(
        ITransfer.Transfer memory transfer
    ) external pure returns (bytes32) {
        return transfer.transferCommitment();
    }
}

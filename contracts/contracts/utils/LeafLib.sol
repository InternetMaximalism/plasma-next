// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {ILeaf} from "../common-interface/ILeaf.sol";

library LeafLib {
    function hashLeaf(
        ILeaf.WithdrawLeaf memory leaf
    ) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    leaf.recipient,
                    leaf.amount.amounts,
                    leaf.startEbn,
                    leaf.endEbn
                )
            );
    }

    function hashLeaf(
        ILeaf.EvidenceLeaf memory leaf
    ) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(leaf.transferCommitment, leaf.ebn));
    }
}

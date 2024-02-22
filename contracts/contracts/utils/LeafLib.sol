// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {ILeaf} from "../common-interface/ILeaf.sol";

library LeafLib {
    function hashLeaf(
        ILeaf.SettlementLeaf memory leaf
    ) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    leaf.withdrawLeaf.recipient,
                    leaf.withdrawLeaf.amount.amounts,
                    leaf.withdrawLeaf.startEbn,
                    leaf.withdrawLeaf.endEbn,
                    leaf.evidenceLeaf.transferCommitment,
                    leaf.evidenceLeaf.ebn
                )
            );
    }
}

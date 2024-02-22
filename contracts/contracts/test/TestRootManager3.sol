// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IMerkleProof} from "../common-interface/IMerkleProof.sol";

contract TestRootManager3 {
    function verifySettlementMerkleProof(
        IMerkleProof.SettlementMerkleProof memory settlement
    ) external view {}
}

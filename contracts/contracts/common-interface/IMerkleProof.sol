/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {ILeaf} from "./ILeaf.sol";

interface IMerkleProof {
    struct SettlementMerkleProof {
        ILeaf.SettlementLeaf leaf;
        uint256 index;
        bytes32[] siblings;
    }
}

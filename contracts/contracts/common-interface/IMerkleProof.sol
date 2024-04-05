/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {ILeaf} from "./ILeaf.sol";

interface IMerkleProof {
    struct EvidenceWithMerkleProof {
        ILeaf.EvidenceLeaf leaf;
        uint256 index;
        bytes32[] siblings;
    }

    struct WithdrawWithMerkleProof {
        ILeaf.WithdrawLeaf leaf;
        uint256 index;
        bytes32[] siblings;
    }
}

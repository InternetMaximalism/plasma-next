// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {MerkleProofLib} from "../utils/MerkleProofLib.sol";

contract TestMerkleProofLib {
    using MerkleProofLib for bytes32;

    function getRootFromMerkleProof(
        bytes32 leafHash,
        uint256 index,
        bytes32[] memory siblings
    ) external pure returns (bytes32) {
        return leafHash.getRootFromMerkleProof(index, siblings);
    }
}

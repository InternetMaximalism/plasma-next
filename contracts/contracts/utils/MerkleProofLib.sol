/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

library MerkleProofLib {
    function getRootFromMerkleProof(
        bytes32 leafHash,
        uint256 index,
        bytes32[] memory siblings
    ) internal pure returns (bytes32) {
        bytes32 computedHash = leafHash;
        for (uint256 i = 0; i < siblings.length; i++) {
            bytes32 sibling = siblings[i];
            if (index % 2 == 0) {
                computedHash = keccak256(
                    abi.encodePacked(computedHash, sibling)
                );
            } else {
                computedHash = keccak256(
                    abi.encodePacked(sibling, computedHash)
                );
            }
            index = index >> 1;
        }
        return computedHash;
    }
}

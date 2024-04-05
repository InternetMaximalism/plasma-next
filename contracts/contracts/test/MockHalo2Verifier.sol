// SPDX-License-Identifier: MIT
// solhint-disable no-unused-vars
// solhint-disable no-inline-assembly
pragma solidity 0.8.23;

contract MockHalo2Verifier {
    function verifyProof(
        address vk,
        bytes calldata proof,
        uint256[] calldata instances
    ) public pure returns (bool) {
        (vk, proof, instances);
        return true;
    }
}

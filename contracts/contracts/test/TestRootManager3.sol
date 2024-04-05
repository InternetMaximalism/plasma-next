// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IMerkleProof} from "../common-interface/IMerkleProof.sol";

contract TestRootManager3 {
    function verifyWithdrawMerkleProof(
        IMerkleProof.WithdrawWithMerkleProof memory withdrawProof
    ) external view {}

    function verifyEvidenceMerkleProof(
        IMerkleProof.EvidenceWithMerkleProof memory evidence
    ) external view {}
}

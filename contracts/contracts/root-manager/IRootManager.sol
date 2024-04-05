// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IVerifier} from "../verifier/IVerifier.sol";
import {IMerkleProof} from "../common-interface/IMerkleProof.sol";
import {IPublicInputs} from "../common-interface/IPublicInputs.sol";

interface IRootManager {
    error InvalidEvidenceMerkleProof(bytes32 root);
    error InvalidWithdrawMerkleProof(bytes32 root);
    error BlockHashMismatch(bytes32 pisBlockHash, bytes32 blockHash);
    error ProofVerificationFailed();

    event BlockHashPosted(bytes32 indexed blockHash);
    event RootPosted(
        bytes32 indexed withdrawRoot,
        bytes32 indexed evidenceRoot
    );

    function config(
        address verifierAddress_,
        address blockManagerAddress_
    ) external;

    function postRoot(
        uint32 blockNumber,
        bytes32[] memory transferRoots,
        bytes32[] memory totalDepositHashes,
        IPublicInputs.PublicInputs memory pis,
        bytes memory proof
    ) external;

    function verifyWithdrawMerkleProof(
        IMerkleProof.WithdrawWithMerkleProof memory withdraw
    ) external view;

    function verifyEvidenceMerkleProof(
        IMerkleProof.EvidenceWithMerkleProof memory evidence
    ) external view;
}

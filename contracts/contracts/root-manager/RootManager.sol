// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {IVerifier} from "../verifier/IVerifier.sol";
import {IBlockManager} from "../block-manager/IBlockManager.sol";
import {IRootManager} from "../root-manager/IRootManager.sol";
import {Verifier} from "../verifier/Verifier.sol";
import {MerkleProofLib} from "../utils/MerkleProofLib.sol";
import {LeafLib} from "../utils/LeafLib.sol";

import {ILeaf} from "../common-interface/ILeaf.sol";
import {IMerkleProof} from "../common-interface/IMerkleProof.sol";
import {IPublicInputs} from "../common-interface/IPublicInputs.sol";

/**
 * @title RootManager
 * @author Intmax
 * @notice This contract is responsible for managing the settlement root.
 * It verifies the ZKP proof and stores the settlement root.
 */
contract RootManager is
    AccessControlUpgradeable,
    UUPSUpgradeable,
    IRootManager
{
    using LeafLib for ILeaf.WithdrawLeaf;
    using LeafLib for ILeaf.EvidenceLeaf;

    /// @notice Mapping that stores the existence of the withdraw root.
    mapping(bytes32 => bool) public doesWithdrawRootExist;
    /// @notice Mapping that stores the existence of the evidence root.
    mapping(bytes32 => bool) public doesEvidenceRootExist;

    /// @notice The address of the block manager contract.
    address public blockManagerAddress;
    /// @notice The address of the verifier contract.
    address public verifierAddress;

    /// @dev Initialization of thi contract at the time of deployment or upgrade
    function initialize(address admin) public initializer {
        __AccessControl_init();
        __UUPSUpgradeable_init();
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
    }

    /// @dev Config contract addresses called by the Config contract
    function config(
        address verifierAddress_,
        address blockManagerAddress_
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        verifierAddress = verifierAddress_;
        blockManagerAddress = blockManagerAddress_;
    }

    /**
     * verify the ZKP proof and store the settlement root.
     * @param pis public inputs
     * @param proof proof
     */
    function postRoot(
        uint32 blockNumber,
        bytes32[] memory transferRoots,
        bytes32[] memory totalDepositHashes,
        IPublicInputs.PublicInputs memory pis,
        bytes memory proof
    ) external {
        IBlockManager(blockManagerAddress).verifyInclusion(
            blockNumber,
            pis.blockHash,
            transferRoots,
            totalDepositHashes
        );
        try IVerifier(verifierAddress).verifyProof(pis, proof) returns (
            bool verified
        ) {
            if (!verified) {
                revert ProofVerificationFailed();
            }
        } catch {
            revert ProofVerificationFailed();
        }
        doesWithdrawRootExist[pis.withdrawRoot] = true;
        doesEvidenceRootExist[pis.evidenceRoot] = true;
        emit RootPosted(pis.withdrawRoot, pis.evidenceRoot);
    }

    /**
     * @notice Verify the withdraw with the merkle proof.
     * @param withdraw withdraw
     */
    function verifyWithdrawMerkleProof(
        IMerkleProof.WithdrawWithMerkleProof memory withdraw
    ) external view {
        bytes32 root = MerkleProofLib.getRootFromMerkleProof(
            withdraw.leaf.hashLeaf(),
            withdraw.index,
            withdraw.siblings
        );
        if (!doesWithdrawRootExist[root]) {
            revert InvalidWithdrawMerkleProof(root);
        }
    }

    /**
     * @notice Verify the evidence with the merkle proof.
     * @param evidence evidence
     */
    function verifyEvidenceMerkleProof(
        IMerkleProof.EvidenceWithMerkleProof memory evidence
    ) external view {
        bytes32 root = MerkleProofLib.getRootFromMerkleProof(
            evidence.leaf.hashLeaf(),
            evidence.index,
            evidence.siblings
        );
        if (!doesEvidenceRootExist[root]) {
            revert InvalidEvidenceMerkleProof(root);
        }
    }

    /// @dev Authorize the upgrade
    function _authorizeUpgrade(
        address
    ) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}
}

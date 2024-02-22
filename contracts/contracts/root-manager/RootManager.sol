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
 * @notice This contract is responsible for managing the settlement root.
 * It verifies the ZKP proof and stores the settlement root.
 */
contract RootManager is
    AccessControlUpgradeable,
    UUPSUpgradeable,
    IRootManager
{
    using LeafLib for ILeaf.SettlementLeaf;
    bytes32 public constant OPERATOR = keccak256("OPERATOR");

    /// @notice Mapping that stores the existence of the settlement root.
    mapping(bytes32 => bool) public doesSettlementRootExist;

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
        address operator_,
        address verifierAddress_,
        address blockManagerAddress_
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        _grantRole(OPERATOR, operator_);
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
        IPublicInputs.PublicInputs memory pis,
        bytes memory proof
    ) external {
        bytes32 blockHash = IBlockManager(blockManagerAddress).getBlockHash(
            blockNumber
        );
        if (blockHash != pis.blockHash) {
            revert BlockHashMismatch(pis.blockHash, blockHash);
        }
        try IVerifier(verifierAddress).verifyProof(pis, proof) returns (
            bool verified
        ) {
            if (!verified) {
                revert ProofVerificationFailed();
            }
        } catch {
            revert ProofVerificationFailed();
        }
        doesSettlementRootExist[pis.settlementRoot] = true;
        emit RootPosted(pis.settlementRoot);
    }

    /**
     * @notice Verify the settlement merkle proof.
     * @param settlement The settlement merkle proof.
     */
    function verifySettlementMerkleProof(
        IMerkleProof.SettlementMerkleProof memory settlement
    ) external view {
        bytes32 root = MerkleProofLib.getRootFromMerkleProof(
            settlement.leaf.hashLeaf(),
            settlement.index,
            settlement.siblings
        );
        if (!doesSettlementRootExist[root]) {
            revert InvalidWithdrawMerkleProof(root);
        }
    }

    /// @dev Authorize the upgrade
    function _authorizeUpgrade(
        address
    ) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}
}

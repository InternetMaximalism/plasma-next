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

contract TestRootManager is
    AccessControlUpgradeable,
    UUPSUpgradeable,
    IRootManager
{
    using LeafLib for ILeaf.SettlementLeaf;
    bytes32 public constant OPERATOR = keccak256("OPERATOR");

    mapping(bytes32 => bool) public doesSettlementRootExist;
    mapping(bytes32 => bool) public doesBlockHashExist;

    address public blockManagerAddress;
    address public verifierAddress;

    function initialize(address admin) public initializer {
        __AccessControl_init();
        __UUPSUpgradeable_init();
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
    }

    function config(
        address operator_,
        address verifierAddress_,
        address blockManagerAddress_
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        _grantRole(OPERATOR, operator_);
        verifierAddress = verifierAddress_;
        blockManagerAddress = blockManagerAddress_;
    }

    // verify the proof and store the root
    // bypass the proof verification for testing
    function postRoot(
        uint32 blockNumber,
        IPublicInputs.PublicInputs memory pis,
        bytes memory proof
    ) external {
        (blockNumber, proof);
        doesSettlementRootExist[pis.settlementRoot] = true;
        emit RootPosted(pis.settlementRoot);
    }

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

    function _authorizeUpgrade(
        address
    ) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}
}

/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IZKPTLC} from "../common-interface/IZKPTLC.sol";
import {IAsset} from "../common-interface/IAsset.sol";
import {ITransfer} from "../common-interface/ITransfer.sol";
import {TransferLib} from "../utils/TransferLib.sol";
import {IRootManager} from "../root-manager/IRootManager.sol";
import {ITransfer} from "../common-interface/ITransfer.sol";
import {IMerkleProof} from "../common-interface/IMerkleProof.sol";
import {TransferLib} from "../utils/TransferLib.sol";

contract DefaultZKPTLC is IZKPTLC {
    using TransferLib for ITransfer.Transfer;

    address public rootManagerAddress;

    constructor(address _rootManagerAddress) {
        rootManagerAddress = _rootManagerAddress;
    }

    function computeInstance(
        ITransfer.Transfer memory transfer
    ) external pure returns (bytes32) {
        return transfer.transferCommitment();
    }

    function _verifyExistence(
        ITransfer.Transfer memory transfer,
        IMerkleProof.EvidenceWithMerkleProof memory proof
    ) internal view {
        if (transfer.transferCommitment() != proof.leaf.transferCommitment) {
            revert("Transfer commitment does not match");
        }
        IRootManager(rootManagerAddress).verifyEvidenceMerkleProof(proof);
    }

    struct Witness {
        ITransfer.Transfer transfer;
        IMerkleProof.EvidenceWithMerkleProof proof;
    }

    function encodeWitness(
        Witness memory witness
    ) external pure returns (bytes memory) {
        return abi.encode(witness);
    }

    function verifyCondition(
        bytes32 instance,
        bytes memory witness
    ) external view returns (IAsset.AssetsDelta memory toOperatorDelta) {
        Witness memory w = abi.decode(witness, (Witness));
        if (instance != w.transfer.transferCommitment()) {
            revert("Invalid instance");
        }
        _verifyExistence(w.transfer, w.proof);
        IAsset.AssetsDelta memory zero;
        return zero;
    }
}

/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IZKPTLC} from "../common-interface/IZKPTLC.sol";
import {IAsset} from "../common-interface/IAsset.sol";
import {IRootManager} from "../root-manager/IRootManager.sol";
import {ITransfer} from "../common-interface/ITransfer.sol";
import {IMerkleProof} from "../common-interface/IMerkleProof.sol";
import {TransferLib} from "../utils/TransferLib.sol";

contract TwoWaySwap is IZKPTLC {
    using TransferLib for ITransfer.Transfer;

    address public rootManagerAddress;

    constructor(address _rootManagerAddress) {
        rootManagerAddress = _rootManagerAddress;
    }

    function computeInstance(
        ITransfer.Transfer memory transfer1,
        ITransfer.Transfer memory transfer2
    ) public pure returns (bytes32) {
        bytes32 tc1 = transfer1.transferCommitment();
        bytes32 tc2 = transfer2.transferCommitment();
        return keccak256(abi.encodePacked(tc1, tc2));
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
        ITransfer.Transfer transfer1;
        ITransfer.Transfer transfer2;
        IMerkleProof.EvidenceWithMerkleProof proof1;
        IMerkleProof.EvidenceWithMerkleProof proof2;
    }

    function verifyCondition(
        bytes32 instance,
        bytes memory witness
    ) external view returns (IAsset.AssetsDelta memory toOperatorDelta) {
        Witness memory w = abi.decode(witness, (Witness));
        bytes32 expectedInstance = computeInstance(w.transfer1, w.transfer2);
        if (instance != expectedInstance) {
            revert("Invalid instance");
        }
        _verifyExistence(w.transfer1, w.proof1);
        _verifyExistence(w.transfer2, w.proof2);
        return toOperatorDelta; // return zero
    }
}

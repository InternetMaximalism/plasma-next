/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IZKPTLC} from "../common-interface/IZKPTLC.sol";
import {ITransfer} from "../common-interface/ITransfer.sol";
import {IMerkleProof} from "../common-interface/IMerkleProof.sol";
import {IAsset} from "../common-interface/IAsset.sol";
import {IRootManager} from "../root-manager/IRootManager.sol";
import {TransferLib} from "../utils/TransferLib.sol";
import {UD60x18, ud, convert} from "@prb/math/src/UD60x18.sol";

contract SpotZKPTLC is IZKPTLC {
    using TransferLib for ITransfer.Transfer;

    address public rootManagerAddress;

    constructor(address _rootManagerAddress) {
        rootManagerAddress = _rootManagerAddress;
    }

    struct Order {
        address user;
        uint32 sellAssetId;
        uint32 buyAssetId;
        uint256 maxSellAmount; // The maximum amount of the sell asset that can be sold
        UD60x18 maxPrice; // The maximum price of the buy asset in terms of the sell asset
        uint32 airdropNonce;
        uint256 deadline;
    }

    struct OrderWitness {
        Order order;
        UD60x18 fillPrice; // The price at which the order was filled (in terms of the sell asset)
        uint256 fillAmount; // The amount of the sell asset that was sold.
        bool useAirdrop; // if this value is false, the following element will be ignored.
        IMerkleProof.EvidenceWithMerkleProof evidence; // The evidence of the filling airdrop.
        ITransfer.Transfer transfer; // The transfer of the filling airdrop.
    }

    function computeOrderInstance(
        Order memory order
    ) public pure returns (bytes32 instance) {
        return keccak256(abi.encode(order));
    }

    function verifyCondition(
        bytes32 instance,
        bytes memory witness
    ) external view returns (IAsset.AssetsDelta memory toOperator) {
        OrderWitness memory orderWitness = abi.decode(witness, (OrderWitness));
        Order memory order = orderWitness.order;

        // instance check
        if (instance != computeOrderInstance(orderWitness.order)) {
            revert("Invalid instance");
        }

        // deadline check
        if (block.timestamp > order.deadline) {
            revert("Order expired");
        }

        // fill price check
        if (orderWitness.fillPrice > order.maxPrice) {
            revert("Fill price too high");
        }

        // fill amount check
        if (orderWitness.fillAmount > order.maxSellAmount) {
            revert("Fill amount too high");
        }

        uint256 buyAmount = convert(
            ud(orderWitness.fillAmount) * orderWitness.fillPrice
        );

        // airdrop verification
        if (orderWitness.useAirdrop) {
            IRootManager(rootManagerAddress).verifyEvidenceMerkleProof(
                orderWitness.evidence
            );
            if (
                orderWitness.transfer.transferCommitment() !=
                orderWitness.evidence.leaf.transferCommitment
            ) {
                revert("Invalid transfer commitment");
            }
            if (orderWitness.transfer.recipient != orderWitness.order.user) {
                revert("Invalid recipient");
            }
            if (
                orderWitness.transfer.nonce != orderWitness.order.airdropNonce
            ) {
                revert("Invalid nonce");
            }
            if (
                orderWitness.transfer.assetId != orderWitness.order.buyAssetId
            ) {
                revert("Invalid assetId");
            }
            if (orderWitness.transfer.amount < buyAmount) {
                revert("Invalid transfer amount");
            }
        }
        // compute the `toOperator` amount
        int256 toOperatorSellAsset = int256(orderWitness.fillAmount);
        int256 toOperatorBuyAsset;
        if (orderWitness.useAirdrop) {
            toOperatorBuyAsset = 0; // the buy asset is sent to the user via the airdrop
        } else {
            toOperatorBuyAsset = -int256(buyAmount); // send the buy asset to the user via the channel
        }
        toOperator.amounts[order.sellAssetId] = toOperatorSellAsset;
        toOperator.amounts[order.buyAssetId] = toOperatorBuyAsset;
        return toOperator;
    }
}

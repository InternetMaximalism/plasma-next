/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IAsset} from "../../common-interface/IAsset.sol";
import {IPayment} from "../../common-interface/IPayment.sol";
import {IMerkleProof} from "../../common-interface/IMerkleProof.sol";
import {ITransfer} from "../../common-interface/ITransfer.sol";

interface IWithdraw {
    // error types for `withdrawRequest`
    error WithdrawalRequestAlreadyExists(address user);
    error WithdrawalRequestNotFound(address user);
    error InvalidUser(address user, address withdrawLeafRecipient);
    error LeafStartEbnIsTooOld(uint64 leafStartEbn, uint64 channelEbn);
    error EbnSanityCheckFailed(uint64 startEbn, uint64 endEbn);
    // error types for `verifySettlementMerkleProof`
    error TransferCommitmentMismatch(
        bytes32 leafTransferCommitment,
        bytes32 computedTransferCommitment
    );
    error EvidenceLeafEbnIsTooOld(uint64 leafEbn, uint64 channelEbn);
    error TimeOutIsNotReached(uint256 timeout, uint256 currentTime);

    event WithdrawalRequested(address indexed user);
    event WithdrawalChallenged(address indexed user);
    event WithdrawalAccepted(address indexed user);
    event WithdrawalTimeOuted(address indexed user);

    function config(
        address operator_,
        address mainAddress_,
        address rootManagerAddress_,
        address liquidityManagerAddress_
    ) external;

    function requestWithdrawal(
        IMerkleProof.SettlementMerkleProof memory settlementProof,
        IAsset.Assets memory redeposit
    ) external;

    function requestWithdrawalWithEvidence(
        ITransfer.Transfer memory transfer,
        IMerkleProof.SettlementMerkleProof memory settlementProof,
        IAsset.Assets memory redeposit
    ) external;

    function challengeWithdrawal(
        address user,
        IPayment.PaymentWithSignature memory paymentWithSignature,
        IMerkleProof.SettlementMerkleProof memory settlementProof
    ) external;

    function acceptWithdrawal(address user) external;

    function timeOutWithdrawal(address user) external;
}

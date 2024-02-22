/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IAsset} from "../../common-interface/IAsset.sol";
import {IState} from "../../common-interface/IState.sol";
import {IPayment} from "../../common-interface/IPayment.sol";
import {IMerkleProof} from "../../common-interface/IMerkleProof.sol";

interface IMain {
    error InvalidCustomDataLength(bytes customData);
    error AdditionalZKPTLCVerificationFailed(address additionalZKPTLCAddress);
    error RoundMismatch(uint32 channelRound, uint32 paymentRound);
    error RecipientMismatch(address leafRecipient, address user);
    error EbnSanityCheckFailed(uint64 startEbn, uint64 endEbn);
    error LatestEbnMismatch(uint64 leafEbn, uint64 channelEbn);
    error LeafStartEbnIsTooOld(uint64 leafStartEbn, uint64 channelEbn);
    error AirdroppedAmountMismatch(
        IAsset.Assets leafAirdroppedAmount,
        IAsset.Assets paymentAirdroppedAmount
    );
    error TransferCommitmentMismatch(
        bytes32 leafTransferCommitment,
        bytes32 paymentTransferCommitment
    );
    error SpentMoreThanDeposit(
        IAsset.Assets spentDepositInPayment,
        IAsset.Assets deposit
    );
    error InvariantViolation(
        IAsset.Assets totalIncome,
        IAsset.Assets totalOutcome
    );

    event Deposited(address indexed user, IAsset.Assets assets);
    event ChannelClosed(address indexed user, uint32 round);

    function config(
        address operator_,
        address withdrawAddress_,
        address rootManagerAddress_,
        address liquidityManagerAddress_
    ) external;

    function getUniqueIdentifier() external view returns (bytes32);

    function deposit(IAsset.Assets memory assets) external;

    function closeChannel(
        IPayment.PaymentWithSignature memory paymentWithSignature,
        IMerkleProof.SettlementMerkleProof memory settlementProof,
        IAsset.Assets memory redeposit
    ) external;

    function closeChannelAsChallenge(
        IPayment.PaymentWithSignature memory paymentWithSignature,
        IMerkleProof.SettlementMerkleProof memory settlementProof
    ) external;

    function getChannelState(
        address user
    ) external view returns (IState.ChannelState memory);

    function setChannelState(
        address user,
        IState.ChannelState memory channelState
    ) external;
}

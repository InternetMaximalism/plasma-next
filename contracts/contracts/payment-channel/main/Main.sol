/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

import {IMain} from "./IMain.sol";
import {IRootManager} from "../../root-manager/IRootManager.sol";
import {ILiquidityManager} from "../../liquidity-manager/ILiquidityManager.sol";

import {IAsset} from "../../common-interface/IAsset.sol";
import {IState} from "../../common-interface/IState.sol";
import {ILeaf} from "../../common-interface/ILeaf.sol";
import {IPayment} from "../../common-interface/IPayment.sol";
import {IMerkleProof} from "../../common-interface/IMerkleProof.sol";
import {IAdditionalZKPTLC} from "../../common-interface/IAdditionalZKPTLC.sol";

import {SignatureLib} from "../../utils/SignatureLib.sol";
import {AssetLib} from "../../utils/AssetLib.sol";

/**
 * @title Main contract
 * @notice This contract is the main contract of the payment channel.
 * It manages the channel state and the settlement of the payment.
 */
contract Main is AccessControlUpgradeable, UUPSUpgradeable, IMain {
    using AssetLib for IAsset.Assets;
    using SignatureLib for IPayment.PaymentWithSignature;

    /// @notice Operator role constant
    bytes32 public constant OPERATOR = keccak256("OPERATOR");
    /// @notice The role that can change the channel state.
    bytes32 public constant INNER_GROUP = keccak256("INNER_GROUP");

    /// @notice The address of the operator
    address public operator;
    /// @notice The address of the root manager
    address public rootManagerAddress;
    /// @notice The address of the liquidity manager
    address public liquidityManagerAddress;

    /// @notice The channel states of each user
    mapping(address => IState.ChannelState) public channelStates;

    /// @dev Initialization of thi contract at the time of deployment or upgrade
    function initialize(address admin) public initializer {
        __AccessControl_init();
        __UUPSUpgradeable_init();
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
    }

    /// @dev Config contract addresses called by the Config contract
    function config(
        address operator_,
        address withdrawAddress_,
        address rootManagerAddress_,
        address liquidityManagerAddress_
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        operator = operator_;
        rootManagerAddress = rootManagerAddress_;
        liquidityManagerAddress = liquidityManagerAddress_;
        _grantRole(OPERATOR, operator_);
        _grantRole(INNER_GROUP, withdrawAddress_);
    }

    /// @notice uniqueIdentifier is used to prevent cross-chain replay attack.
    function getUniqueIdentifier() external view returns (bytes32) {
        return SignatureLib.getUniqueIdentifier();
    }

    /**
     * @dev Verify the custom data
     * @param customData The first 20 bytes are the address of AdditionalZKPTLC,
     * and the latter is the data passed to verifyAdditionalZKPTLC of AdditionalZKPTLC.
     */
    function _verifyCustomData(bytes memory customData) internal view {
        if (customData.length > 0) {
            if (customData.length <= 20) {
                revert InvalidCustomDataLength(customData);
            }
            // Extract additional ZKPTLC address from customData
            address additionalZKPTLCAddress;
            // solhint-disable-next-line no-inline-assembly
            assembly {
                additionalZKPTLCAddress := mload(add(customData, 20))
            }
            bytes memory remainingData = new bytes(customData.length - 20);
            for (uint i = 20; i < customData.length; i++) {
                remainingData[i - 20] = customData[i];
            }
            if (
                !IAdditionalZKPTLC(additionalZKPTLCAddress)
                    .verifyAdditionalZKPTLC(remainingData)
            ) {
                revert AdditionalZKPTLCVerificationFailed(
                    additionalZKPTLCAddress
                );
            }
        }
    }

    /// @dev Verify the payment and the settlement merkle proof
    /// @param user The user address
    /// @param paymentWithSignature The payment with the user and operator signatures
    /// @param settlementProof The settlement merkle proof
    function _verifyPayment(
        address user,
        IPayment.PaymentWithSignature memory paymentWithSignature,
        IMerkleProof.SettlementMerkleProof memory settlementProof
    ) private view {
        paymentWithSignature.verifyPaymentSignature(operator, user);
        IRootManager(rootManagerAddress).verifySettlementMerkleProof(
            settlementProof
        );
        _verifyCustomData(paymentWithSignature.payment.customData);
        // verify consistency of the payment
        IState.ChannelState memory channelState = channelStates[user];
        IPayment.Payment memory payment = paymentWithSignature.payment;
        ILeaf.WithdrawLeaf memory withdrawLeaf = settlementProof
            .leaf
            .withdrawLeaf;
        ILeaf.EvidenceLeaf memory evidenceLeaf = settlementProof
            .leaf
            .evidenceLeaf;

        if (withdrawLeaf.recipient != user) {
            revert RecipientMismatch({
                leafRecipient: withdrawLeaf.recipient,
                user: user
            });
        }
        if (channelState.round != payment.round) {
            revert RoundMismatch({
                channelRound: channelState.round,
                paymentRound: payment.round
            });
        }
        if (withdrawLeaf.endEbn < withdrawLeaf.startEbn) {
            revert EbnSanityCheckFailed({
                startEbn: withdrawLeaf.startEbn,
                endEbn: withdrawLeaf.endEbn
            });
        }
        if (withdrawLeaf.endEbn != payment.latestEbn) {
            revert LatestEbnMismatch({
                leafEbn: withdrawLeaf.endEbn,
                channelEbn: payment.latestEbn
            });
        }
        if (withdrawLeaf.startEbn <= channelState.ebn) {
            revert LeafStartEbnIsTooOld({
                leafStartEbn: withdrawLeaf.startEbn,
                channelEbn: channelState.ebn
            });
        }
        if (!withdrawLeaf.amount.isEq(payment.airdropped)) {
            revert AirdroppedAmountMismatch({
                leafAirdroppedAmount: withdrawLeaf.amount,
                paymentAirdroppedAmount: payment.airdropped
            });
        }
        // ignore the lastest transfer commitment if it is 0x0
        if (payment.latestTransferCommitment != 0x0) {
            if (
                evidenceLeaf.transferCommitment !=
                payment.latestTransferCommitment
            ) {
                revert TransferCommitmentMismatch({
                    leafTransferCommitment: evidenceLeaf.transferCommitment,
                    paymentTransferCommitment: payment.latestTransferCommitment
                });
            }
        }
        // verify that deposit amount
        if (!payment.spentDeposit.isLe(channelState.userDeposit)) {
            revert SpentMoreThanDeposit({
                spentDepositInPayment: payment.spentDeposit,
                deposit: channelState.userDeposit
            });
        }
        // verify the total income and outcome
        IAsset.Assets memory totalIncome = payment.airdropped.add(
            payment.spentDeposit
        );
        IAsset.Assets memory totalOutcome = payment.userBalance.add(
            payment.operatorBalance
        );
        if (!totalIncome.isEq(totalOutcome)) {
            revert InvariantViolation({
                totalIncome: totalIncome,
                totalOutcome: totalOutcome
            });
        }
    }

    /// @dev Settle the payment, update the channel state, and send the assets
    function _settle(
        IPayment.Payment memory payment,
        IAsset.Assets memory redeposit
    ) private {
        address user = payment.user;
        IState.ChannelState memory channelState = channelStates[user];
        IAsset.Assets memory remainedDeposit = channelState.userDeposit.sub(
            payment.spentDeposit
        );
        IAsset.Assets memory totalUserBalance = payment.userBalance.add(
            remainedDeposit
        );
        if (totalUserBalance.isLe(redeposit)) {
            redeposit = totalUserBalance; // avoid underflow
        }
        IAsset.Assets memory userWithdrawal = totalUserBalance.sub(redeposit);
        channelStates[user] = IState.ChannelState({
            userDeposit: redeposit,
            ebn: payment.latestEbn,
            round: channelState.round + 1
        });
        ILiquidityManager(liquidityManagerAddress).sendAssets(
            operator,
            payment.operatorBalance
        );
        ILiquidityManager(liquidityManagerAddress).sendAssets(
            payment.user,
            userWithdrawal
        );
    }

    function deposit(IAsset.Assets memory assets) external {
        address user = _msgSender();
        ILiquidityManager(liquidityManagerAddress).receiveAssets(user, assets);
        channelStates[user].userDeposit = channelStates[user].userDeposit.add(
            assets
        );
        emit Deposited(user, assets);
    }

    /**
     * @notice cClose channel and settle the payment.
     * @dev This function is called by the operator. if the user wants to close the channel by themself,
     * the user should call `withdraw` function.
     * @param paymentWithSignature The payment with the user and operator signatures
     * @param settlementProof The settlement merkle proof
     * @param redeposit The amount of the deposit that the user wants to redeposit
     */
    function closeChannel(
        IPayment.PaymentWithSignature memory paymentWithSignature,
        IMerkleProof.SettlementMerkleProof memory settlementProof,
        IAsset.Assets memory redeposit
    ) external onlyRole(OPERATOR) {
        address user = paymentWithSignature.payment.user;
        _verifyPayment(user, paymentWithSignature, settlementProof);
        _settle(paymentWithSignature.payment, redeposit);
        emit ChannelClosed(user, paymentWithSignature.payment.round);
    }

    function closeChannelAsChallenge(
        IPayment.PaymentWithSignature memory paymentWithSignature,
        IMerkleProof.SettlementMerkleProof memory settlementProof
    ) external onlyRole(INNER_GROUP) {
        address user = paymentWithSignature.payment.user;
        _verifyPayment(user, paymentWithSignature, settlementProof);
        IAsset.Assets memory zeroAssets;
        _settle(paymentWithSignature.payment, zeroAssets);
        emit ChannelClosed(user, paymentWithSignature.payment.round);
    }

    /// @notice Get the channel state of the user.
    function getChannelState(
        address user
    ) external view returns (IState.ChannelState memory) {
        return channelStates[user];
    }

    /// @notice Set the channel state of the user.
    function setChannelState(
        address user,
        IState.ChannelState memory channelState
    ) external onlyRole(INNER_GROUP) {
        channelStates[user] = channelState;
    }

    /// @dev Authorize the upgrade
    function _authorizeUpgrade(
        address
    ) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}
}

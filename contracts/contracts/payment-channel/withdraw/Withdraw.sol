/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

import {IWithdraw} from "./IWithdraw.sol";
import {IMain} from "../main/IMain.sol";
import {IRootManager} from "../../root-manager/IRootManager.sol";
import {ILiquidityManager} from "../../liquidity-manager/ILiquidityManager.sol";

import {IMerkleProof} from "../../common-interface/IMerkleProof.sol";
import {IState} from "../../common-interface/IState.sol";
import {ITransfer} from "../../common-interface/ITransfer.sol";
import {IAsset} from "../../common-interface/IAsset.sol";
import {ILeaf} from "../../common-interface/ILeaf.sol";
import {IPayment} from "../../common-interface/IPayment.sol";

import {AssetLib} from "../../utils/AssetLib.sol";
import {TransferLib} from "../../utils/TransferLib.sol";

/**
 * @title Withdraw contract
 * @notice This contract supports user-initiated withdrawals without the need for a payment channel.
 * If the operator is cooperative, there is no need to use this contract.
 */
contract Withdraw is AccessControlUpgradeable, UUPSUpgradeable, IWithdraw {
    using TransferLib for ITransfer.Transfer;
    using AssetLib for IAsset.Assets;

    /// @notice Operator role constant
    bytes32 public constant OPERATOR = keccak256("OPERATOR");
    /// @notice Minimum waiting time required for withdrawal
    uint256 public constant WITHDRAW_TIMEOUT = 3 days;

    /// @notice The contract address of main payment channel
    address public mainAddress;
    /// @notice The contract address of root manager
    address public rootManagerAddress;
    /// @notice The contract address of liquidity manager
    address public liquidityManagerAddress;
    /// @notice Withdrawal requests issued by each account
    mapping(address => IState.WithdrawalRequest) public withdrawalRequests;

    /// @notice Ensure that the withdraw request has not been made.
    modifier onlyBeforeRequest() {
        if (withdrawalRequests[_msgSender()].requestedAt != 0) {
            revert WithdrawalRequestAlreadyExists(_msgSender());
        }
        _;
    }

    /// @notice Ensure that the withdraw request has been made.
    modifier onlyAfterRequest(address user) {
        if (withdrawalRequests[user].requestedAt == 0) {
            revert WithdrawalRequestNotFound(user);
        }
        _;
    }

    /// @dev Initialization of thi contract at the time of deployment or upgrade
    function initialize(address admin) public initializer {
        __AccessControl_init();
        __UUPSUpgradeable_init();
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
    }

    /// @dev Config contract addresses called by the Config contract
    function config(
        address operator_,
        address mainAddress_,
        address rootManagerAddress_,
        address liquidityManagerAddress_
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        _grantRole(OPERATOR, operator_);
        mainAddress = mainAddress_;
        rootManagerAddress = rootManagerAddress_;
        liquidityManagerAddress = liquidityManagerAddress_;
    }

    /**
     * @notice User issues a withdrawal request with a settlement proof of the total amount of
     * multiple airdrops. If the `WITHDRAW_TIMEOUT` has passed without being challenged by the operator,
     * the funds can be finalized through a `timeOutWithdrawal`.
     * @param settlementProof The settlement proof that proves the total amount of airdrops.
     * @param redeposit The amount of assets to be redeposited
     */
    function requestWithdrawal(
        IMerkleProof.SettlementMerkleProof memory settlementProof,
        IAsset.Assets memory redeposit
    ) external onlyBeforeRequest {
        address user = _msgSender();
        IRootManager(rootManagerAddress).verifySettlementMerkleProof(
            settlementProof
        );
        ILeaf.WithdrawLeaf memory withdrawLeaf = settlementProof
            .leaf
            .withdrawLeaf;
        IState.ChannelState memory channelState = IMain(mainAddress)
            .getChannelState(user);
        if (withdrawLeaf.recipient != user) {
            revert InvalidUser(user, withdrawLeaf.recipient);
        }
        if (withdrawLeaf.endEbn < withdrawLeaf.startEbn) {
            revert EbnSanityCheckFailed({
                startEbn: withdrawLeaf.startEbn,
                endEbn: withdrawLeaf.endEbn
            });
        }
        if (withdrawLeaf.startEbn < channelState.ebn) {
            revert LeafStartEbnIsTooOld(
                withdrawLeaf.startEbn,
                channelState.ebn
            );
        }
        withdrawalRequests[user] = IState.WithdrawalRequest({
            // solhint-disable-next-line not-rely-on-time
            requestedAt: block.timestamp,
            airdropped: withdrawLeaf.amount,
            redeposit: redeposit,
            newEbn: withdrawLeaf.endEbn
        });
        emit WithdrawalRequested(user);
    }

    /**
     * @notice User issues a withdrawal request with a settlement proof of the amount of
     * an airdrop. If the `WITHDRAW_TIMEOUT` has passed without being challenged by the operator,
     * the funds can be finalized through a `timeOutWithdrawal`.
     * @param transfer The transfer
     * @param settlementProof The settlement proof of the channel state
     * @param redeposit The amount of assets to be redeposited
     */
    function requestWithdrawalWithEvidence(
        ITransfer.Transfer memory transfer,
        IMerkleProof.SettlementMerkleProof memory settlementProof,
        IAsset.Assets memory redeposit
    ) external onlyBeforeRequest {
        address user = _msgSender();
        IRootManager(rootManagerAddress).verifySettlementMerkleProof(
            settlementProof
        );
        ILeaf.EvidenceLeaf memory evidenceLeaf = settlementProof
            .leaf
            .evidenceLeaf;
        IState.ChannelState memory channelState = IMain(mainAddress)
            .getChannelState(user);
        if (evidenceLeaf.transferCommitment != transfer.transferCommitment()) {
            revert TransferCommitmentMismatch({
                leafTransferCommitment: evidenceLeaf.transferCommitment,
                computedTransferCommitment: transfer.transferCommitment()
            });
        }
        if (evidenceLeaf.ebn <= channelState.ebn) {
            revert EvidenceLeafEbnIsTooOld({
                leafEbn: evidenceLeaf.ebn,
                channelEbn: channelState.ebn
            });
        }
        withdrawalRequests[user] = IState.WithdrawalRequest({
            // solhint-disable-next-line not-rely-on-time
            requestedAt: block.timestamp,
            airdropped: AssetLib.singleAsset(transfer.assetId, transfer.amount),
            redeposit: redeposit,
            newEbn: evidenceLeaf.ebn
        });
        emit WithdrawalRequested(user);
    }

    /**
     * @notice Operator challenges the withdrawal request. The user will not incur any penalties.
     * @param user The user address
     * @param paymentWithSignature The payment with signature
     * @param settlementProof The settlement proof of the channel state
     */
    function challengeWithdrawal(
        address user,
        IPayment.PaymentWithSignature memory paymentWithSignature,
        IMerkleProof.SettlementMerkleProof memory settlementProof
    ) external onlyRole(OPERATOR) onlyAfterRequest(user) {
        IMain(mainAddress).closeChannelAsChallenge(
            paymentWithSignature,
            settlementProof
        );
        delete withdrawalRequests[user];
        emit WithdrawalChallenged(user);
    }

    /**
     * @notice Operator accepts the withdrawal request.
     * The channel state will be updated according to the request.
     * @dev Inner function of `acceptWithdrawal` and `timeOutWithdrawal`
     * @param user The user address
     */
    function _acceptWithdrawal(address user) internal {
        IState.WithdrawalRequest memory withdrawalRequest = withdrawalRequests[
            user
        ];
        IAsset.Assets memory redeposit = withdrawalRequest.redeposit;
        if (withdrawalRequest.airdropped.isLe(redeposit)) {
            redeposit = withdrawalRequest.airdropped; // avoid underflow
        }
        IAsset.Assets memory userWithdrawal = withdrawalRequest.airdropped.sub(
            redeposit
        );
        // update withdrawalRequest
        delete withdrawalRequests[user];
        // update channel state
        IState.ChannelState memory channelState = IMain(mainAddress)
            .getChannelState(user);
        IMain(mainAddress).setChannelState(
            user,
            IState.ChannelState({
                userDeposit: redeposit,
                ebn: withdrawalRequest.newEbn,
                round: channelState.round + 1
            })
        );
        ILiquidityManager(liquidityManagerAddress).sendAssets(
            user,
            userWithdrawal
        );
    }

    /**
     * @notice Operator accepts the withdrawal request.
     * The channel state will be updated according to the request.
     * @param user The user address
     */
    function acceptWithdrawal(
        address user
    ) external onlyRole(OPERATOR) onlyAfterRequest(user) {
        _acceptWithdrawal(user);
        emit WithdrawalAccepted(user);
    }

    /**
     * @notice User can finalize the withdrawal request if the operator
     * does not challenge it within the `WITHDRAW_TIMEOUT`.
     * @param user The user address
     */
    function timeOutWithdrawal(address user) external onlyAfterRequest(user) {
        uint256 requestedAt = withdrawalRequests[user].requestedAt;
        // solhint-disable-next-line not-rely-on-time
        if (block.timestamp < requestedAt + WITHDRAW_TIMEOUT) {
            revert TimeOutIsNotReached(
                requestedAt + WITHDRAW_TIMEOUT,
                // solhint-disable-next-line not-rely-on-time
                block.timestamp
            );
        }
        _acceptWithdrawal(user);
        emit WithdrawalTimeOuted(user);
    }

    /// @dev Authorize the upgrade
    function _authorizeUpgrade(
        address
    ) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}
}

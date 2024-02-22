/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import {MessageHashUtils} from "@openzeppelin/contracts/utils/cryptography/MessageHashUtils.sol";
import {IPayment} from "../common-interface/IPayment.sol";

library SignatureLib {
    using ECDSA for bytes32;
    using MessageHashUtils for bytes32;

    error InvalidUniqueIdentifier(bytes32 expected, bytes32 got);
    error InvalidUserSignature(address expected, address recovered);
    error InvalidOperatorSignature(address expected, address recovered);
    error UserMismatch(address expected, address got);

    /// @dev uniqueIdentifier is used to prevent cross-chain replay attack.
    function getUniqueIdentifier() internal view returns (bytes32) {
        return keccak256(abi.encodePacked(block.chainid, address(this)));
    }

    function verifyPaymentSignature(
        IPayment.PaymentWithSignature memory paymentWithSignature,
        address operator,
        address user
    ) internal view {
        bytes32 uniqueIdentifier = getUniqueIdentifier();
        IPayment.Payment memory payment = paymentWithSignature.payment;
        if (payment.user != user) {
            revert UserMismatch(user, payment.user);
        }
        if (payment.uniqueIdentifier != uniqueIdentifier) {
            revert InvalidUniqueIdentifier(
                uniqueIdentifier,
                payment.uniqueIdentifier
            );
        }
        bytes32 hash = keccak256(
            abi.encodePacked(
                payment.uniqueIdentifier,
                payment.user,
                payment.round,
                payment.nonce,
                payment.userBalance.amounts,
                payment.operatorBalance.amounts,
                payment.airdropped.amounts,
                payment.spentDeposit.amounts,
                payment.latestTransferCommitment,
                payment.latestEbn,
                payment.customData
            )
        );
        bytes32 ethSignedHash = hash.toEthSignedMessageHash();
        address signer = ethSignedHash.recover(
            paymentWithSignature.userSignature
        );
        if (signer != user) {
            revert InvalidUserSignature(user, signer);
        }
        signer = ethSignedHash.recover(paymentWithSignature.operatorSignature);
        if (signer != operator) {
            revert InvalidOperatorSignature(operator, signer);
        }
    }
}

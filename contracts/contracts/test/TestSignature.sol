// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {SignatureLib} from "../utils/SignatureLib.sol";
import {IPayment} from "../common-interface/IPayment.sol";

contract TestSignature {
    using SignatureLib for IPayment.PaymentWithSignature;

    function verifyPaymentSignature(
        IPayment.PaymentWithSignature memory paymentWithSignature,
        address operator,
        address user
    ) external view {
        paymentWithSignature.verifyPaymentSignature(operator, user);
    }

    function getUniqueIdentifier() external view returns (bytes32) {
        return SignatureLib.getUniqueIdentifier();
    }
}

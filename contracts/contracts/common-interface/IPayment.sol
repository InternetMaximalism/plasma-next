/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IAsset} from "./IAsset.sol";

interface IPayment is IAsset {
    struct Payment {
        address user; // the address of the user.
        uint32 round; // incremented every time the payment channel is closed.
        uint32 nonce; // incremented every time the payment channel is updated.
        Assets userBalance;
        Assets operatorBalance;
        Assets airdropped; // the amount of airdrop to this user.
        Assets spentDeposit; // the total amount of spent deposit.
        uint64 latestEbn; // the latest ebn of airdrop to this user.
        address zkptlcAddress; // the address of zkptlc.
        bytes32 zkptlcInstance; // the instance of zkptlc.
    }

    struct PaymentWithSignature {
        Payment payment;
        bytes userSignature;
        bytes operatorSignature;
    }
}

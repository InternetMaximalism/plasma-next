/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IAsset} from "./IAsset.sol";
import {IPayment} from "./IPayment.sol";

interface IState is IAsset, IPayment {
    struct ChannelState {
        Assets userDeposit;
        uint64 ebn;
        uint32 round;
    }

    struct WithdrawalRequest {
        uint256 requestedAt; // 0 means no request
        Assets airdropped;
        Assets redeposit;
        uint64 newEbn;
    }
}

// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {Withdraw} from "../payment-channel/withdraw/Withdraw.sol";

contract TestWithdraw2 is Withdraw {
    function getVal() external pure returns (uint256) {
        return 4;
    }
}

/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {Main} from "../payment-channel/main/Main.sol";

contract TestMain2 is Main {
    function getVal() external pure returns (uint256) {
        return 10;
    }
}

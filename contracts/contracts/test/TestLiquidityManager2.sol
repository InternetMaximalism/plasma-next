// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {LiquidityManager} from "../liquidity-manager/LiquidityManager.sol";

contract TestLiquidityManager2 is LiquidityManager {
    function getVal() external pure returns (uint256) {
        return 2;
    }
}

// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {Config} from "../config/Config.sol";

contract TestConfig2 is Config {
    function getVal() external pure returns (uint256) {
        return 1;
    }
}

// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {BlockManager} from "../block-manager/BlockManager.sol";

contract TestBlockManager2 is BlockManager {
    function getVal() external pure returns (uint256) {
        return 7;
    }
}

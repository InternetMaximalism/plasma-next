// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {BlockManager} from "../block-manager/BlockManager.sol";

contract TestGetBlockHash is BlockManager {
    function getBlockHash(Block memory block_) external pure returns (bytes32) {
        return _computeBlockHash(block_);
    }
}

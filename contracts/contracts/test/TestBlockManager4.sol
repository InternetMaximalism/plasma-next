// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {BlockManager} from "../block-manager/BlockManager.sol";

contract TestBlockManager4 is BlockManager {
    bytes32 public blockHashIfAvailableResult;

    function setBlockHashIfAvailableResult(bytes32 _result) external {
        blockHashIfAvailableResult = _result;
    }

    function _getBlockHashIfAvailable(
        uint32
    ) internal view override returns (bytes32) {
        return blockHashIfAvailableResult;
    }
}

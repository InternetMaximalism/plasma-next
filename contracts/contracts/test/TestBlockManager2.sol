// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {BlockManager} from "../block-manager/BlockManager.sol";

contract TestBlockManager2 is BlockManager {
    function getVal() external pure returns (uint256) {
        return 7;
    }

    function testGetBlockHashIfAvailable(
        uint32 blockNumber
    ) external view returns (bytes32) {
        return _getBlockHashIfAvailable(blockNumber);
    }

    function testComputeHashChain(
        uint32 startBlockNumber,
        bytes32 startBlockHash,
        bytes32[] memory transferRoots,
        bytes32[] memory totalDepositHashes
    ) external pure returns (uint32 endBlockNumber, bytes32 endBlockHash) {
        return
            _computeHashChain(
                startBlockNumber,
                startBlockHash,
                transferRoots,
                totalDepositHashes
            );
    }

    function setLastBlockNumber(uint32 _lastBlockNumber) external {
        lastBlockNumber = _lastBlockNumber;
    }

    function setLastBlockHash(bytes32 _lastBlockHash) external {
        lastBlockHash = _lastBlockHash;
    }

    function setBlockHashCheckpoints(
        bytes32[] memory _blockHashCheckpoints
    ) external {
        blockHashCheckpoints = _blockHashCheckpoints;
    }
}

// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

contract TestBlockManager3 {
    mapping(uint32 => bytes32) public getBlockHashResult;

    function setGetBlockHashResult(
        uint32 _blockNumber,
        bytes32 _result
    ) external {
        getBlockHashResult[_blockNumber] = _result;
    }

    function getBlockHash(uint32 _blockNumber) external view returns (bytes32) {
        return getBlockHashResult[_blockNumber];
    }
}

/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IAsset} from "../common-interface/IAsset.sol";

interface IBlockManager {
    error BlockNumberTooBig(uint32 blockNumber, uint32 latestBlockNumber);

    struct Block {
        bytes32 prevBlockHash;
        bytes32 transferRoot;
        IAsset.Assets totalDeposit;
        uint32 blockNumber;
    }

    event Deposited(IAsset.Assets assets);

    event BlockPosted(
        uint256 indexed blockNumber,
        bytes32 indexed blockHash,
        bytes32 prevBlockHash,
        bytes32 transferRoot,
        IAsset.Assets totalDeposit
    );

    event BlockHashRelayed(
        uint32 indexed blockNumber,
        bool indexed isDestL2,
        bytes32 indexed blockHash
    );

    function config(
        address operator_,
        address liquidityManagerAddress_
    ) external;

    function depositAndPostBlocks(
        bytes32[] memory transferRoots,
        IAsset.Assets memory amounts
    ) external;

    function getBlockHash(uint32 blockNumber) external view returns (bytes32);

    function getLatestBlockNumber() external view returns (uint256);
}

/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IAsset} from "../common-interface/IAsset.sol";

interface IBlockManager {
    error BlockHashNotAvailable(
        uint32 blockNumber,
        uint32 lastBlockNumber,
        uint32 lastCheckpoint
    );

    error VerifyInclusionFailed(
        uint32 blockNumber,
        bytes32 computedBlockHash,
        bytes32 actualBlockHash
    );

    struct Block {
        bytes32 prevBlockHash;
        bytes32 transferRoot;
        IAsset.Assets totalDeposit;
        uint32 blockNumber;
    }

    event Deposited(
        IAsset.Assets deposit,
        IAsset.Assets totalDeposit,
        bytes32 totalDepositHash
    );

    event BlockPosted(
        uint256 indexed blockNumber,
        bytes32 prevBlockHash,
        bytes32 transferRoot,
        bytes32 totalDepositHash
    );

    function config(
        address operator_,
        address liquidityManagerAddress_
    ) external;

    function deposit(IAsset.Assets memory amount) external;

    function postBlocks(bytes32[] memory transferRoots) external;

    function verifyInclusion(
        uint32 targetBlockNumber,
        bytes32 targetBlockHash,
        bytes32[] memory transferRoots,
        bytes32[] memory totalDepositHashes
    ) external view;

    function getLastCheckpointBlockNumber() external view returns (uint32);
}

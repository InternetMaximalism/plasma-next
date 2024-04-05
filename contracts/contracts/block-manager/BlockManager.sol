/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {IBlockManager} from "./IBlockManager.sol";
import {ILiquidityManager} from "../liquidity-manager/ILiquidityManager.sol";
import {AssetLib} from "../utils/AssetLib.sol";
import {IAsset} from "../common-interface/IAsset.sol";

/**
 * @title BlockManager
 * @author Intmax
 * @notice This contract is responsible for managing the blocks.
 */
contract BlockManager is
    AccessControlUpgradeable,
    UUPSUpgradeable,
    IBlockManager
{
    using AssetLib for IAsset.Assets;

    /// @notice The interval of the block hash checkpoint.
    uint256 public constant BLOCK_HASH_CHECKPOINT_INTERVAL = 32;

    /// @notice operator role constant
    bytes32 public constant OPERATOR = keccak256("OPERATOR");

    /// @notice The address of the liquidity manager contract.
    address public liquidityManagerAddress;

    /// @notice The block hash checkpoints.
    bytes32[] public blockHashCheckpoints;

    /// @notice The last block hash.
    bytes32 public lastBlockHash;

    /// @notice The last block number.
    uint32 public lastBlockNumber;

    /// @notice The total deposit of the operator.
    IAsset.Assets internal totalDeposit;

    /// @notice The hash of totalDeposit.
    bytes32 public totalDepositHash;

    /// @dev Initialization of thi contract at the time of deployment or upgrade
    function initialize(address admin) public initializer {
        __AccessControl_init();
        __UUPSUpgradeable_init();
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
    }

    /// @dev Config contract addresses called by the Config contract
    function config(
        address operator_,
        address liquidityManagerAddress_
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        liquidityManagerAddress = liquidityManagerAddress_;
        _grantRole(OPERATOR, operator_);
        if (lastBlockHash == bytes32(0)) {
            _postGenesisBlock();
        }
    }

    /**
     * @notice Deposit operator assets to the liquidity manager.
     * @param amount The assets to be deposited.
     */
    function deposit(IAsset.Assets memory amount) external onlyRole(OPERATOR) {
        ILiquidityManager(liquidityManagerAddress).receiveAssets(
            _msgSender(),
            amount
        );
        totalDeposit = totalDeposit.add(amount);
        totalDepositHash = totalDeposit.hash();
        emit Deposited(amount, totalDeposit, totalDepositHash);
    }

    /**
     * @notice Post the genesis block.
     */
    function _postGenesisBlock() internal {
        totalDepositHash = totalDeposit.hash();
        lastBlockNumber = 0;
        lastBlockHash = keccak256(
            abi.encodePacked(
                lastBlockHash,
                bytes32(0), // transferRoot
                totalDepositHash,
                lastBlockNumber
            )
        );
        emit BlockPosted({
            blockNumber: lastBlockNumber,
            prevBlockHash: bytes32(0),
            transferRoot: bytes32(0),
            totalDepositHash: totalDepositHash
        });
        emit Deposited(totalDeposit, totalDeposit, totalDepositHash);
    }

    /**
     * @notice Compute the block hash and post the block.
     * @param transferRoot The transfer root of the block.
     */
    function _postBlock(bytes32 transferRoot) private {
        bytes32 prevBlockHash = lastBlockHash;
        lastBlockNumber += 1;
        lastBlockHash = keccak256(
            abi.encodePacked(
                prevBlockHash,
                transferRoot,
                totalDepositHash,
                lastBlockNumber
            )
        );
        if (lastBlockNumber % BLOCK_HASH_CHECKPOINT_INTERVAL == 0) {
            blockHashCheckpoints.push(lastBlockHash);
        }
        emit BlockPosted({
            blockNumber: lastBlockNumber,
            prevBlockHash: prevBlockHash,
            transferRoot: transferRoot,
            totalDepositHash: totalDepositHash
        });
    }

    /**
     * @notice Deposit operator assets and post the blocks.
     * @param transferRoots The transfer roots of each block.
     */
    function postBlocks(
        bytes32[] memory transferRoots
    ) external onlyRole(OPERATOR) {
        for (uint i = 0; i < transferRoots.length; i++) {
            _postBlock(transferRoots[i]);
        }
    }

    /**
     * @notice Get the corresponding block hash if available
     * @param blockNumber The block number
     * @return blockHash The corresponding block hash
     */
    function _getBlockHashIfAvailable(
        uint32 blockNumber
    ) internal view virtual returns (bytes32) {
        if (blockNumber == lastBlockNumber) {
            return lastBlockHash;
        }
        if (
            blockNumber % BLOCK_HASH_CHECKPOINT_INTERVAL == 0 &&
            blockNumber / BLOCK_HASH_CHECKPOINT_INTERVAL <
            blockHashCheckpoints.length
        ) {
            return
                blockHashCheckpoints[
                    blockNumber / BLOCK_HASH_CHECKPOINT_INTERVAL
                ];
        }
        revert BlockHashNotAvailable({
            blockNumber: blockNumber,
            lastBlockNumber: lastBlockNumber,
            lastCheckpoint: uint32(blockNumber / BLOCK_HASH_CHECKPOINT_INTERVAL)
        });
    }

    /**
     * @notice Compute the block hash chain
     * @param startBlockNumber Start block Number
     * @param startBlockHash Start block Hash
     * @param transferRoots Transfer roots from the blockNumber `startBlockNumber` + 1
     * @param totalDepositHashes Total deposit Hashes from the blockNumber `startBlockNumber` + 1
     */
    function _computeHashChain(
        uint32 startBlockNumber,
        bytes32 startBlockHash,
        bytes32[] memory transferRoots,
        bytes32[] memory totalDepositHashes
    ) internal pure returns (uint32 endBlockNumber, bytes32 endBlockHash) {
        bytes32 prevBlockHash = startBlockHash;
        for (uint256 i = 0; i < transferRoots.length; i++) {
            prevBlockHash = keccak256(
                abi.encodePacked(
                    prevBlockHash,
                    transferRoots[i],
                    totalDepositHashes[i],
                    uint32(i + startBlockNumber + 1)
                )
            );
        }
        endBlockNumber = uint32(startBlockNumber + transferRoots.length);
        endBlockHash = prevBlockHash;
    }

    /**
     * @notice Verify that the target block is included in the L2 blocks by verifying the hash chain.
     * @dev This function is called by `RootManager`.
     * @param targetBlockNumber The target block number.
     * @param targetBlockHash The target block hash.
     * @param transferRoots The transfer roots from block number `targetBlockNumber`+1
     * to either the block before the checkpoint or the last block.
     * @param totalDepositHashes The total deposit hashes from block number `targetBlockNumber`+1
     * to either the block before the checkpoint or the last block.
     */
    function verifyInclusion(
        uint32 targetBlockNumber,
        bytes32 targetBlockHash,
        bytes32[] memory transferRoots,
        bytes32[] memory totalDepositHashes
    ) external view {
        (uint32 endBlockNumber, bytes32 endBlockHash) = _computeHashChain(
            targetBlockNumber,
            targetBlockHash,
            transferRoots,
            totalDepositHashes
        );
        bytes32 blockHash = _getBlockHashIfAvailable(endBlockNumber);
        if (endBlockHash != blockHash) {
            revert VerifyInclusionFailed({
                blockNumber: endBlockNumber,
                computedBlockHash: endBlockHash,
                actualBlockHash: blockHash
            });
        }
    }

    /**
     * @notice Get the last checkpoint block number
     * @return The last checkpoint block number
     */
    function getLastCheckpointBlockNumber() external view returns (uint32) {
        return
            uint32(
                blockHashCheckpoints.length * BLOCK_HASH_CHECKPOINT_INTERVAL
            );
    }

    /// @dev Authorize the upgrade
    function _authorizeUpgrade(
        address
    ) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}
}

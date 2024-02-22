/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {IBlockManager} from "./IBlockManager.sol";
import {IRootManager} from "../root-manager/IRootManager.sol";
import {ILiquidityManager} from "../liquidity-manager/ILiquidityManager.sol";
import {AssetLib} from "../utils/AssetLib.sol";
import {IAsset} from "../common-interface/IAsset.sol";

/**
 * @title BlockManager
 * @notice This contract is responsible for managing the blocks.
 */
contract BlockManager is
    AccessControlUpgradeable,
    UUPSUpgradeable,
    IBlockManager
{
    using AssetLib for IAsset.Assets;

    /// @notice operator role constant
    bytes32 public constant OPERATOR = keccak256("OPERATOR");

    /// @notice Operator's address
    address public operator;
    /// @notice The address of the liquidity manager contract.
    address public liquidityManagerAddress;

    /// @notice Array that stores the block hash.
    bytes32[] public blocks;
    /// @notice Operator's deposit to the Plasma Next.
    IAsset.Assets internal operatorDeposit;

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
        operator = operator_;
        liquidityManagerAddress = liquidityManagerAddress_;
        _grantRole(OPERATOR, operator);
        if (blocks.length == 0) {
            _postGenesisBlock();
        }
    }

    /**
     * @notice Post the genesis block.
     */
    function _postGenesisBlock() private {
        IAsset.Assets memory zeroAssets;
        Block memory genesisBlock = Block({
            prevBlockHash: bytes32(0),
            transferRoot: bytes32(0),
            totalDeposit: zeroAssets,
            blockNumber: 0
        });
        _postBlock(genesisBlock);
    }

    /**
     * @notice Deposit operator assets to the liquidity manager.
     * @param amounts The amounts to be deposited by the operator.
     */
    function _deposit(IAsset.Assets memory amounts) private {
        ILiquidityManager(liquidityManagerAddress).receiveAssets(
            _msgSender(),
            amounts
        );
        operatorDeposit = operatorDeposit.add(amounts);
        emit Deposited(amounts);
    }

    /**
     * @notice Build a block with the given transfer root.
     * @param transferRoot The transfer root of the block.
     * @return The block.
     */
    function _buildBlock(
        bytes32 transferRoot
    ) private view returns (Block memory) {
        bytes32 latestBlockHash = blocks[blocks.length - 1];
        return
            Block({
                prevBlockHash: latestBlockHash,
                transferRoot: transferRoot,
                totalDeposit: operatorDeposit,
                blockNumber: uint32(blocks.length)
            });
    }

    /**
     * @notice Compute the block hash.
     * @param block_ The block to compute the hash.
     * @return The block hash.
     */
    function _computeBlockHash(
        Block memory block_
    ) internal pure returns (bytes32) {
        return
            keccak256(
                abi.encodePacked(
                    block_.prevBlockHash,
                    block_.transferRoot,
                    block_.totalDeposit.amounts,
                    block_.blockNumber
                )
            );
    }

    /**
     * @notice Compute the block hash and post the block.
     * @param _block The block to be posted.
     */
    function _postBlock(Block memory _block) private {
        bytes32 blockHash = _computeBlockHash(_block);
        blocks.push(blockHash);
        emit BlockPosted(
            _block.blockNumber,
            blockHash,
            _block.prevBlockHash,
            _block.transferRoot,
            _block.totalDeposit
        );
    }

    /**
     * @notice Deposit operator assets and post the blocks.
     * @param transferRoots The transfer roots of each block.
     * @param amounts The amounts to be deposited by the operator.
     */
    function depositAndPostBlocks(
        bytes32[] memory transferRoots,
        IAsset.Assets memory amounts
    ) external onlyRole(OPERATOR) {
        _deposit(amounts);
        for (uint i = 0; i < transferRoots.length; i++) {
            _postBlock(_buildBlock(transferRoots[i]));
        }
    }

    /**
     * @notice Get the block hash by the block number.
     * @param blockNumber The block number to get the block hash.
     * @return The block hash.
     */
    function getBlockHash(uint32 blockNumber) external view returns (bytes32) {
        uint32 latestBlockNumber = uint32(blocks.length - 1);
        if (blockNumber > latestBlockNumber) {
            revert BlockNumberTooBig(blockNumber, latestBlockNumber);
        }
        return blocks[blockNumber];
    }

    /**
     * @notice Get the latest block number.
     * @return The latest block number.
     */
    function getLatestBlockNumber() external view returns (uint256) {
        return blocks.length - 1;
    }

    /// @dev Authorize the upgrade
    function _authorizeUpgrade(
        address
    ) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}
}

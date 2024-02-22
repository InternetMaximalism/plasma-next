/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

import {ILiquidityManager} from "./ILiquidityManager.sol";

import {IAsset} from "../common-interface/IAsset.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/**
 * @title LiquidityManager
 * @notice This contract is responsible for managing liquidity.
 * It receives assets from a user and sends assets to a user.
 */
contract LiquidityManager is
    AccessControlUpgradeable,
    UUPSUpgradeable,
    ILiquidityManager
{
    using SafeERC20 for IERC20;

    /// @dev INNER_GROUP is a role that is allowed to send or receive assets.
    bytes32 public constant INNER_GROUP = keccak256("INNER_GROUP");
    /// @dev tokenAddresses is a struct that contains addresses of tokens.
    IAsset.AssetsAddress private tokenAddresses;

    /// @dev Initialization of thi contract at the time of deployment or upgrade
    function initialize(address admin) public initializer {
        __AccessControl_init();
        __UUPSUpgradeable_init();
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
    }

    /// @dev Config contract addresses called by the Config contract
    function config(
        IAsset.AssetsAddress memory tokenAddresses_,
        address blockManagerAddress_,
        address mainAddress_,
        address withdrawAddress_
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        tokenAddresses = tokenAddresses_;
        _grantRole(INNER_GROUP, blockManagerAddress_);
        _grantRole(INNER_GROUP, mainAddress_);
        _grantRole(INNER_GROUP, withdrawAddress_);
    }

    /**
     * @dev Recieve assets from a user.
     * @param sender The address of the user who sends assets.
     * @param assets The assets that the user sends.
     */
    function receiveAssets(
        address sender,
        IAsset.Assets memory assets
    ) external onlyRole(INNER_GROUP) {
        for (uint256 i = 0; i < tokenAddresses.addresses.length; i++) {
            if (assets.amounts[i] > 0) {
                IERC20(tokenAddresses.addresses[i]).safeTransferFrom(
                    sender,
                    address(this),
                    assets.amounts[i]
                );
            }
        }
    }

    /**
     * @notice Send assets to a user.
     * @param recipient The address of the user who receives assets.
     * @param assets  The assets that the user receives.
     */
    function sendAssets(
        address recipient,
        IAsset.Assets memory assets
    ) external onlyRole(INNER_GROUP) {
        for (uint256 i = 0; i < tokenAddresses.addresses.length; i++) {
            if (assets.amounts[i] > 0) {
                IERC20(tokenAddresses.addresses[i]).safeTransfer(
                    recipient,
                    assets.amounts[i]
                );
            }
        }
    }

    /// @dev Authorize the upgrade
    function _authorizeUpgrade(
        address
    ) internal override onlyRole(DEFAULT_ADMIN_ROLE) {}
}

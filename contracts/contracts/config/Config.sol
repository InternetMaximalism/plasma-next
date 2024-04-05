/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {AccessControlUpgradeable} from "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

import {IConfig} from "./IConfig.sol";
import {IVerifier} from "../verifier/IVerifier.sol";
import {IRootManager} from "../root-manager/IRootManager.sol";
import {IBlockManager} from "../block-manager/IBlockManager.sol";
import {ILiquidityManager} from "../liquidity-manager/ILiquidityManager.sol";
import {IMain} from "../payment-channel/main/IMain.sol";
import {IWithdraw} from "../payment-channel/withdraw/IWithdraw.sol";

import {IAsset} from "../common-interface/IAsset.sol";

contract Config is AccessControlUpgradeable, UUPSUpgradeable, IConfig {
    bytes32 public constant DEPLOYER = keccak256("DEPLOYER");

    AddressBook public addressBook;

    function initialize() public initializer {
        __AccessControl_init();
        __UUPSUpgradeable_init();
        _grantRole(DEPLOYER, _msgSender());
    }

    function configure(
        address operator_,
        IAsset.AssetsAddress memory tokenAddresses_,
        address halo2VerifyingKeyAddress_,
        address halo2VerifierAddress_,
        address verifier_,
        address rootManager_,
        address blockManager_,
        address liquidityManager_,
        address main_,
        address withdraw_
    ) external onlyRole(DEPLOYER) {
        IVerifier(verifier_).config(
            halo2VerifyingKeyAddress_,
            halo2VerifierAddress_
        );
        IRootManager(rootManager_).config(verifier_, blockManager_);
        IBlockManager(blockManager_).config(operator_, liquidityManager_);
        ILiquidityManager(liquidityManager_).config(
            tokenAddresses_,
            blockManager_,
            main_,
            withdraw_
        );
        IMain(main_).config(
            operator_,
            withdraw_,
            rootManager_,
            liquidityManager_
        );
        IWithdraw(withdraw_).config(
            operator_,
            main_,
            rootManager_,
            liquidityManager_
        );

        addressBook = AddressBook(
            operator_,
            tokenAddresses_,
            halo2VerifyingKeyAddress_,
            halo2VerifierAddress_,
            verifier_,
            rootManager_,
            blockManager_,
            liquidityManager_,
            main_,
            withdraw_
        );
    }

    function getAddressBook() external view returns (AddressBook memory) {
        return addressBook;
    }

    function _authorizeUpgrade(address) internal override onlyRole(DEPLOYER) {}
}

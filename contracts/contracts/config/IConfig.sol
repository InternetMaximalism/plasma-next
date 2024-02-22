/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IAsset} from "../common-interface/IAsset.sol";

interface IConfig {
    struct AddressBook {
        address operator;
        IAsset.AssetsAddress tokenAddresses;
        address halo2VerifyingKeyAddress;
        address halo2VerifierAddress;
        address verifier;
        address rootManager;
        address blockManager;
        address liquidityManager;
        address main;
        address withdraw;
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
    ) external;

    function getAddressBook() external view returns (AddressBook memory);
}

/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IAsset} from "../common-interface/IAsset.sol";

interface ILiquidityManager {
    function config(
        IAsset.AssetsAddress memory tokenAddresses_,
        address blockManagerAddress_,
        address mainAddress_,
        address withdrawAddress_
    ) external;

    function receiveAssets(
        address sender,
        IAsset.Assets memory assets
    ) external;

    function sendAssets(
        address recipient,
        IAsset.Assets memory assets
    ) external;
}

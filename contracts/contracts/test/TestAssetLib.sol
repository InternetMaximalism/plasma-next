// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IAsset} from "../common-interface/IAsset.sol";
import {AssetLib} from "../utils/AssetLib.sol";

contract TestAssetLib {
    using AssetLib for IAsset.Assets;

    function isLe(
        IAsset.Assets memory left,
        IAsset.Assets memory right
    ) external pure returns (bool) {
        return left.isLe(right);
    }

    function isZero(IAsset.Assets memory assets) external pure returns (bool) {
        return assets.isZero();
    }

    function isEq(
        IAsset.Assets memory left,
        IAsset.Assets memory right
    ) external pure returns (bool) {
        return left.isEq(right);
    }

    function add(
        IAsset.Assets memory left,
        IAsset.Assets memory right
    ) external pure returns (IAsset.Assets memory) {
        return left.add(right);
    }

    function singleAsset(
        uint256 assetId,
        uint256 amount
    ) external pure returns (IAsset.Assets memory) {
        return AssetLib.singleAsset(assetId, amount);
    }

    function sub(
        IAsset.Assets memory left,
        IAsset.Assets memory right
    ) external pure returns (IAsset.Assets memory) {
        return left.sub(right);
    }
}

/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IAsset} from "../common-interface/IAsset.sol";

library AssetLib {
    // left <= right
    function isLe(
        IAsset.Assets memory left,
        IAsset.Assets memory right
    ) internal pure returns (bool) {
        // numAssets = 4
        for (uint256 i = 0; i < 4; i++) {
            if (left.amounts[i] > right.amounts[i]) {
                return false;
            }
        }
        return true;
    }

    function isZero(IAsset.Assets memory assets) internal pure returns (bool) {
        // numAssets = 4
        for (uint256 i = 0; i < 4; i++) {
            if (assets.amounts[i] != 0) {
                return false;
            }
        }
        return true;
    }

    function isEq(
        IAsset.Assets memory left,
        IAsset.Assets memory right
    ) internal pure returns (bool) {
        // numAssets = 4
        for (uint256 i = 0; i < 4; i++) {
            if (left.amounts[i] != right.amounts[i]) {
                return false;
            }
        }
        return true;
    }

    function add(
        IAsset.Assets memory left,
        IAsset.Assets memory right
    ) internal pure returns (IAsset.Assets memory) {
        IAsset.Assets memory result;
        // numAssets = 4
        for (uint256 i = 0; i < 4; i++) {
            result.amounts[i] = left.amounts[i] + right.amounts[i];
        }
        return result;
    }

    function singleAsset(
        uint256 assetId,
        uint256 amount
    ) internal pure returns (IAsset.Assets memory) {
        IAsset.Assets memory assets;
        assets.amounts[assetId] += amount;
        return assets;
    }

    function sub(
        IAsset.Assets memory left,
        IAsset.Assets memory right
    ) internal pure returns (IAsset.Assets memory) {
        IAsset.Assets memory result;
        for (uint256 i = 0; i < 4; i++) {
            result.amounts[i] = left.amounts[i] - right.amounts[i];
        }
        return result;
    }
}

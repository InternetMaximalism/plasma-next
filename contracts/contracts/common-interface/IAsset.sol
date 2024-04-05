/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

interface IAsset {
    struct Assets {
        uint256[4] amounts;
    }

    struct AssetsDelta {
        int256[4] amounts;
    }

    struct AssetsAddress {
        address[4] addresses;
    }
}

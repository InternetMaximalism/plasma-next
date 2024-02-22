// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;
import {IAsset} from "../common-interface/IAsset.sol";

contract TestLiquidityManager4 {
    address[2] public latestRecipient;
    uint256[2] public latestAssets0;
    uint256[2] public latestAssets1;
    uint256[2] public latestAssets2;
    uint256[2] public latestAssets3;

    uint256 public index;

    function sendAssets(
        address recipient,
        IAsset.Assets memory assets
    ) external {
        latestRecipient[index] = recipient;
        latestAssets0[index] = assets.amounts[0];
        latestAssets1[index] = assets.amounts[1];
        latestAssets2[index] = assets.amounts[2];
        latestAssets3[index] = assets.amounts[3];
        index++;
    }
}

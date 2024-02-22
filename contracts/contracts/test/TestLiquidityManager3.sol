// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;
import {IAsset} from "../common-interface/IAsset.sol";

contract TestLiquidityManager3 {
    address public latestRecipient;
    uint256 public latestAssets0;
    uint256 public latestAssets1;
    uint256 public latestAssets2;
    uint256 public latestAssets3;

    function sendAssets(
        address recipient,
        IAsset.Assets memory assets
    ) external {
        latestRecipient = recipient;
        latestAssets0 = assets.amounts[0];
        latestAssets1 = assets.amounts[1];
        latestAssets2 = assets.amounts[2];
        latestAssets3 = assets.amounts[3];
    }
}

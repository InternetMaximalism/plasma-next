/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

interface ITransfer {
    struct Transfer {
        address recipient;
        uint256 amount;
        uint32 assetId;
    }
}

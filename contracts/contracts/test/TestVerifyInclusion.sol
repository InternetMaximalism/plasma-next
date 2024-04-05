// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.23;

import {BlockManager} from "../block-manager/BlockManager.sol";

contract TestVerifyInclusion is BlockManager {
    constructor() {
        _grantRole(OPERATOR, msg.sender);
        if (lastBlockHash == bytes32(0)) {
            _postGenesisBlock();
        }
    }
}

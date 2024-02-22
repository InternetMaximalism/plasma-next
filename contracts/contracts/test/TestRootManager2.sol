// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {RootManager} from "../root-manager/RootManager.sol";
import {IMerkleProof} from "../common-interface/IMerkleProof.sol";
import {MerkleProofLib} from "../utils/MerkleProofLib.sol";

contract TestRootManager2 is RootManager {
    function getVal() external pure returns (uint256) {
        return 9;
    }
}

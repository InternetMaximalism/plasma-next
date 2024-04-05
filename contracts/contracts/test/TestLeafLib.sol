// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {ILeaf} from "../common-interface/ILeaf.sol";
import {LeafLib} from "../utils/LeafLib.sol";

contract TestLeafLib {
    using LeafLib for ILeaf.WithdrawLeaf;
    using LeafLib for ILeaf.EvidenceLeaf;

    function hashWithdrawLeaf(
        ILeaf.WithdrawLeaf memory leaf
    ) external pure returns (bytes32) {
        return leaf.hashLeaf();
    }

    function hashEvidenceLeaf(
        ILeaf.EvidenceLeaf memory leaf
    ) external pure returns (bytes32) {
        return leaf.hashLeaf();
    }
}

/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IPayment} from "./IPayment.sol";
import {ILeaf} from "./ILeaf.sol";

interface IAdditionalZKPTLC {
    function verifyAdditionalZKPTLC(
        bytes memory customData
    ) external view returns (bool);
}

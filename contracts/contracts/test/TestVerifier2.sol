// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.23;

import {IPublicInputs} from "../common-interface/IPublicInputs.sol";

contract TestVerifier2 {
    bool public result;
    bool public isRevert = false;

    function setResult(bool _result) external {
        result = _result;
    }

    function setRevert(bool _isRevert) external {
        isRevert = _isRevert;
    }

    function verifyProof(
        IPublicInputs.PublicInputs memory,
        bytes memory
    ) external view returns (bool) {
        if (isRevert) {
            revert("TestVerifier2: revert");
        }
        return result;
    }
}

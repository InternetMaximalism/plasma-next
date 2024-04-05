// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

contract TestAdditionalZKPTLC {
    bool public errorFlg = false;

    function setErrorFlg(bool _errorFlg) external {
        errorFlg = _errorFlg;
    }

    function verifyCondition(bytes32, bytes memory) external view {
        if (errorFlg) {
            revert("error");
        }
    }
}

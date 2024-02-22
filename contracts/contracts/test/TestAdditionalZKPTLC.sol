// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

contract TestAdditionalZKPTLC {
    bool public result;

    function setVerifyAdditionalZKPTLCResult(bool _result) external {
        result = _result;
    }

    function verifyAdditionalZKPTLC(bytes memory) external view returns (bool) {
        return result;
    }
}

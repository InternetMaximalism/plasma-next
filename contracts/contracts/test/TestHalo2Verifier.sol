// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;
import {IAsset} from "../common-interface/IAsset.sol";

contract TestHalo2Verifier {
    bool public verifyProofResult;

    function verifyProof(
        address,
        bytes memory,
        uint256[] memory
    ) external view returns (bool) {
        return verifyProofResult;
    }

    function setVerifyProofResult(bool _value) external {
        verifyProofResult = _value;
    }
}

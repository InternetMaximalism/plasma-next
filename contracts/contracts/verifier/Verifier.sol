// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.23;

import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";
import {IVerifier} from "./IVerifier.sol";
import {IHalo2Verifier} from "../halo2-verifier/IHalo2Verifier.sol";

contract Verifier is IVerifier, AccessControl {
    address public halo2VerifyingKeyAddress;
    address public halo2VerifierAddress;

    constructor(address admin) {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
    }

    function config(
        address halo2VerifyingKeyAddress_,
        address halo2VerifierAddress_
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        halo2VerifyingKeyAddress = halo2VerifyingKeyAddress_;
        halo2VerifierAddress = halo2VerifierAddress_;
    }

    function verifyProof(
        PublicInputs memory pis,
        bytes memory proof
    ) external view returns (bool) {
        bool success = IHalo2Verifier(halo2VerifierAddress).verifyProof(
            halo2VerifyingKeyAddress,
            proof,
            _toSolidityPis(pis)
        );
        return success;
    }

    function _toSolidityPis(
        IVerifier.PublicInputs memory pis
    ) private pure returns (uint256[] memory) {
        bytes32 h = keccak256(
            abi.encodePacked(pis.blockHash, pis.evidenceRoot, pis.withdrawRoot)
        );
        uint32[8] memory result;
        for (uint i = 0; i < 8; i++) {
            result[i] = uint32(uint256(h) / (2 ** (32 * (7 - i))));
        }
        uint256[] memory result2 = new uint256[](4);
        uint p = 18446744069414584321; // goldilocks prime
        for (uint i = 0; i < 4; i++) {
            result2[i] =
                (uint(result[i * 2]) * (1 << 32) + uint(result[i * 2 + 1])) %
                p;
        }
        return result2;
    }
}

// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.23;

import {IPublicInputs} from "../common-interface/IPublicInputs.sol";

interface IVerifier is IPublicInputs {
    function config(
        address halo2VerifyingKeyAddress_,
        address halo2VerifierAddress_
    ) external;

    function verifyProof(
        PublicInputs memory pis,
        bytes memory proof
    ) external view returns (bool);
}

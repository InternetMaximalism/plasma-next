/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

interface IPublicInputs {
    struct PublicInputs {
        bytes32 blockHash;
        bytes32 evidenceRoot;
        bytes32 withdrawRoot;
    }
}

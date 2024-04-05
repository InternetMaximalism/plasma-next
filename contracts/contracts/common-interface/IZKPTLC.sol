/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IAsset} from "./IAsset.sol";

interface IZKPTLC {
    /// @notice This function, defined by the interface, is invoked by the main payment-channel contract
    /// to verify the ZKPTLC (Zero-Knowledge Proof Time-Locked Contract) condition.
    /// @param instance The fixed public data for the ZKPTLC, which is primarily the hash of the condition
    /// agreed upon by both the operator and the user.
    /// @param witness The data required to verify the condition specified by the `instance`.
    /// @return toOperatorDelta The amount of assets to be transferred from the user to the operator using the channel capacity.
    function verifyCondition(
        bytes32 instance,
        bytes memory witness
    ) external view returns (IAsset.AssetsDelta memory toOperatorDelta); // NUM_ASSETS = 4
}

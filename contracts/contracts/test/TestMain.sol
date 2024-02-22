/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IState, IAsset} from "../common-interface/IState.sol";
import {IPayment} from "../common-interface/IPayment.sol";
import {IMerkleProof} from "../common-interface/IMerkleProof.sol";

contract TestMain {
    mapping(address => IState.ChannelState) public channelStates;

    function setChannelStateEbn(address user, uint64 ebn) external {
        IAsset.Assets memory zeroAssets;
        channelStates[user] = IState.ChannelState({
            userDeposit: zeroAssets,
            ebn: ebn,
            round: 0
        });
    }

    function setChannelState(
        address user,
        IState.ChannelState memory channelState
    ) external {
        channelStates[user] = channelState;
    }

    function getChannelState(
        address user
    ) external view returns (IState.ChannelState memory) {
        return channelStates[user];
    }

    function closeChannelAsChallenge(
        IPayment.PaymentWithSignature memory paymentWithSignature,
        IMerkleProof.SettlementMerkleProof memory settlementProof
    ) external pure {}
}

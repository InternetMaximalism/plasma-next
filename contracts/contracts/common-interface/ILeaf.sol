/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IAsset} from "./IAsset.sol";

interface ILeaf is IAsset {
    struct EvidenceLeaf {
        bytes32 transferCommitment;
        uint64 ebn;
    }

    struct WithdrawLeaf {
        address recipient;
        Assets amount;
        uint64 startEbn;
        uint64 endEbn;
    }

    struct SettlementLeaf {
        WithdrawLeaf withdrawLeaf;
        EvidenceLeaf evidenceLeaf;
    }
}

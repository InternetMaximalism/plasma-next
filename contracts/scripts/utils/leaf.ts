import { Bytes32, SettlementLeaf } from "../types/common"
import { ethers } from "hardhat"

export function hashSettlementLeaf(leaf: SettlementLeaf): Bytes32 {
  const amounts = leaf.withdrawLeaf.amount.amounts
  return ethers.solidityPackedKeccak256(
    ["address", "uint256[4]", "uint64", "uint64", "bytes32", "uint64"],
    [
      leaf.withdrawLeaf.recipient,
      amounts,
      leaf.withdrawLeaf.startEbn,
      leaf.withdrawLeaf.endEbn,
      leaf.evidenceLeaf.transferCommitment,
      leaf.evidenceLeaf.ebn,
    ]
  )
}

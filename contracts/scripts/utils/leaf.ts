import { Bytes32, EvidenceLeaf, WithdrawLeaf } from "../types/common"
import { ethers } from "hardhat"

export function hashWithdrawLeaf(leaf: WithdrawLeaf): Bytes32 {
  const amounts = leaf.amount.amounts
  return ethers.solidityPackedKeccak256(
    ["address", "uint256[4]", "uint64", "uint64"],
    [leaf.recipient, amounts, leaf.startEbn, leaf.endEbn]
  )
}

export function hashEvidenceLeaf(leaf: EvidenceLeaf): Bytes32 {
  return ethers.solidityPackedKeccak256(
    ["bytes32", "uint64"],
    [leaf.transferCommitment, leaf.ebn]
  )
}

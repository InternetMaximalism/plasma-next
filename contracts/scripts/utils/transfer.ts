import { Bytes32, Transfer } from "../types/common"
import { ethers } from "hardhat"

export function computeTransferCommitment(transfer: Transfer): Bytes32 {
  return ethers.solidityPackedKeccak256(
    ["address", "uint256", "uint32"],
    [transfer.recipient, transfer.amount, transfer.assetId]
  )
}

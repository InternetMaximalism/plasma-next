import { Bytes32, Transfer, TransferInfo, U64 } from "../types/common"
import { ethers } from "hardhat"

export function computeTransferCommitment(transfer: Transfer): Bytes32 {
  return ethers.solidityPackedKeccak256(
    ["address", "uint256", "uint32", "uint32"],
    [transfer.recipient, transfer.amount, transfer.assetId, transfer.nonce]
  )
}

export function computeEbn(transferInfo: TransferInfo): U64 {
  return (
    BigInt(transferInfo.block.blockNumber) * (1n << 32n) +
    BigInt(transferInfo.transferIndex)
  )
}

import { ethers } from "hardhat"
import { Block, Bytes32 } from "../types/common"

export function getBlockHash(block: Block): Bytes32 {
  return ethers.solidityPackedKeccak256(
    ["bytes32", "bytes32", "uint256[4]", "uint32"],
    [
      block.prevBlockHash,
      block.transferRoot,
      block.totalDeposit.amounts,
      block.blockNumber,
    ]
  )
}

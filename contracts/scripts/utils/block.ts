import { ethers } from "hardhat"
import { Block, BlockWithAmounts, Bytes32, U32 } from "../types/common"
import { BlockManager } from "../../typechain-types"

export function getBlockHash(block: Block): Bytes32 {
  const r = ethers.solidityPackedKeccak256(
    ["bytes32", "bytes32", "bytes32", "uint32"],
    [
      block.prevBlockHash,
      block.transferRoot,
      block.totalDepositHash,
      block.blockNumber,
    ]
  )
  return r
}

export async function getBlocks(blockManager: BlockManager): Promise<Block[]> {
  const events = await blockManager.queryFilter(
    blockManager.filters.BlockPosted(undefined)
  )
  const blocks: Block[] = events.map((event) => {
    const { prevBlockHash, transferRoot, totalDepositHash, blockNumber } =
      event.args
    return {
      prevBlockHash,
      transferRoot,
      totalDepositHash,
      blockNumber: Number(blockNumber),
    }
  })
  return blocks
}

export async function getLastBlock(
  blockManager: BlockManager
): Promise<BlockWithAmounts> {
  const blockEvents = await blockManager.queryFilter(
    blockManager.filters.BlockPosted(undefined)
  )
  const blockEvent = blockEvents[blockEvents.length - 1]
  const block: Block = {
    prevBlockHash: blockEvent.args.prevBlockHash,
    transferRoot: blockEvent.args.transferRoot,
    totalDepositHash: blockEvent.args.totalDepositHash,
    blockNumber: Number(blockEvent.args.blockNumber),
  }
  const depositEvents = await blockManager.queryFilter(
    blockManager.filters.Deposited()
  )
  const depositEvent = depositEvents[depositEvents.length - 1]
  if (block.totalDepositHash !== depositEvent.args.totalDepositHash) {
    throw new Error("Invalid totalDepositHash")
  }
  return { block, totalDeposit: depositEvent.args.totalDeposit }
}

export function prepareRoots(
  startBlockNumber: U32,
  endBlockNumber: U32,
  blocks: Block[]
): { transferRoots: Bytes32[]; totalDepositHashes: Bytes32[] } {
  const transferRoots: Bytes32[] = []
  const totalDepositHashes: Bytes32[] = []
  for (let i = startBlockNumber + 1; i <= endBlockNumber; i++) {
    transferRoots.push(blocks[i].transferRoot)
    totalDepositHashes.push(blocks[i].totalDepositHash)
  }
  return { transferRoots, totalDepositHashes }
}

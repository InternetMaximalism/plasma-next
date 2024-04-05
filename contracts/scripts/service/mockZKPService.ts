import { randomBytes } from "ethers"
import {
  Assets,
  BlockInfo,
  BlockWithAmounts,
  EvidenceMerkleProof,
  Transfer,
  TransferInfo,
  U32,
  WithdrawMerkleProof,
  WrapPublicInputs,
} from "../types/common"
import {
  addAssets,
  addSingleAsset,
  hashAssets,
  zeroAssets,
} from "../utils/assets"
import { getBlockHash } from "../utils/block"
import { computeEbn, computeTransferCommitment } from "../utils/transfer"
import {
  generateDummyEvidenceMerkleProof,
  generateDummyWithdrawMerkleProof,
  getEvidenceRoot,
  getWithdrawRoot,
} from "../utils/merkleProof"
import { ethers } from "hardhat"

export class MockZKPService {
  prevBlock: BlockWithAmounts

  constructor(prevBlock: BlockWithAmounts) {
    this.prevBlock = prevBlock
  }

  async generateBlock(
    transfers: Transfer[],
    deposit: Assets
  ): Promise<BlockInfo> {
    const totalDeposit = addAssets(this.prevBlock.totalDeposit, deposit)
    const newBlock: BlockWithAmounts = {
      block: {
        prevBlockHash: getBlockHash(this.prevBlock.block),
        blockNumber: this.prevBlock.block.blockNumber + 1,
        transferRoot: ethers.hexlify(randomBytes(32)),
        totalDepositHash: hashAssets(totalDeposit),
      },
      totalDeposit: totalDeposit,
    }
    const transferInfo = transfers.map((transfer, i) => ({
      transfer,
      transferIndex: i as U32,
      transferMerkleProof: [],
      block: newBlock.block,
    }))
    return {
      block: newBlock,
      transferInfo,
    }
  }

  async tick(block: BlockWithAmounts): Promise<void> {
    this.prevBlock = block
  }

  async computeWithdrawProof(transferInfo: TransferInfo[]): Promise<{
    withdrawProof: WithdrawMerkleProof
    wrapPis: WrapPublicInputs
    blockNumber: U32
  }> {
    if (transferInfo.length === 0) {
      throw new Error("No transfers")
    }
    const recipient = transferInfo[0].transfer.recipient
    // assert all transfers have the same recipient
    if (!transferInfo.every((info) => info.transfer.recipient === recipient)) {
      throw new Error("All transfers must have the same recipient")
    }
    const sortedTransferInfo = transferInfo.sort((a, b) =>
      Number(computeEbn(a) - computeEbn(b))
    )
    const totalAmount = sortedTransferInfo.reduce(
      (acc, info) =>
        addSingleAsset(acc, info.transfer.amount, info.transfer.assetId),
      zeroAssets()
    )
    const first = sortedTransferInfo[0]
    const last = sortedTransferInfo[sortedTransferInfo.length - 1]
    const withdrawProof = generateDummyWithdrawMerkleProof(
      10,
      recipient,
      totalAmount,
      computeEbn(first),
      computeEbn(last)
    )
    const wrapPis = {
      blockHash: getBlockHash(this.prevBlock.block),
      evidenceRoot: ethers.ZeroHash,
      withdrawRoot: getWithdrawRoot(withdrawProof),
    }
    return {
      withdrawProof,
      wrapPis,
      blockNumber: this.prevBlock.block.blockNumber,
    }
  }

  async computeEvidenceProof(transferInfo: TransferInfo): Promise<{
    evidenceProof: EvidenceMerkleProof
    wrapPis: WrapPublicInputs
    blockNumber: U32
  }> {
    const evidenceProof = generateDummyEvidenceMerkleProof(
      10,
      computeTransferCommitment(transferInfo.transfer),
      computeEbn(transferInfo)
    )
    const wrapPis = {
      blockHash: getBlockHash(this.prevBlock.block),
      evidenceRoot: getEvidenceRoot(evidenceProof),
      withdrawRoot: ethers.ZeroHash,
    }
    return {
      evidenceProof,
      wrapPis,
      blockNumber: this.prevBlock.block.blockNumber,
    }
  }
}

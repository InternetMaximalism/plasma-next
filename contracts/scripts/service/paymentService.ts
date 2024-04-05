import {
  BlockManager,
  BlockManager__factory,
  Config__factory,
  IConfig,
  Main,
  Main__factory,
  RootManager,
  RootManager__factory,
  Withdraw,
  Withdraw__factory,
} from "../../typechain-types"
import {
  Address,
  BlockWithAmounts,
  Bytes,
  Bytes32,
  Payment,
  PaymentWithSignature,
  Transfer,
  TransferInfo,
  U256,
  U32,
  WrapPublicInputs,
} from "../types/common"
import { Signer } from "ethers"
import {
  getUniqueIdentifier,
  initialPayment,
  signPayment,
} from "../utils/payment"
import {
  addAssets,
  addSingleAsset,
  subSingleAsset,
  zeroAssets,
} from "../utils/assets"
import {} from "../utils/merkleProof"
import { ethers } from "hardhat"
import { TestToken__factory } from "../../typechain-types/factories/contracts/test"
import { getRandomU32 } from "../utils/random"
import { MockZKPService } from "./mockZKPService"
import { computeEbn, computeTransferCommitment } from "../utils/transfer"
import { encodeWitness } from "../utils/witness"
import {
  getBlockHash,
  getBlocks,
  getLastBlock,
  prepareRoots,
} from "../utils/block"

export class PaymentService {
  zkpService: MockZKPService

  uniqueIdentifier: Bytes32
  user: Signer
  operator: Signer
  lastSend: TransferInfo | undefined
  pendingAirdrops: TransferInfo[]
  spentAirdrops: TransferInfo[]
  payments: PaymentWithSignature[]

  main: Main
  withdraw: Withdraw
  rootManager: RootManager
  blockManager: BlockManager
  liquidityManagerAddress: Address
  tokens: Address[]
  zkptlcAddress: Address

  nonce: U32

  constructor(
    operator: Signer,
    user: Signer,
    addressBook: IConfig.AddressBookStructOutput,
    uniqueIdentifier: Bytes32,
    zkptlcAddress: Address,
    prevBlock: BlockWithAmounts
  ) {
    this.zkpService = new MockZKPService(prevBlock)
    this.user = user
    this.nonce = 0
    this.operator = operator
    this.user = user

    this.pendingAirdrops = []
    this.spentAirdrops = []
    this.lastSend = undefined
    this.payments = []

    this.blockManager = BlockManager__factory.connect(
      addressBook.blockManager,
      operator
    )
    this.rootManager = RootManager__factory.connect(
      addressBook.rootManager,
      operator
    )
    this.main = Main__factory.connect(addressBook.main, operator)
    this.withdraw = Withdraw__factory.connect(addressBook.withdraw, operator)
    this.tokens = addressBook.tokenAddresses.addresses
    this.liquidityManagerAddress = addressBook.liquidityManager
    this.uniqueIdentifier = uniqueIdentifier
    this.zkptlcAddress = zkptlcAddress
  }

  async getPrevPayment(): Promise<Payment> {
    const state = await this.main.getChannelState(await this.user.getAddress())
    const prevPayment =
      this.payments.length === 0
        ? initialPayment(
            await this.user.getAddress(),
            state.ebn,
            Number(state.round)
          )
        : this.payments[this.payments.length - 1].payment
    return prevPayment
  }

  async approveAll() {
    for (const tokenAddress of this.tokens) {
      const token = TestToken__factory.connect(
        tokenAddress.toString(),
        this.operator
      )
      await token.approve(this.liquidityManagerAddress, ethers.MaxUint256)
    }
  }

  async postBlock(transfer: Transfer): Promise<TransferInfo> {
    const transferAmount = addSingleAsset(
      zeroAssets(),
      transfer.amount,
      transfer.assetId
    )
    const blockInfo = await this.zkpService.generateBlock(
      [transfer],
      transferAmount
    )
    await this.zkpService.tick(blockInfo.block)
    await this.blockManager.deposit(transferAmount)
    await this.blockManager.postBlocks([blockInfo.block.block.transferRoot])
    return blockInfo.transferInfo[0]
  }

  async postRoot(blockNumber: U32, wrapPis: WrapPublicInputs) {
    const lastBlockNumber = await this.blockManager.lastBlockNumber()
    const lastCheckpoint =
      await this.blockManager.getLastCheckpointBlockNumber()
    let checkpoint: U32
    if (blockNumber <= lastCheckpoint) {
      checkpoint =
        blockNumber /
        Number(await this.blockManager.BLOCK_HASH_CHECKPOINT_INTERVAL())
    } else if (
      blockNumber > lastCheckpoint &&
      lastBlockNumber >= lastCheckpoint
    ) {
      checkpoint = Number(lastBlockNumber)
    } else {
      throw Error("invalid block number")
    }
    const blocks = await getBlocks(this.blockManager)
    const { transferRoots, totalDepositHashes } = prepareRoots(
      blockNumber,
      checkpoint,
      blocks
    )
    // assertion
    await this.blockManager.verifyInclusion(
      blockNumber,
      getBlockHash(blocks[blockNumber]),
      transferRoots,
      totalDepositHashes
    )
    await this.rootManager.postRoot(
      blockNumber,
      transferRoots,
      totalDepositHashes,
      wrapPis,
      "0x"
    )
  }

  async airdrop(amount: U256, assetId: U32) {
    const transfer: Transfer = {
      recipient: await this.user.getAddress(),
      amount,
      assetId,
      nonce: getRandomU32(),
    }
    const transferInfo = await this.postBlock(transfer)
    this.pendingAirdrops.push(transferInfo)
  }

  async applyPendingAirdrops(): Promise<Payment> {
    const totalPendingAirdrops = this.pendingAirdrops.reduce(
      (acc, cur) =>
        addSingleAsset(acc, cur.transfer.amount, cur.transfer.assetId),
      zeroAssets()
    )
    const prevPayment = await this.getPrevPayment()
    let latestEbn = prevPayment.latestEbn
    if (this.pendingAirdrops.length > 0) {
      const firstAirdrop = this.pendingAirdrops[0]
      if (computeEbn(firstAirdrop) <= latestEbn) {
        throw Error("first airdrop's ebn should be greater than latestEbn")
      }
    }
    for (const airdrop of this.pendingAirdrops) {
      if (computeEbn(airdrop) > latestEbn) latestEbn = computeEbn(airdrop)
    }
    const newPayment: Payment = {
      user: prevPayment.user,
      round: prevPayment.round,
      nonce: prevPayment.nonce,
      userBalance: addAssets(prevPayment.userBalance, totalPendingAirdrops),
      operatorBalance: prevPayment.operatorBalance,
      airdropped: addAssets(prevPayment.airdropped, totalPendingAirdrops),
      spentDeposit: prevPayment.spentDeposit,
      latestEbn,
      zkptlcAddress: ethers.ZeroAddress,
      zkptlcInstance: ethers.ZeroHash,
    }
    this.spentAirdrops.push(...this.pendingAirdrops)
    this.pendingAirdrops = []
    return newPayment
  }

  async send(recipient: Address, amount: U256, assetId: U32) {
    const transfer: Transfer = {
      recipient,
      amount,
      assetId,
      nonce: getRandomU32(),
    }
    const transferInfo = await this.postBlock(transfer)
    this.lastSend = transferInfo
    const prevPayment = await this.applyPendingAirdrops()
    const newPayment = {
      user: prevPayment.user,
      round: prevPayment.round,
      nonce: prevPayment.nonce + 1,
      userBalance: subSingleAsset(prevPayment.userBalance, amount, assetId),
      operatorBalance: addSingleAsset(
        prevPayment.operatorBalance,
        amount,
        assetId
      ),
      airdropped: prevPayment.airdropped,
      spentDeposit: prevPayment.spentDeposit,
      latestEbn: prevPayment.latestEbn,
      zkptlcAddress: this.zkptlcAddress,
      zkptlcInstance: computeTransferCommitment(transfer),
    }
    const newPaymentWithSignature = await signPayment(
      this.user,
      this.operator,
      this.uniqueIdentifier,
      newPayment
    )
    this.payments.push(newPaymentWithSignature)
  }

  async clearZkptlc() {
    const prevPayment = await this.applyPendingAirdrops()
    const newPayment = {
      user: prevPayment.user,
      round: prevPayment.round,
      nonce: prevPayment.nonce + 1,
      userBalance: prevPayment.userBalance,
      operatorBalance: prevPayment.operatorBalance,
      airdropped: prevPayment.airdropped,
      spentDeposit: prevPayment.spentDeposit,
      latestEbn: prevPayment.latestEbn,
      zkptlcAddress: ethers.ZeroAddress,
      zkptlcInstance: ethers.ZeroHash,
    }
    const newPaymentWithSignature = await signPayment(
      this.user,
      this.operator,
      this.uniqueIdentifier,
      newPayment
    )
    this.payments.push(newPaymentWithSignature)
  }

  async prepareWitness(
    instance: Bytes32,
    lastSend: TransferInfo
  ): Promise<Bytes> {
    const { evidenceProof, wrapPis, blockNumber } =
      await this.zkpService.computeEvidenceProof(lastSend)
    await this.postRoot(blockNumber, wrapPis)
    await this.rootManager.verifyEvidenceMerkleProof(evidenceProof) // assert
    const witness = {
      transfer: lastSend.transfer,
      evidenceProof,
    }
    const encodedWitness = encodeWitness(witness)
    // assertion
    const zkptlc = new ethers.Contract(
      this.zkptlcAddress.toString(),
      [
        "function verifyCondition(bytes32 instance,bytes memory witness) external view",
      ],
      this.operator
    )
    await zkptlc.verifyCondition(instance, encodedWitness)
    return encodedWitness
  }

  async closeChannel() {
    await this.clearZkptlc()
    const lastPayment = this.payments[this.payments.length - 1]
    const { withdrawProof, wrapPis, blockNumber } =
      await this.zkpService.computeWithdrawProof(this.spentAirdrops)
    await this.postRoot(blockNumber, wrapPis)
    await this.rootManager.verifyWithdrawMerkleProof(withdrawProof) // assert
    const witness = "0x"
    await this.main.closeChannel(
      lastPayment,
      withdrawProof,
      witness,
      zeroAssets()
    )
  }

  async closeChannelForce() {
    if (this.payments.length == 0) {
      throw Error("payment should be at least one")
    }
    const lastPayment = this.payments[this.payments.length - 1]
    let witness = "0x"
    if (lastPayment.payment.zkptlcAddress !== ethers.ZeroAddress) {
      if (!this.lastSend) {
        throw Error("last send should not be empty")
      }
      witness = await this.prepareWitness(
        lastPayment.payment.zkptlcInstance,
        this.lastSend
      )
    }
    // withdraw root
    const { withdrawProof, wrapPis, blockNumber } =
      await this.zkpService.computeWithdrawProof(this.spentAirdrops)
    await this.postRoot(blockNumber, wrapPis)
    await this.rootManager.verifyWithdrawMerkleProof(withdrawProof)
    await this.main.closeChannel(
      lastPayment,
      withdrawProof,
      witness,
      zeroAssets()
    )
  }

  async acceptWithdrawal() {
    const userAddress = await this.user.getAddress()
    await this.withdraw.acceptWithdrawal(userAddress)
  }

  async challengeWithdrawal() {
    if (this.payments.length == 0) {
      throw Error("payment should be at least one")
    }
    const lastPayment = this.payments[this.payments.length - 1]
    let witness = "0x"
    if (lastPayment.payment.zkptlcAddress !== ethers.ZeroAddress) {
      if (!this.lastSend) {
        throw Error("last send should not be empty")
      }
      witness = await this.prepareWitness(
        lastPayment.payment.zkptlcInstance,
        this.lastSend
      )
    }
    const { withdrawProof, wrapPis, blockNumber } =
      await this.zkpService.computeWithdrawProof(this.spentAirdrops)
    await this.postRoot(blockNumber, wrapPis)
    await this.rootManager.verifyWithdrawMerkleProof(withdrawProof)
    const userAddress = await this.user.getAddress()
    await this.withdraw.challengeWithdrawal(
      userAddress,
      lastPayment,
      withdrawProof,
      witness
    )
  }
}

export async function createPaymentService(
  operator: Signer,
  user: Signer,
  configAddress: Address,
  zkptlcAddress: Address
): Promise<PaymentService> {
  const config = Config__factory.connect(configAddress.toString(), operator)
  const addressBook = await config.getAddressBook()
  const blockManager = BlockManager__factory.connect(
    addressBook.blockManager,
    operator
  )
  const lastBlock = await getLastBlock(blockManager)
  const uniqueIdentifier = await getUniqueIdentifier(addressBook.main)
  return new PaymentService(
    operator,
    user,
    addressBook,
    uniqueIdentifier,
    zkptlcAddress,
    lastBlock
  )
}

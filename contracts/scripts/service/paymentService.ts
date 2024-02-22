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
  Payment,
  PaymentWithSignature,
  Transfer,
  U256,
  U32,
  U64,
  SettlementMerkleProof,
} from "../types/common"
import { Signer } from "ethers"
import { initialPayment, signPayment } from "../utils/payment"
import {
  addAssets,
  addSingleAsset,
  isLe,
  subAssets,
  subSingleAsset,
  sumTransfers,
  zeroAssets,
} from "../utils/assets"
import { computeTransferCommitment } from "../utils/transfer"
import {
  generateDummySettlementMerkleProof,
  getSettlementRoot,
} from "../utils/merkleProof"
import { ethers } from "hardhat"
import { TestToken__factory } from "../../typechain-types/factories/contracts/test"

export class PaymentService {
  user: Signer
  operator: Signer
  airdrops: Transfer[]
  sends: Transfer[]
  payments: PaymentWithSignature[]
  latestEbn: U64

  main: Main
  withdraw: Withdraw
  rootManager: RootManager
  blockManager: BlockManager
  liquidityManagerAddress: Address
  tokens: Address[]

  nonce: U32
  round: U32

  constructor(
    operator: Signer,
    user: Signer,
    addressBook: IConfig.AddressBookStructOutput,
    round: U32,
    latestEbn: U64
  ) {
    this.airdrops = []
    this.sends = []
    this.user = user
    this.payments = []
    this.nonce = 0
    this.round = round
    this.latestEbn = latestEbn
    this.operator = operator
    this.user = user

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
  }

  async getPrevPayment(): Promise<Payment> {
    const uniqueIdentifier = await this.main.getUniqueIdentifier()
    const curEbn = (
      await this.main.getChannelState(await this.user.getAddress())
    ).ebn
    const prevPayment =
      this.payments.length === 0
        ? initialPayment(
            uniqueIdentifier,
            await this.user.getAddress(),
            curEbn,
            this.round
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

  async airdrop(amount: U256, assetId: U32) {
    const airdropTransfer = {
      recipient: await this.user.getAddress(),
      amount,
      assetId,
    }
    const airdropAmount = addSingleAsset(zeroAssets(), amount, assetId)
    await this.blockManager.depositAndPostBlocks([], airdropAmount) // actually, this will be done with batch
    this.latestEbn += 100n // todo: use actual ebn
    this.airdrops.push(airdropTransfer)

    const prevPayment = await this.getPrevPayment()

    const newPayment = {
      uniqueIdentifier: prevPayment.uniqueIdentifier,
      user: await this.user.getAddress(),
      round: prevPayment.round,
      nonce: prevPayment.nonce + 1,
      userBalance: addSingleAsset(prevPayment.userBalance, amount, assetId),
      operatorBalance: prevPayment.operatorBalance,
      airdropped: addSingleAsset(prevPayment.airdropped, amount, assetId),
      spentDeposit: prevPayment.spentDeposit,
      latestTransferCommitment: prevPayment.latestTransferCommitment,
      latestEbn: this.latestEbn,
      customData: "0x",
    }

    const newPaymentWithSignature = await signPayment(
      this.user,
      this.operator,
      newPayment
    )
    this.payments.push(newPaymentWithSignature)
  }

  async send(recipient: Address, amount: U256, assetId: U32) {
    const sendTransfer = {
      recipient,
      amount,
      assetId,
    }
    const transferAmount = addSingleAsset(zeroAssets(), amount, assetId)
    await this.blockManager.depositAndPostBlocks([], transferAmount) // actually, this will be done with batch
    const latestTransferCommitment = computeTransferCommitment(sendTransfer)
    this.sends.push(sendTransfer)

    const prevPayment = await this.getPrevPayment()

    const newPayment = {
      uniqueIdentifier: prevPayment.uniqueIdentifier,
      user: await this.user.getAddress(),
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
      latestTransferCommitment,
      latestEbn: this.latestEbn,
      customData: "0x",
    }
    const newPaymentWithSignature = await signPayment(
      this.user,
      this.operator,
      newPayment
    )
    this.payments.push(newPaymentWithSignature)
  }

  async reflectDeposit() {
    const userAddress = await this.user.getAddress()
    const currentDeposit = (await this.main.getChannelState(userAddress))
      .userDeposit
    const prevPayment = await this.getPrevPayment()
    const prevSpentDeposit = prevPayment.spentDeposit

    if (isLe(currentDeposit, prevSpentDeposit)) {
      throw new Error(
        `currentDeposit ${currentDeposit} <= prevSpentDeposit ${prevSpentDeposit}`
      )
    }

    const delta = subAssets(currentDeposit, prevSpentDeposit)
    const newUserBalance = addAssets(prevPayment.userBalance, delta)

    const newPayment = {
      uniqueIdentifier: prevPayment.uniqueIdentifier,
      user: await this.user.getAddress(),
      round: prevPayment.round,
      nonce: prevPayment.nonce + 1,
      userBalance: newUserBalance,
      operatorBalance: prevPayment.operatorBalance,
      airdropped: prevPayment.airdropped,
      spentDeposit: currentDeposit,
      latestTransferCommitment: prevPayment.latestTransferCommitment,
      latestEbn: prevPayment.latestEbn,
      customData: "0x",
    }
    const newPaymentWithSignature = await signPayment(
      this.user,
      this.operator,
      newPayment
    )
    this.payments.push(newPaymentWithSignature)
  }

  async postSettlementRoot(): Promise<SettlementMerkleProof> {
    if (this.airdrop.length === 0) {
      throw new Error("no payment")
    }
    const oldEbn =
      (await this.main.getChannelState(await this.user.getAddress())).ebn + 1n // todo: this should be the oldest ebn
    const newEbn = this.latestEbn
    const totalAirdrop = sumTransfers(this.airdrops)
    const latestPayment = await this.getPrevPayment()

    const w = generateDummySettlementMerkleProof(
      10,
      await this.user.getAddress(),
      totalAirdrop,
      oldEbn,
      newEbn,
      latestPayment.latestTransferCommitment,
      this.latestEbn
    )
    await this.rootManager.postRoot(
      0,
      {
        blockHash: ethers.ZeroHash,
        settlementRoot: getSettlementRoot(w),
      },
      "0x"
    )
    await this.rootManager.verifySettlementMerkleProof(w)
    return w
  }
}

export async function createPaymentService(
  operator: Signer,
  user: Signer,
  configAddress: Address
): Promise<PaymentService> {
  const config = Config__factory.connect(configAddress.toString(), operator)
  const addressBook = await config.getAddressBook()
  const main = Main__factory.connect(addressBook.main, operator)
  const userAddress = await user.getAddress()
  const channelState = await main.getChannelState(userAddress)
  const round = Number(channelState.round)
  const latestEbn = channelState.ebn
  return new PaymentService(operator, user, addressBook, round, latestEbn)
}

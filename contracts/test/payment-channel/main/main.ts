import { expect } from "chai"
import { ethers, upgrades } from "hardhat"
import { loadFixture } from "@nomicfoundation/hardhat-toolbox/network-helpers"
import {
  getSigners,
  deployAllContracts,
  generateDummyAddresses,
  getUupsContract,
  testHash1,
  testHash2,
} from "../../test-utils"
import { Config, Main, TestLiquidityManager4 } from "../../../typechain-types"
import {
  getUniqueIdentifier,
  initialPayment,
  signPayment,
} from "../../../scripts/utils/payment"
import { Payment, PaymentWithSignature } from "../../../scripts/types/common"
import {
  IPayment,
  IMerkleProof,
} from "../../../typechain-types/contracts/payment-channel/main/Main"

describe("main", () => {
  const setup = async (): Promise<[Main, Config]> => {
    const config = await deployAllContracts()
    const addressBook = await config.getAddressBook()
    const mainFactory = await ethers.getContractFactory("Main")
    const main = mainFactory.attach(addressBook.main)
    return [main as Main, config]
  }
  const createPaymentWithSignature = (
    payment: Payment,
    signed: PaymentWithSignature
  ): IPayment.PaymentWithSignatureStruct => {
    return {
      payment,
      userSignature: signed.userSignature,
      operatorSignature: signed.operatorSignature,
    }
  }
  const generateSettlementMerkleProofTemplate =
    (): IMerkleProof.WithdrawWithMerkleProofStruct => {
      return {
        leaf: {
          recipient: ethers.ZeroAddress,
          amount: {
            amounts: [0n, 0n, 0n, 0n],
          },
          startEbn: 0n,
          endEbn: 0n,
        },
        index: 1n,
        siblings: [ethers.ZeroHash, ethers.ZeroHash],
      }
    }
  const setupCloseChannel = async (): Promise<
    [Main, TestLiquidityManager4, Payment]
  > => {
    const mainFactory = await ethers.getContractFactory("Main")
    const main = await mainFactory.deploy()
    const testRootManager3Factory = await ethers.getContractFactory(
      "TestRootManager3"
    )
    const testRootManager3 = await testRootManager3Factory.deploy()
    const testLiquidityManager4Factory = await ethers.getContractFactory(
      "TestLiquidityManager4"
    )
    const testLiquidityManager4 = await testLiquidityManager4Factory.deploy()
    const signers = await getSigners()
    await main.initialize(signers.dummyConfig.address)
    await main
      .connect(signers.dummyConfig)
      .config(
        signers.operator.address,
        signers.dummyWithdraw.address,
        await testRootManager3.getAddress(),
        await testLiquidityManager4.getAddress()
      )
    await main
      .connect(signers.dummyWithdraw)
      .setChannelState(signers.user.address, {
        userDeposit: {
          amounts: [10n, 11n, 12n, 13n],
        },
        ebn: 1n,
        round: 2n,
      })
    const payment = initialPayment(signers.user.address, 10n, 0)
    return [main, testLiquidityManager4, payment]
  }
  describe("initialize", () => {
    describe("success", () => {
      it("initialize was called", async () => {
        const [main, config] = await loadFixture(setup)
        const result = await main.hasRole(
          await main.DEFAULT_ADMIN_ROLE(),
          await config.getAddress()
        )
        expect(result).to.equal(true)
      })
    })
    describe("fail", () => {
      it("Initialization can only be done once", async () => {
        const [main] = await loadFixture(setup)
        await expect(
          main.initialize(ethers.ZeroAddress)
        ).to.be.revertedWithCustomError(main, "InvalidInitialization")
      })
    })
  })

  describe("config", () => {
    describe("success", () => {
      it("set address info", async () => {
        const mainFactory = await ethers.getContractFactory("Main")
        const main = await mainFactory.deploy()
        const signers = await getSigners()
        await main.initialize(signers.dummyConfig.address)
        const addresses = generateDummyAddresses(4)
        const operatorRole = await main.OPERATOR()
        const innerGroupRole = await main.INNER_GROUP()
        expect(await main.hasRole(operatorRole, addresses[0])).to.equal(false)
        expect(await main.hasRole(innerGroupRole, addresses[1])).to.equal(false)
        await main
          .connect(signers.dummyConfig)
          .config(addresses[0], addresses[1], addresses[2], addresses[3])

        expect(await main.hasRole(operatorRole, addresses[0])).to.equal(true)
        expect(await main.hasRole(innerGroupRole, addresses[1])).to.equal(true)
        expect(await main.operator()).to.equal(addresses[0])
        expect(await main.rootManagerAddress()).to.equal(addresses[2])
        expect(await main.liquidityManagerAddress()).to.equal(addresses[3])
      })
    })
    describe("fail", () => {
      it("only admin", async () => {
        const [main] = await loadFixture(setup)
        const signers = await getSigners()
        const role = await main.DEFAULT_ADMIN_ROLE()
        const testAddress = generateDummyAddresses(4)
        await expect(
          main
            .connect(signers.illegalUser)
            .config(
              testAddress[0],
              testAddress[1],
              testAddress[2],
              testAddress[3]
            )
        )
          .to.be.revertedWithCustomError(
            main,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, role)
      })
    })
  })
  describe("deposit", () => {
    it("deposit tokens", async () => {
      const [main, config] = await loadFixture(setup)
      const addressBook = await config.getAddressBook()
      await Promise.all(
        addressBook.tokenAddresses.addresses.map(async (erc20Address) => {
          const erc20 = await ethers.getContractAt("TestToken", erc20Address)

          await erc20.approve(addressBook.liquidityManager, 100n)
        })
      )
      await Promise.all(
        addressBook.tokenAddresses.addresses.map(async (erc20Address) => {
          const erc20 = await ethers.getContractAt("TestToken", erc20Address)

          const balance = await erc20.balanceOf(addressBook.liquidityManager)
          expect(balance).to.equal(0)
        })
      )
      await main.deposit({
        amounts: [10n, 10n, 10n, 10n],
      })
      await Promise.all(
        addressBook.tokenAddresses.addresses.map(async (erc20Address) => {
          const erc20 = await ethers.getContractAt("TestToken", erc20Address)

          const balance = await erc20.balanceOf(addressBook.liquidityManager)
          expect(balance).to.equal(10)
        })
      )
    })
    it("update channel status", async () => {
      const [main, config] = await loadFixture(setup)
      const addressBook = await config.getAddressBook()
      const signers = await getSigners()
      await Promise.all(
        addressBook.tokenAddresses.addresses.map(async (erc20Address) => {
          const erc20 = await ethers.getContractAt("TestToken", erc20Address)

          await erc20.approve(addressBook.liquidityManager, 100n)
        })
      )
      const status = await main.channelStates(signers.deployer.address)
      expect(status.userDeposit.amounts).to.eql([0n, 0n, 0n, 0n])
      await main.deposit({
        amounts: [10n, 10n, 10n, 10n],
      })
      const newStatus = await main.channelStates(signers.deployer.address)
      expect(newStatus.userDeposit.amounts).to.eql([10n, 10n, 10n, 10n])
    })
    it("generate Deposited event", async () => {
      const [main, config] = await loadFixture(setup)
      const addressBook = await config.getAddressBook()
      const signers = await getSigners()
      await Promise.all(
        addressBook.tokenAddresses.addresses.map(async (erc20Address) => {
          const erc20 = await ethers.getContractAt("TestToken", erc20Address)

          await erc20.approve(addressBook.liquidityManager, 100n)
        })
      )
      await main.deposit({
        amounts: [10n, 10n, 10n, 10n],
      })
      // Get the Deposited event from the main contract
      const events = await main.queryFilter(main.filters.Deposited())
      expect(events.length).to.equal(1)
      const event = events[0]
      expect(event.args.user).to.equal(signers.deployer.address)
      expect(event.args.assets.amounts).to.eql([10n, 10n, 10n, 10n])
    })
  })
  describe("getChannelState, setChannelState", () => {
    describe("success", async () => {
      it("get channel status", async () => {
        const mainFactory = await ethers.getContractFactory("Main")
        const main = await mainFactory.deploy()
        const signers = await getSigners()
        await main.initialize(signers.dummyConfig.address)
        const addresses = generateDummyAddresses(3)
        await main
          .connect(signers.dummyConfig)
          .config(
            addresses[0],
            signers.dummyWithdraw.address,
            addresses[1],
            addresses[2]
          )
        const state = await main.getChannelState(signers.user.address)
        expect(state.userDeposit.amounts).to.eql([0n, 0n, 0n, 0n])
        expect(state.ebn).to.equal(0n)
        expect(state.round).to.equal(0n)
        await main
          .connect(signers.dummyWithdraw)
          .setChannelState(signers.user.address, {
            userDeposit: {
              amounts: [10n, 11n, 12n, 13n],
            },
            ebn: 1n,
            round: 2n,
          })
        const newState = await main.getChannelState(signers.user.address)
        expect(newState.userDeposit.amounts).to.eql([10n, 11n, 12n, 13n])
        expect(newState.ebn).to.equal(1n)
        expect(newState.round).to.equal(2n)
      })
    })
    describe("fail", async () => {
      it("only inner group", async () => {
        const [main] = await loadFixture(setup)
        const role = await main.INNER_GROUP()
        const signers = await getSigners()
        await expect(
          main
            .connect(signers.illegalUser)
            .setChannelState(ethers.ZeroAddress, {
              userDeposit: {
                amounts: [10n, 11n, 12n, 13n],
              },
              ebn: 1n,
              round: 2n,
            })
        )
          .to.be.revertedWithCustomError(
            main,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, role)
      })
    })
  })
  describe("getUniqueIdentifier", () => {
    it("set address info", async () => {
      const [main] = await loadFixture(setup)
      const identifier = await main.getUniqueIdentifier()
      const tmp = await getUniqueIdentifier(await main.getAddress())
      expect(identifier).to.equal(tmp)
    })
  })

  describe("closeChannel", () => {
    describe("success", async () => {
      it("update channel state", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const state = await main.getChannelState(signers.user.address)
        expect(state.userDeposit.amounts).to.eql([10n, 11n, 12n, 13n])
        expect(state.ebn).to.equal(1n)
        expect(state.round).to.equal(2n)
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await main
          .connect(signers.operator)
          .closeChannel(
            createPaymentWithSignature(payment, signed),
            proof,
            "0x",
            {
              amounts: [1n, 2n, 3n, 4n],
            }
          )
        const newState = await main.getChannelState(signers.user.address)
        expect(newState.userDeposit.amounts).to.eql([1n, 2n, 3n, 4n])
        expect(newState.ebn).to.equal(10n)
        expect(newState.round).to.equal(3n)
      })
      it("call sendAssets func twice", async () => {
        const [main, liquidityManager, payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]
        await main
          .connect(signers.operator)
          .closeChannel(
            createPaymentWithSignature(payment, signed),
            proof,
            "0x",
            {
              amounts: [1n, 2n, 3n, 4n],
            }
          )
        const latestRecipient0 = await liquidityManager.latestRecipient(0)
        const latestAssets0_0 = await liquidityManager.latestAssets0(0)
        const latestAssets1_0 = await liquidityManager.latestAssets1(0)
        const latestAssets2_0 = await liquidityManager.latestAssets2(0)
        const latestAssets3_0 = await liquidityManager.latestAssets3(0)

        expect(latestRecipient0).to.equal(signers.operator.address)
        expect(latestAssets0_0).to.equal(payment.operatorBalance.amounts[0])
        expect(latestAssets1_0).to.equal(payment.operatorBalance.amounts[1])
        expect(latestAssets2_0).to.equal(payment.operatorBalance.amounts[2])
        expect(latestAssets3_0).to.equal(payment.operatorBalance.amounts[3])

        const latestRecipient1 = await liquidityManager.latestRecipient(1)
        const latestAssets0_1 = await liquidityManager.latestAssets0(1)
        const latestAssets1_1 = await liquidityManager.latestAssets1(1)
        const latestAssets2_1 = await liquidityManager.latestAssets2(1)
        const latestAssets3_1 = await liquidityManager.latestAssets3(1)

        expect(latestRecipient1).to.equal(signers.user.address)
        expect(latestAssets0_1).to.equal(9n)
        expect(latestAssets1_1).to.equal(9n)
        expect(latestAssets2_1).to.equal(9n)
        expect(latestAssets3_1).to.equal(9n)
      })
      it("generate ChannelClosed event", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]
        await main
          .connect(signers.operator)
          .closeChannel(
            createPaymentWithSignature(payment, signed),
            proof,
            "0x",
            {
              amounts: [1n, 2n, 3n, 4n],
            }
          )
        const events = await main.queryFilter(main.filters.ChannelClosed())
        expect(events.length).to.equal(1)
        const event = events[0]
        expect(event.args.user).to.equal(signers.user.address)
        expect(event.args.round).to.equal(payment.round)
      })
    })
    describe("fail", async () => {
      it("only operator", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()

        await expect(
          main
            .connect(signers.illegalUser)
            .closeChannel(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x",
              {
                amounts: [1n, 2n, 3n, 4n],
              }
            )
        )
          .to.be.revertedWithCustomError(
            main,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, await main.OPERATOR())
      })
      it("recipient mismatch", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.illegalUser.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.operator)
            .closeChannel(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x",
              {
                amounts: [1n, 2n, 3n, 4n],
              }
            )
        )
          .to.be.revertedWithCustomError(main, "RecipientMismatch")
          .withArgs(signers.illegalUser.address, signers.user.address)
      })
      it("round mismatch", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 3
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.operator)
            .closeChannel(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x",
              {
                amounts: [1n, 2n, 3n, 4n],
              }
            )
        )
          .to.be.revertedWithCustomError(main, "RoundMismatch")
          .withArgs(2, payment.round)
      })
      it("ebn sanity check failed", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 4n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.operator)
            .closeChannel(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x",
              {
                amounts: [1n, 2n, 3n, 4n],
              }
            )
        )
          .to.be.revertedWithCustomError(main, "EbnSanityCheckFailed")
          .withArgs(proof.leaf.startEbn, proof.leaf.endEbn)
      })
      it("latest ebn mismatch", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 9n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.operator)
            .closeChannel(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x",
              {
                amounts: [1n, 2n, 3n, 4n],
              }
            )
        )
          .to.be.revertedWithCustomError(main, "LatestEbnMismatch")
          .withArgs(proof.leaf.endEbn, payment.latestEbn)
      })
      it("leaf start ebn is too old", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 0n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.operator)
            .closeChannel(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x",
              {
                amounts: [1n, 2n, 3n, 4n],
              }
            )
        )
          .to.be.revertedWithCustomError(main, "LeafStartEbnIsTooOld")
          .withArgs(proof.leaf.startEbn, 1)
      })
      it("airdropped amount mismatch", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = [1n, 2n, 3n, 5n]
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.operator)
            .closeChannel(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x",
              {
                amounts: [1n, 2n, 3n, 4n],
              }
            )
        )
          .to.be.revertedWithCustomError(main, "AirdroppedAmountMismatch")
          .withArgs([[1n, 2n, 3n, 5n]], [payment.airdropped.amounts])
      })
      it("ZKPTLC verification failed", async () => {
        const [main, , payment] = await setupCloseChannel()
        const testAdditionalZKPTLCFactory = await ethers.getContractFactory(
          "TestAdditionalZKPTLC"
        )
        const testAdditionalZKPTLC = await testAdditionalZKPTLCFactory.deploy()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        payment.zkptlcAddress = await testAdditionalZKPTLC.getAddress()
        await testAdditionalZKPTLC.setErrorFlg(true)
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.operator)
            .closeChannel(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x",
              {
                amounts: [1n, 2n, 3n, 4n],
              }
            )
        )
          .to.be.revertedWithCustomError(main, "ZKPTLCVerificationFailed")
          .withArgs(payment.zkptlcAddress)
      })
      it("spent more than deposit", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        payment.spentDeposit.amounts = [11n, 11n, 12n, 13n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.operator)
            .closeChannel(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x",
              {
                amounts: [1n, 2n, 3n, 4n],
              }
            )
        )
          .to.be.revertedWithCustomError(main, "SpentMoreThanDeposit")
          .withArgs([payment.spentDeposit.amounts], [[10n, 11n, 12n, 13n]])
      })
      it("invariant violation", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 5n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.operator)
            .closeChannel(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x",
              {
                amounts: [1n, 2n, 3n, 4n],
              }
            )
        )
          .to.be.revertedWithCustomError(main, "InvariantViolation")
          .withArgs(
            [payment.airdropped.amounts],
            [payment.operatorBalance.amounts]
          )
      })
    })
  })

  describe("closeChannelAsChallenge", () => {
    describe("success", async () => {
      it("update channel state", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const state = await main.getChannelState(signers.user.address)
        expect(state.userDeposit.amounts).to.eql([10n, 11n, 12n, 13n])
        expect(state.ebn).to.equal(1n)
        expect(state.round).to.equal(2n)
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await main
          .connect(signers.dummyWithdraw)
          .closeChannelAsChallenge(
            createPaymentWithSignature(payment, signed),
            proof,
            "0x"
          )
        const newState = await main.getChannelState(signers.user.address)
        expect(newState.userDeposit.amounts).to.eql([0n, 0n, 0n, 0n])
        expect(newState.ebn).to.equal(10n)
        expect(newState.round).to.equal(3n)
      })
      it("call sendAssets func twice", async () => {
        const [main, liquidityManager, payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]
        await main
          .connect(signers.dummyWithdraw)
          .closeChannelAsChallenge(
            createPaymentWithSignature(payment, signed),
            proof,
            "0x"
          )
        const latestRecipient0 = await liquidityManager.latestRecipient(0)
        const latestAssets0_0 = await liquidityManager.latestAssets0(0)
        const latestAssets1_0 = await liquidityManager.latestAssets1(0)
        const latestAssets2_0 = await liquidityManager.latestAssets2(0)
        const latestAssets3_0 = await liquidityManager.latestAssets3(0)
        expect(latestRecipient0).to.equal(signers.operator.address)
        expect(latestAssets0_0).to.equal(payment.operatorBalance.amounts[0])
        expect(latestAssets1_0).to.equal(payment.operatorBalance.amounts[1])
        expect(latestAssets2_0).to.equal(payment.operatorBalance.amounts[2])
        expect(latestAssets3_0).to.equal(payment.operatorBalance.amounts[3])

        const latestRecipient1 = await liquidityManager.latestRecipient(1)
        const latestAssets0_1 = await liquidityManager.latestAssets0(1)
        const latestAssets1_1 = await liquidityManager.latestAssets1(1)
        const latestAssets2_1 = await liquidityManager.latestAssets2(1)
        const latestAssets3_1 = await liquidityManager.latestAssets3(1)

        expect(latestRecipient1).to.equal(signers.user.address)
        expect(latestAssets0_1).to.equal(10n)
        expect(latestAssets1_1).to.equal(11n)
        expect(latestAssets2_1).to.equal(12n)
        expect(latestAssets3_1).to.equal(13n)
      })
      it("generate ChannelClosed event", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]
        await main
          .connect(signers.dummyWithdraw)
          .closeChannelAsChallenge(
            createPaymentWithSignature(payment, signed),
            proof,
            "0x"
          )
        const events = await main.queryFilter(main.filters.ChannelClosed())
        expect(events.length).to.equal(1)
        const event = events[0]
        expect(event.args.user).to.equal(signers.user.address)
        expect(event.args.round).to.equal(payment.round)
      })
    })
    describe("fail", async () => {
      it("only operator", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()

        await expect(
          main
            .connect(signers.illegalUser)
            .closeChannelAsChallenge(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x"
            )
        )
          .to.be.revertedWithCustomError(
            main,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, await main.INNER_GROUP())
      })
      it("recipient mismatch", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.illegalUser.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.dummyWithdraw)
            .closeChannelAsChallenge(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x"
            )
        )
          .to.be.revertedWithCustomError(main, "RecipientMismatch")
          .withArgs(signers.illegalUser.address, signers.user.address)
      })
      it("round mismatch", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 3
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.dummyWithdraw)
            .closeChannelAsChallenge(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x"
            )
        )
          .to.be.revertedWithCustomError(main, "RoundMismatch")
          .withArgs(2, payment.round)
      })
      it("ebn sanity check failed", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 4n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.dummyWithdraw)
            .closeChannelAsChallenge(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x"
            )
        )
          .to.be.revertedWithCustomError(main, "EbnSanityCheckFailed")
          .withArgs(proof.leaf.startEbn, proof.leaf.endEbn)
      })
      it("latest ebn mismatch", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 9n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.dummyWithdraw)
            .closeChannelAsChallenge(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x"
            )
        )
          .to.be.revertedWithCustomError(main, "LatestEbnMismatch")
          .withArgs(proof.leaf.endEbn, payment.latestEbn)
      })
      it("leaf start ebn is too old", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 0n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.dummyWithdraw)
            .closeChannelAsChallenge(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x"
            )
        )
          .to.be.revertedWithCustomError(main, "LeafStartEbnIsTooOld")
          .withArgs(proof.leaf.startEbn, 1)
      })
      it("airdropped amount mismatch", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = [1n, 2n, 3n, 5n]
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.dummyWithdraw)
            .closeChannelAsChallenge(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x"
            )
        )
          .to.be.revertedWithCustomError(main, "AirdroppedAmountMismatch")
          .withArgs([[1n, 2n, 3n, 5n]], [payment.airdropped.amounts])
      })
      it("spent more than deposit", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        payment.spentDeposit.amounts = [11n, 11n, 12n, 13n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.dummyWithdraw)
            .closeChannelAsChallenge(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x"
            )
        )
          .to.be.revertedWithCustomError(main, "SpentMoreThanDeposit")
          .withArgs([payment.spentDeposit.amounts], [[10n, 11n, 12n, 13n]])
      })
      it("invariant violation", async () => {
        const [main, , payment] = await setupCloseChannel()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 5n]
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]

        await expect(
          main
            .connect(signers.dummyWithdraw)
            .closeChannelAsChallenge(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x"
            )
        )
          .to.be.revertedWithCustomError(main, "InvariantViolation")
          .withArgs(
            [payment.airdropped.amounts],
            [payment.operatorBalance.amounts]
          )
      })
      it("ZKPTLC verification failed", async () => {
        const [main, , payment] = await setupCloseChannel()
        const testAdditionalZKPTLCFactory = await ethers.getContractFactory(
          "TestAdditionalZKPTLC"
        )
        const testAdditionalZKPTLC = await testAdditionalZKPTLCFactory.deploy()
        const testAdditionalZKPTLCAddress =
          await testAdditionalZKPTLC.getAddress()
        const signers = await getSigners()
        payment.round = 2
        payment.airdropped.amounts = [1n, 2n, 3n, 4n]
        payment.operatorBalance.amounts = [1n, 2n, 3n, 4n]
        payment.zkptlcAddress = testAdditionalZKPTLCAddress

        await testAdditionalZKPTLC.setErrorFlg(true)
        const identifier = await getUniqueIdentifier(await main.getAddress())
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        const proof = generateSettlementMerkleProofTemplate()
        proof.leaf.recipient = signers.user.address
        proof.leaf.amount.amounts = payment.airdropped.amounts
        proof.leaf.startEbn = 5n
        proof.leaf.endEbn = 10n
        proof.index = 1n
        proof.siblings = [testHash1, testHash2]
        await expect(
          main
            .connect(signers.dummyWithdraw)
            .closeChannelAsChallenge(
              createPaymentWithSignature(payment, signed),
              proof,
              "0x"
            )
        )
          .to.be.revertedWithCustomError(main, "ZKPTLCVerificationFailed")
          .withArgs(testAdditionalZKPTLCAddress)
      })
    })
  })

  describe("upgrade", () => {
    it("contract is upgradable", async () => {
      const signers = await getSigners()
      const main = await getUupsContract<Main>("Main", [
        signers.dummyConfig.address,
      ])
      const role = await main.DEFAULT_ADMIN_ROLE()
      const result = await main.hasRole(role, signers.dummyConfig.address)
      expect(result).to.equal(true)

      const factory = await ethers.getContractFactory(
        "TestMain2",
        signers.dummyConfig
      )
      const next = await upgrades.upgradeProxy(await main.getAddress(), factory)
      const result2 = await next.hasRole(role, signers.dummyConfig.address)
      expect(result).to.equal(result2)
      const val: number = (await next.getVal()) as number
      expect(val).to.equal(10)
    })
    it("Cannot upgrade except for a deployer.", async () => {
      const [main] = await loadFixture(setup)
      const signers = await getSigners()
      const factory = await ethers.getContractFactory(
        "TestMain2",
        signers.illegalUser
      )
      const role = await main.DEFAULT_ADMIN_ROLE()
      await expect(upgrades.upgradeProxy(await main.getAddress(), factory))
        .to.be.revertedWithCustomError(main, "AccessControlUnauthorizedAccount")
        .withArgs(signers.illegalUser.address, role)
    })
  })
})

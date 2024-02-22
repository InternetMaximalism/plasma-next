import { expect } from "chai"
import { ethers, upgrades } from "hardhat"
import {
  loadFixture,
  time,
} from "@nomicfoundation/hardhat-toolbox/network-helpers"
import {
  getSigners,
  deployAllContracts,
  generateDummyAddresses,
  generateDummyHashes,
  getUupsContract,
  testAddress1,
} from "../../test-utils"
import {
  Config,
  Withdraw,
  TestMain,
  TestLiquidityManager3,
} from "../../../typechain-types"
import { IAsset } from "../../../typechain-types/contracts/payment-channel/withdraw/Withdraw"

describe("Withdraw", () => {
  const setup = async (): Promise<[Withdraw, Config]> => {
    const config = await deployAllContracts()
    const addressBook = await config.getAddressBook()
    const testWithdrawFactory = await ethers.getContractFactory("Withdraw")
    const withdraw = testWithdrawFactory.attach(addressBook.withdraw)
    return [withdraw as unknown as Withdraw, config]
  }
  const setupRequestWithdrawal = async (): Promise<[Withdraw, TestMain]> => {
    const testWithdrawFactory = await ethers.getContractFactory("Withdraw")
    const withdraw = await testWithdrawFactory.deploy()
    const testTestMainFactory = await ethers.getContractFactory("TestMain")
    const main = await testTestMainFactory.deploy()
    const testTestRootManager3Factory = await ethers.getContractFactory(
      "TestRootManager3"
    )
    const rootManager = await testTestRootManager3Factory.deploy()

    const signers = await getSigners()
    const testAddress = generateDummyAddresses(1)

    await withdraw.initialize(signers.dummyConfig.address)
    await withdraw
      .connect(signers.dummyConfig)
      .config(
        signers.operator.address,
        await main.getAddress(),
        await rootManager.getAddress(),
        testAddress[0]
      )
    await main.setChannelStateEbn(signers.user.address, 10n)
    return [withdraw, main]
  }
  const setupAcceptWithdrawal = async (
    changeWithdrawLeafAmount = false
  ): Promise<[Withdraw, TestMain, TestLiquidityManager3]> => {
    const testWithdrawFactory = await ethers.getContractFactory("Withdraw")
    const withdraw = await testWithdrawFactory.deploy()
    const testTestMainFactory = await ethers.getContractFactory("TestMain")
    const main = await testTestMainFactory.deploy()
    const testTestLiquidityManager3Factory = await ethers.getContractFactory(
      "TestLiquidityManager3"
    )
    const liquidityManager = await testTestLiquidityManager3Factory.deploy()
    const testTestRootManager3Factory = await ethers.getContractFactory(
      "TestRootManager3"
    )
    const rootManager = await testTestRootManager3Factory.deploy()
    const signers = await getSigners()

    await withdraw.initialize(signers.dummyConfig.address)
    await withdraw
      .connect(signers.dummyConfig)
      .config(
        signers.operator.address,
        await main.getAddress(),
        await rootManager.getAddress(),
        await liquidityManager.getAddress()
      )
    const testHashes = generateDummyHashes(3)
    const tmp = changeWithdrawLeafAmount
      ? ({
          amounts: [10n, 11n, 12n, 13n],
        } as IAsset.AssetsStruct)
      : withdrawLeafAmounts
    await withdraw.connect(signers.user).requestWithdrawal(
      {
        leaf: {
          withdrawLeaf: {
            recipient: signers.user.address,
            amount: tmp,
            startEbn: 11n,
            endEbn: 15n,
          },
          evidenceLeaf: {
            transferCommitment: testHashes[0],
            ebn: 4n,
          },
        },
        index: 0n,
        siblings: [testHashes[1], testHashes[2]],
      },
      amounts
    )
    return [withdraw, main, liquidityManager]
  }
  const setupChallengeWithdrawal = async (): Promise<Withdraw> => {
    const [withdraw] = await setupRequestWithdrawal()
    const signers = await getSigners()
    const testHashes = generateDummyHashes(3)
    await withdraw.connect(signers.user).requestWithdrawal(
      {
        leaf: {
          withdrawLeaf: {
            recipient: signers.user.address,
            amount: withdrawLeafAmounts,
            startEbn: 11n,
            endEbn: 15n,
          },
          evidenceLeaf: {
            transferCommitment: testHashes[0],
            ebn: 4n,
          },
        },
        index: 0n,
        siblings: [testHashes[1], testHashes[2]],
      },
      amounts
    )
    return withdraw
  }
  const withdrawLeafAmounts = {
    amounts: [1n, 2n, 3n, 4n],
  } as IAsset.AssetsStruct
  const amounts = { amounts: [5n, 6n, 7n, 8n] } as IAsset.AssetsStruct
  describe("initialize", () => {
    describe("success", () => {
      it("initialize was called", async () => {
        const [withdraw, config] = await loadFixture(setup)
        const result = await withdraw.hasRole(
          await withdraw.DEFAULT_ADMIN_ROLE(),
          await config.getAddress()
        )
        expect(result).to.equal(true)
      })
    })
    describe("fail", () => {
      it("Initialization can only be done once", async () => {
        const [withdraw] = await loadFixture(setup)
        await expect(
          withdraw.initialize(ethers.ZeroAddress)
        ).to.be.revertedWithCustomError(withdraw, "InvalidInitialization")
      })
    })
  })

  describe("config", () => {
    describe("success", () => {
      it("set address info", async () => {
        const testWithdrawFactory = await ethers.getContractFactory("Withdraw")
        const withdraw = await testWithdrawFactory.deploy()
        const signers = await getSigners()
        await withdraw.initialize(signers.dummyConfig.address)
        const addresses = generateDummyAddresses(4)
        await withdraw
          .connect(signers.dummyConfig)
          .config(addresses[0], addresses[1], addresses[2], addresses[3])
        const role = await withdraw.OPERATOR()
        expect(await withdraw.hasRole(role, addresses[0])).to.equal(true)
        expect(await withdraw.mainAddress()).to.equal(addresses[1])
        expect(await withdraw.rootManagerAddress()).to.equal(addresses[2])
        expect(await withdraw.liquidityManagerAddress()).to.equal(addresses[3])
      })
    })
    describe("fail", () => {
      it("only admin", async () => {
        const [withdraw] = await loadFixture(setup)
        const signers = await getSigners()
        const role = await withdraw.DEFAULT_ADMIN_ROLE()
        const testAddresses = generateDummyAddresses(4)
        await expect(
          withdraw
            .connect(signers.illegalUser)
            .config(
              testAddresses[0],
              testAddresses[1],
              testAddresses[2],
              testAddresses[3]
            )
        )
          .to.be.revertedWithCustomError(
            withdraw,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, role)
      })
    })
  })

  describe("requestWithdrawal", () => {
    describe("success", () => {
      it("set withdraw requests", async () => {
        const [withdraw] = await setupRequestWithdrawal()
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)
        await withdraw.connect(signers.user).requestWithdrawal(
          {
            leaf: {
              withdrawLeaf: {
                recipient: signers.user.address,
                amount: withdrawLeafAmounts,
                startEbn: 11n,
                endEbn: 15n,
              },
              evidenceLeaf: {
                transferCommitment: testHashes[0],
                ebn: 4n,
              },
            },
            index: 0n,
            siblings: [testHashes[1], testHashes[2]],
          },
          amounts
        )
        const withdrawalRequests = await withdraw.withdrawalRequests(
          signers.user.address
        )
        expect(withdrawalRequests.requestedAt).to.equal(await time.latest())
        expect(withdrawalRequests.airdropped.amounts).to.eql(
          withdrawLeafAmounts.amounts
        )
        expect(withdrawalRequests.redeposit.amounts).to.eql(amounts.amounts)
        expect(withdrawalRequests.newEbn).to.equal(15n)
      })
      it("generate WithdrawalRequested event", async () => {
        const [withdraw] = await setupRequestWithdrawal()
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)
        await expect(
          withdraw.connect(signers.user).requestWithdrawal(
            {
              leaf: {
                withdrawLeaf: {
                  recipient: signers.user.address,
                  amount: withdrawLeafAmounts,
                  startEbn: 11n,
                  endEbn: 15n,
                },
                evidenceLeaf: {
                  transferCommitment: testHashes[0],
                  ebn: 4n,
                },
              },
              index: 0n,
              siblings: [testHashes[1], testHashes[2]],
            },
            amounts
          )
        )
          .to.emit(withdraw, "WithdrawalRequested")
          .withArgs(signers.user.address)
      })
    })
    describe("fail", () => {
      it("invalid user", async () => {
        const [withdraw] = await setupRequestWithdrawal()
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)
        await expect(
          withdraw.requestWithdrawal(
            {
              leaf: {
                withdrawLeaf: {
                  recipient: signers.user.address,
                  amount: withdrawLeafAmounts,
                  startEbn: 11n,
                  endEbn: 15n,
                },
                evidenceLeaf: {
                  transferCommitment: testHashes[0],
                  ebn: 4n,
                },
              },
              index: 0n,
              siblings: [testHashes[1], testHashes[2]],
            },
            amounts
          )
        )
          .to.be.revertedWithCustomError(withdraw, "InvalidUser")
          .withArgs(signers.deployer.address, signers.user.address)
      })
      it("ebn sanity check failed", async () => {
        const [withdraw] = await setupRequestWithdrawal()
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)
        await expect(
          withdraw.connect(signers.user).requestWithdrawal(
            {
              leaf: {
                withdrawLeaf: {
                  recipient: signers.user.address,
                  amount: withdrawLeafAmounts,
                  startEbn: 16n,
                  endEbn: 15n,
                },
                evidenceLeaf: {
                  transferCommitment: testHashes[0],
                  ebn: 4n,
                },
              },
              index: 0n,
              siblings: [testHashes[1], testHashes[2]],
            },
            amounts
          )
        )
          .to.be.revertedWithCustomError(withdraw, "EbnSanityCheckFailed")
          .withArgs(16n, 15n)
      })
      it("leaf start ebn is too old", async () => {
        const [withdraw, main] = await setupRequestWithdrawal()
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)
        await main.setChannelStateEbn(signers.user.address, 15n)
        await expect(
          withdraw.connect(signers.user).requestWithdrawal(
            {
              leaf: {
                withdrawLeaf: {
                  recipient: signers.user.address,
                  amount: withdrawLeafAmounts,
                  startEbn: 11n,
                  endEbn: 15n,
                },
                evidenceLeaf: {
                  transferCommitment: testHashes[0],
                  ebn: 4n,
                },
              },
              index: 0n,
              siblings: [testHashes[1], testHashes[2]],
            },
            amounts
          )
        )
          .to.be.revertedWithCustomError(withdraw, "LeafStartEbnIsTooOld")
          .withArgs(11n, 15n)
      })
      it("withdrawal request already exists", async () => {
        const [withdraw] = await setupRequestWithdrawal()
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)
        await withdraw.connect(signers.user).requestWithdrawal(
          {
            leaf: {
              withdrawLeaf: {
                recipient: signers.user.address,
                amount: withdrawLeafAmounts,
                startEbn: 11n,
                endEbn: 15n,
              },
              evidenceLeaf: {
                transferCommitment: testHashes[0],
                ebn: 4n,
              },
            },
            index: 0n,
            siblings: [testHashes[1], testHashes[2]],
          },
          amounts
        )
        await expect(
          withdraw.connect(signers.user).requestWithdrawal(
            {
              leaf: {
                withdrawLeaf: {
                  recipient: signers.user.address,
                  amount: withdrawLeafAmounts,
                  startEbn: 11n,
                  endEbn: 15n,
                },
                evidenceLeaf: {
                  transferCommitment: testHashes[0],
                  ebn: 4n,
                },
              },
              index: 0n,
              siblings: [testHashes[1], testHashes[2]],
            },
            amounts
          )
        )
          .to.be.revertedWithCustomError(
            withdraw,
            "WithdrawalRequestAlreadyExists"
          )
          .withArgs(signers.user.address)
      })
    })
  })
  describe("requestWithdrawalWithEvidence", () => {
    describe("success", () => {
      it("set withdraw requests", async () => {
        const [withdraw] = await setupRequestWithdrawal()
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)

        await withdraw.connect(signers.user).requestWithdrawalWithEvidence(
          {
            recipient: testAddress1,
            amount: 2n,
            assetId: 0n,
          },
          {
            leaf: {
              withdrawLeaf: {
                recipient: signers.user.address,
                amount: withdrawLeafAmounts,
                startEbn: 11n,
                endEbn: 15n,
              },
              evidenceLeaf: {
                transferCommitment:
                  "0x1dc73b3c28607bdd2e8c9fd554516e95a5b636dc2bc8d71401476ce2aa42a11e",
                ebn: 20n,
              },
            },
            index: 0n,
            siblings: [testHashes[1], testHashes[2]],
          },
          amounts
        )
        const withdrawalRequests = await withdraw.withdrawalRequests(
          signers.user.address
        )
        expect(withdrawalRequests.requestedAt).to.equal(await time.latest())
        expect(withdrawalRequests.airdropped.amounts).to.eql([2n, 0n, 0n, 0n])
        expect(withdrawalRequests.redeposit.amounts).to.eql(amounts.amounts)
        expect(withdrawalRequests.newEbn).to.equal(20n)
      })
      it("generate WithdrawalRequested event", async () => {
        const [withdraw] = await setupRequestWithdrawal()
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)
        await expect(
          withdraw.connect(signers.user).requestWithdrawalWithEvidence(
            {
              recipient: testAddress1,
              amount: 2n,
              assetId: 0n,
            },
            {
              leaf: {
                withdrawLeaf: {
                  recipient: signers.user.address,
                  amount: withdrawLeafAmounts,
                  startEbn: 11n,
                  endEbn: 15n,
                },
                evidenceLeaf: {
                  transferCommitment:
                    "0x1dc73b3c28607bdd2e8c9fd554516e95a5b636dc2bc8d71401476ce2aa42a11e",
                  ebn: 20n,
                },
              },
              index: 0n,
              siblings: [testHashes[1], testHashes[2]],
            },
            amounts
          )
        )
          .to.emit(withdraw, "WithdrawalRequested")
          .withArgs(signers.user.address)
      })
    })
    describe("fail", () => {
      it("transfer commitment mismatch", async () => {
        const [withdraw] = await setupRequestWithdrawal()
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)
        await expect(
          withdraw.requestWithdrawalWithEvidence(
            {
              recipient: testAddress1,
              amount: 2n,
              assetId: 0n,
            },
            {
              leaf: {
                withdrawLeaf: {
                  recipient: signers.user.address,
                  amount: withdrawLeafAmounts,
                  startEbn: 11n,
                  endEbn: 15n,
                },
                evidenceLeaf: {
                  transferCommitment: testHashes[0],
                  ebn: 4n,
                },
              },
              index: 0n,
              siblings: [testHashes[1], testHashes[2]],
            },
            amounts
          )
        )
          .to.be.revertedWithCustomError(withdraw, "TransferCommitmentMismatch")
          .withArgs(
            testHashes[0],
            "0x1dc73b3c28607bdd2e8c9fd554516e95a5b636dc2bc8d71401476ce2aa42a11e"
          )
      })
      it("evidence leaf ebn is too old", async () => {
        const [withdraw] = await setupRequestWithdrawal()
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)
        await expect(
          withdraw.connect(signers.user).requestWithdrawalWithEvidence(
            {
              recipient: testAddress1,
              amount: 2n,
              assetId: 0n,
            },
            {
              leaf: {
                withdrawLeaf: {
                  recipient: signers.user.address,
                  amount: withdrawLeafAmounts,
                  startEbn: 16n,
                  endEbn: 15n,
                },
                evidenceLeaf: {
                  transferCommitment:
                    "0x1dc73b3c28607bdd2e8c9fd554516e95a5b636dc2bc8d71401476ce2aa42a11e",
                  ebn: 4n,
                },
              },
              index: 0n,
              siblings: [testHashes[1], testHashes[2]],
            },
            amounts
          )
        )
          .to.be.revertedWithCustomError(withdraw, "EvidenceLeafEbnIsTooOld")
          .withArgs(4n, 10n)
      })
      it("withdrawal request already exists", async () => {
        const [withdraw] = await setupRequestWithdrawal()
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)

        await withdraw.connect(signers.user).requestWithdrawalWithEvidence(
          {
            recipient: testAddress1,
            amount: 2n,
            assetId: 0n,
          },
          {
            leaf: {
              withdrawLeaf: {
                recipient: signers.user.address,
                amount: withdrawLeafAmounts,
                startEbn: 11n,
                endEbn: 15n,
              },
              evidenceLeaf: {
                transferCommitment:
                  "0x1dc73b3c28607bdd2e8c9fd554516e95a5b636dc2bc8d71401476ce2aa42a11e",
                ebn: 20n,
              },
            },
            index: 0n,
            siblings: [testHashes[1], testHashes[2]],
          },
          amounts
        )
        await expect(
          withdraw.connect(signers.user).requestWithdrawalWithEvidence(
            {
              recipient: testAddress1,
              amount: 2n,
              assetId: 0n,
            },
            {
              leaf: {
                withdrawLeaf: {
                  recipient: signers.user.address,
                  amount: withdrawLeafAmounts,
                  startEbn: 11n,
                  endEbn: 15n,
                },
                evidenceLeaf: {
                  transferCommitment:
                    "0x1dc73b3c28607bdd2e8c9fd554516e95a5b636dc2bc8d71401476ce2aa42a11e",
                  ebn: 20n,
                },
              },
              index: 0n,
              siblings: [testHashes[1], testHashes[2]],
            },
            amounts
          )
        )
          .to.be.revertedWithCustomError(
            withdraw,
            "WithdrawalRequestAlreadyExists"
          )
          .withArgs(signers.user.address)
      })
    })
  })

  describe("challengeWithdrawal", () => {
    describe("success", () => {
      it("delete withdraw requests", async () => {
        const withdraw = await setupChallengeWithdrawal()
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)
        const testAddress = generateDummyAddresses(1)
        const withdrawalRequests = await withdraw.withdrawalRequests(
          signers.user.address
        )
        expect(withdrawalRequests.requestedAt).to.not.equal(0)
        await withdraw.connect(signers.operator).challengeWithdrawal(
          signers.user.address,
          {
            payment: {
              uniqueIdentifier: testHashes[0],
              user: testAddress[0],
              round: 0,
              nonce: 0,
              userBalance: {
                amounts: [1n, 2n, 3n, 4n],
              },
              operatorBalance: {
                amounts: [1n, 2n, 3n, 4n],
              },
              airdropped: {
                amounts: [1n, 2n, 3n, 4n],
              },
              spentDeposit: {
                amounts: [1n, 2n, 3n, 4n],
              },
              latestTransferCommitment: testHashes[1],
              latestEbn: 0n,
              customData: "0x",
            },
            userSignature: "0x",
            operatorSignature: "0x",
          },
          {
            leaf: {
              withdrawLeaf: {
                recipient: signers.user.address,
                amount: withdrawLeafAmounts,
                startEbn: 11n,
                endEbn: 15n,
              },
              evidenceLeaf: {
                transferCommitment: testHashes[0],
                ebn: 4n,
              },
            },
            index: 0n,
            siblings: [testHashes[1], testHashes[2]],
          }
        )
        const withdrawalRequestsAfter = await withdraw.withdrawalRequests(
          signers.user.address
        )
        expect(withdrawalRequestsAfter.requestedAt).to.equal(0)
      })
      it("generate WithdrawalChallenged event", async () => {
        const withdraw = await setupChallengeWithdrawal()
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)
        const testAddress = generateDummyAddresses(1)
        await expect(
          withdraw.connect(signers.operator).challengeWithdrawal(
            signers.user.address,
            {
              payment: {
                uniqueIdentifier: testHashes[0],
                user: testAddress[0],
                round: 0,
                nonce: 0,
                userBalance: {
                  amounts: [1n, 2n, 3n, 4n],
                },
                operatorBalance: {
                  amounts: [1n, 2n, 3n, 4n],
                },
                airdropped: {
                  amounts: [1n, 2n, 3n, 4n],
                },
                spentDeposit: {
                  amounts: [1n, 2n, 3n, 4n],
                },
                latestTransferCommitment: testHashes[1],
                latestEbn: 0n,
                customData: "0x",
              },
              userSignature: "0x",
              operatorSignature: "0x",
            },
            {
              leaf: {
                withdrawLeaf: {
                  recipient: signers.user.address,
                  amount: withdrawLeafAmounts,
                  startEbn: 11n,
                  endEbn: 15n,
                },
                evidenceLeaf: {
                  transferCommitment: testHashes[0],
                  ebn: 4n,
                },
              },
              index: 0n,
              siblings: [testHashes[1], testHashes[2]],
            }
          )
        )
          .to.emit(withdraw, "WithdrawalChallenged")
          .withArgs(signers.user.address)
      })
    })
    describe("fail", () => {
      it("only operator", async () => {
        const [withdraw] = await loadFixture(setup)
        const signers = await getSigners()
        const role = await withdraw.OPERATOR()
        const testHashes = generateDummyHashes(3)
        const testAddress = generateDummyAddresses(1)
        await expect(
          withdraw.connect(signers.illegalUser).challengeWithdrawal(
            signers.user.address,
            {
              payment: {
                uniqueIdentifier: testHashes[0],
                user: testAddress[0],
                round: 0,
                nonce: 0,
                userBalance: {
                  amounts: [1n, 2n, 3n, 4n],
                },
                operatorBalance: {
                  amounts: [1n, 2n, 3n, 4n],
                },
                airdropped: {
                  amounts: [1n, 2n, 3n, 4n],
                },
                spentDeposit: {
                  amounts: [1n, 2n, 3n, 4n],
                },
                latestTransferCommitment: testHashes[1],
                latestEbn: 0n,
                customData: "0x",
              },
              userSignature: "0x",
              operatorSignature: "0x",
            },
            {
              leaf: {
                withdrawLeaf: {
                  recipient: signers.user.address,
                  amount: withdrawLeafAmounts,
                  startEbn: 11n,
                  endEbn: 15n,
                },
                evidenceLeaf: {
                  transferCommitment: testHashes[0],
                  ebn: 4n,
                },
              },
              index: 0n,
              siblings: [testHashes[1], testHashes[2]],
            }
          )
        )
          .to.be.revertedWithCustomError(
            withdraw,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, role)
      })
      it("only after withdraw request", async () => {
        const [withdraw] = await loadFixture(setup)
        const signers = await getSigners()
        const testHashes = generateDummyHashes(3)
        const testAddress = generateDummyAddresses(1)
        await expect(
          withdraw.connect(signers.operator).challengeWithdrawal(
            signers.user.address,
            {
              payment: {
                uniqueIdentifier: testHashes[0],
                user: testAddress[0],
                round: 0,
                nonce: 0,
                userBalance: {
                  amounts: [1n, 2n, 3n, 4n],
                },
                operatorBalance: {
                  amounts: [1n, 2n, 3n, 4n],
                },
                airdropped: {
                  amounts: [1n, 2n, 3n, 4n],
                },
                spentDeposit: {
                  amounts: [1n, 2n, 3n, 4n],
                },
                latestTransferCommitment: testHashes[1],
                latestEbn: 0n,
                customData: "0x",
              },
              userSignature: "0x",
              operatorSignature: "0x",
            },
            {
              leaf: {
                withdrawLeaf: {
                  recipient: signers.user.address,
                  amount: withdrawLeafAmounts,
                  startEbn: 11n,
                  endEbn: 15n,
                },
                evidenceLeaf: {
                  transferCommitment: testHashes[0],
                  ebn: 4n,
                },
              },
              index: 0n,
              siblings: [testHashes[1], testHashes[2]],
            }
          )
        )
          .to.be.revertedWithCustomError(withdraw, "WithdrawalRequestNotFound")
          .withArgs(signers.user.address)
      })
    })
  })

  describe("acceptWithdrawal", () => {
    describe("success", () => {
      it("delete withdraw requests", async () => {
        const [withdraw] = await setupAcceptWithdrawal()
        const signers = await getSigners()
        const withdrawalRequests = await withdraw.withdrawalRequests(
          signers.user.address
        )
        expect(withdrawalRequests.requestedAt).to.not.equal(0)
        await withdraw
          .connect(signers.operator)
          .acceptWithdrawal(signers.user.address)
        const withdrawalRequestsAfter = await withdraw.withdrawalRequests(
          signers.user.address
        )
        expect(withdrawalRequestsAfter.requestedAt).to.equal(0)
      })
      it("set new channel state data", async () => {
        const [withdraw, main] = await setupAcceptWithdrawal()
        const signers = await getSigners()
        const state = await main.getChannelState(signers.user.address)
        expect(state.userDeposit.amounts).to.eql([0n, 0n, 0n, 0n])
        expect(state.ebn).to.equal(0n)
        expect(state.round).to.equal(0n)
        await withdraw
          .connect(signers.operator)
          .acceptWithdrawal(signers.user.address)
        const newState = await main.getChannelState(signers.user.address)
        expect(newState.userDeposit.amounts).to.eql(withdrawLeafAmounts.amounts)
        expect(newState.ebn).to.equal(15n)
        expect(newState.round).to.equal(1n)
      })
      it("set airdropped amount", async () => {
        const [withdraw, main] = await setupAcceptWithdrawal(true)
        const signers = await getSigners()
        const state = await main.getChannelState(signers.user.address)
        expect(state.userDeposit.amounts).to.eql([0n, 0n, 0n, 0n])
        expect(state.ebn).to.equal(0n)
        expect(state.round).to.equal(0n)
        await withdraw
          .connect(signers.operator)
          .acceptWithdrawal(signers.user.address)
        const newState = await main.getChannelState(signers.user.address)
        expect(newState.userDeposit.amounts).to.eql(amounts.amounts)
        expect(newState.ebn).to.equal(15n)
        expect(newState.round).to.equal(1n)
      })

      it("call liquidity manager sendAssets func", async () => {
        const [withdraw, , liquidityManager] = await setupAcceptWithdrawal(true)
        const signers = await getSigners()
        expect(await liquidityManager.latestRecipient()).to.equal(
          ethers.ZeroAddress
        )
        expect(await liquidityManager.latestAssets0()).to.equal(0n)
        expect(await liquidityManager.latestAssets1()).to.equal(0n)
        expect(await liquidityManager.latestAssets2()).to.equal(0n)
        expect(await liquidityManager.latestAssets3()).to.equal(0n)
        await withdraw
          .connect(signers.operator)
          .acceptWithdrawal(signers.user.address)
        expect(await liquidityManager.latestRecipient()).to.equal(
          signers.user.address
        )
        expect(await liquidityManager.latestAssets0()).to.equal(5n)
        expect(await liquidityManager.latestAssets1()).to.equal(5n)
        expect(await liquidityManager.latestAssets2()).to.equal(5n)
        expect(await liquidityManager.latestAssets3()).to.equal(5n)
      })

      it("generate WithdrawalAccepted event", async () => {
        const [withdraw] = await setupAcceptWithdrawal(true)
        const signers = await getSigners()
        await expect(
          withdraw
            .connect(signers.operator)
            .acceptWithdrawal(signers.user.address)
        )
          .to.emit(withdraw, "WithdrawalAccepted")
          .withArgs(signers.user.address)
      })
    })
    describe("fail", () => {
      it("only operator", async () => {
        const [withdraw] = await loadFixture(setup)
        const signers = await getSigners()
        const role = await withdraw.OPERATOR()
        await expect(
          withdraw
            .connect(signers.illegalUser)
            .acceptWithdrawal(signers.user.address)
        )
          .to.be.revertedWithCustomError(
            withdraw,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, role)
      })
      it("only after withdraw request", async () => {
        const [withdraw] = await loadFixture(setup)
        const signers = await getSigners()
        await expect(
          withdraw
            .connect(signers.operator)
            .acceptWithdrawal(signers.user.address)
        )
          .to.be.revertedWithCustomError(withdraw, "WithdrawalRequestNotFound")
          .withArgs(signers.user.address)
      })
    })
  })

  describe("timeOutWithdrawal", () => {
    describe("success", () => {
      it("delete withdraw requests", async () => {
        const [withdraw] = await setupAcceptWithdrawal()
        const signers = await getSigners()
        const withdrawalRequests = await withdraw.withdrawalRequests(
          signers.user.address
        )
        expect(withdrawalRequests.requestedAt).to.not.equal(0)
        await time.increase(86400 * 3)
        await withdraw
          .connect(signers.operator)
          .timeOutWithdrawal(signers.user.address)
        const withdrawalRequestsAfter = await withdraw.withdrawalRequests(
          signers.user.address
        )
        expect(withdrawalRequestsAfter.requestedAt).to.equal(0)
      })
      it("set new channel state data", async () => {
        const [withdraw, main] = await setupAcceptWithdrawal()
        const signers = await getSigners()
        const state = await main.getChannelState(signers.user.address)
        expect(state.userDeposit.amounts).to.eql([0n, 0n, 0n, 0n])
        expect(state.ebn).to.equal(0n)
        expect(state.round).to.equal(0n)
        await time.increase(86400 * 3)
        await withdraw
          .connect(signers.operator)
          .timeOutWithdrawal(signers.user.address)
        const newState = await main.getChannelState(signers.user.address)
        expect(newState.userDeposit.amounts).to.eql(withdrawLeafAmounts.amounts)
        expect(newState.ebn).to.equal(15n)
        expect(newState.round).to.equal(1n)
      })
      it("set airdropped amount", async () => {
        const [withdraw, main] = await setupAcceptWithdrawal(true)
        const signers = await getSigners()
        const state = await main.getChannelState(signers.user.address)
        expect(state.userDeposit.amounts).to.eql([0n, 0n, 0n, 0n])
        expect(state.ebn).to.equal(0n)
        expect(state.round).to.equal(0n)
        await time.increase(86400 * 3)
        await withdraw
          .connect(signers.operator)
          .timeOutWithdrawal(signers.user.address)
        const newState = await main.getChannelState(signers.user.address)
        expect(newState.userDeposit.amounts).to.eql(amounts.amounts)
        expect(newState.ebn).to.equal(15n)
        expect(newState.round).to.equal(1n)
      })

      it("call liquidity manager sendAssets func", async () => {
        const [withdraw, , liquidityManager] = await setupAcceptWithdrawal(true)
        const signers = await getSigners()
        expect(await liquidityManager.latestRecipient()).to.equal(
          ethers.ZeroAddress
        )
        expect(await liquidityManager.latestAssets0()).to.equal(0n)
        expect(await liquidityManager.latestAssets1()).to.equal(0n)
        expect(await liquidityManager.latestAssets2()).to.equal(0n)
        expect(await liquidityManager.latestAssets3()).to.equal(0n)
        await time.increase(86400 * 3)
        await withdraw
          .connect(signers.operator)
          .timeOutWithdrawal(signers.user.address)
        expect(await liquidityManager.latestRecipient()).to.equal(
          signers.user.address
        )
        expect(await liquidityManager.latestAssets0()).to.equal(5n)
        expect(await liquidityManager.latestAssets1()).to.equal(5n)
        expect(await liquidityManager.latestAssets2()).to.equal(5n)
        expect(await liquidityManager.latestAssets3()).to.equal(5n)
      })

      it("generate WithdrawalTimeOuted event", async () => {
        const [withdraw] = await setupAcceptWithdrawal(true)
        const signers = await getSigners()
        await time.increase(86400 * 3)
        await expect(
          withdraw
            .connect(signers.operator)
            .timeOutWithdrawal(signers.user.address)
        )
          .to.emit(withdraw, "WithdrawalTimeOuted")
          .withArgs(signers.user.address)
      })
    })
    describe("fail", () => {
      it("certain time has not passed", async () => {
        const [withdraw] = await setupAcceptWithdrawal()
        const signers = await getSigners()
        const currentTime = await time.latest()
        const withdrawalRequests = await withdraw.withdrawalRequests(
          signers.user.address
        )
        const target = Number(withdrawalRequests.requestedAt) + 86400 * 3
        await expect(
          withdraw
            .connect(signers.illegalUser)
            .timeOutWithdrawal(signers.user.address)
        )
          .to.be.revertedWithCustomError(withdraw, "TimeOutIsNotReached")
          .withArgs(target, currentTime + 1)
      })
      it("only after withdraw request", async () => {
        const [withdraw] = await loadFixture(setup)
        const signers = await getSigners()
        await expect(
          withdraw
            .connect(signers.operator)
            .acceptWithdrawal(signers.user.address)
        )
          .to.be.revertedWithCustomError(withdraw, "WithdrawalRequestNotFound")
          .withArgs(signers.user.address)
      })
    })
  })

  describe("upgrade", () => {
    it("contract is upgradable", async () => {
      const signers = await getSigners()
      const withdraw = await getUupsContract<Withdraw>("Withdraw", [
        signers.dummyConfig.address,
      ])
      const role = await withdraw.DEFAULT_ADMIN_ROLE()
      const result = await withdraw.hasRole(role, signers.dummyConfig.address)
      expect(result).to.equal(true)

      const factory = await ethers.getContractFactory(
        "TestWithdraw2",
        signers.dummyConfig
      )
      const next = await upgrades.upgradeProxy(
        await withdraw.getAddress(),
        factory
      )
      const result2 = await next.hasRole(role, signers.dummyConfig.address)
      expect(result).to.equal(result2)
      const val: number = (await next.getVal()) as number
      expect(val).to.equal(4)
    })
    it("Cannot upgrade except for a deployer.", async () => {
      const [withdraw] = await loadFixture(setup)
      const signers = await getSigners()
      const factory = await ethers.getContractFactory(
        "TestWithdraw2",
        signers.illegalUser
      )
      const role = await withdraw.DEFAULT_ADMIN_ROLE()
      await expect(upgrades.upgradeProxy(await withdraw.getAddress(), factory))
        .to.be.revertedWithCustomError(
          withdraw,
          "AccessControlUnauthorizedAccount"
        )
        .withArgs(signers.illegalUser.address, role)
    })
  })
})

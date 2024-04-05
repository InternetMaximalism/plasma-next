import { expect } from "chai"
import { ethers, upgrades } from "hardhat"
import { loadFixture } from "@nomicfoundation/hardhat-toolbox/network-helpers"
import {
  getSigners,
  deployAllContracts,
  generateDummyAddresses,
  getUupsContract,
  generateDummyHashes,
  testHash1,
  testHash2,
  testHash3,
} from "../test-utils"
import {
  Config,
  BlockManager,
  TestBlockManager2,
  TestBlockManager4,
} from "../../typechain-types"
import {
  BlockPostedEvent,
  DepositedEvent,
} from "../../typechain-types/contracts/block-manager/BlockManager"
import { IConfig } from "../../typechain-types/contracts/config"
import { TypedContractEvent, TypedEventLog } from "../../typechain-types/common"

describe("BlockManager", () => {
  const setup = async (): Promise<[BlockManager, Config]> => {
    const config = await deployAllContracts()
    const addressBook = await config.getAddressBook()
    const testBlockManagerFactory = await ethers.getContractFactory(
      "BlockManager"
    )
    const blockManager = testBlockManagerFactory.attach(
      addressBook.blockManager
    )
    return [blockManager as BlockManager, config]
  }
  const prepareLiquidityDeposit = async (
    addressBook: IConfig.AddressBookStructOutput
  ): Promise<void> => {
    const signers = await getSigners()
    await Promise.all(
      addressBook.tokenAddresses.addresses.map(async (tokenAddress: string) => {
        const token = await ethers.getContractAt("IERC20", tokenAddress)
        await token.transfer(signers.operator.address, 100n)
        await token
          .connect(signers.operator)
          .approve(addressBook.liquidityManager, 100n)
      })
    )
  }
  const getTokenBalances = async (
    tokenAddresses: string[],
    targetAddress: string
  ): Promise<bigint[]> => {
    return await Promise.all(
      tokenAddresses.map(async (tokenAddress: string) => {
        const token = await ethers.getContractAt("IERC20", tokenAddress)
        return token.balanceOf(targetAddress)
      })
    )
  }
  const getDepositedEvents = async (
    blockManager: BlockManager
  ): Promise<
    TypedEventLog<
      TypedContractEvent<
        DepositedEvent.InputTuple,
        DepositedEvent.OutputTuple,
        DepositedEvent.OutputObject
      >
    >[]
  > => {
    const events = await blockManager.queryFilter(
      blockManager.filters.Deposited()
    )
    return events
  }

  const getLatestDepositedEvent = async (
    blockManager: BlockManager
  ): Promise<
    TypedEventLog<
      TypedContractEvent<
        DepositedEvent.InputTuple,
        DepositedEvent.OutputTuple,
        DepositedEvent.OutputObject
      >
    >
  > => {
    const events = await getDepositedEvents(blockManager)
    return events[events.length - 1]
  }

  const getBlockPostedEvents = async (
    blockManager: BlockManager
  ): Promise<
    TypedEventLog<
      TypedContractEvent<
        BlockPostedEvent.InputTuple,
        BlockPostedEvent.OutputTuple,
        BlockPostedEvent.OutputObject
      >
    >[]
  > => {
    const events = await blockManager.queryFilter(
      blockManager.filters.BlockPosted()
    )
    return events
  }

  const getLatestBlockPostedEvent = async (
    blockManager: BlockManager
  ): Promise<
    TypedEventLog<
      TypedContractEvent<
        BlockPostedEvent.InputTuple,
        BlockPostedEvent.OutputTuple,
        BlockPostedEvent.OutputObject
      >
    >
  > => {
    const events = await getBlockPostedEvents(blockManager)
    return events[events.length - 1]
  }
  const totalDepositHash =
    "0x012893657d8eb2efad4de0a91bcd0e39ad9837745dec3ea923737ea803fc8e3d"
  const firstBlockHash =
    "0x84f88760a49416f03fe86c18e40d58966031fb51e81e55021e0c592f25060cb3"
  describe("initialize", () => {
    describe("success", () => {
      it("initialize was called", async () => {
        const [blockManager, config] = await loadFixture(setup)
        const result = await blockManager.hasRole(
          await blockManager.DEFAULT_ADMIN_ROLE(),
          await config.getAddress()
        )
        expect(result).to.equal(true)
      })
    })
    describe("fail", () => {
      it("Initialization can only be done once", async () => {
        const [blockManager] = await loadFixture(setup)
        const addresses = generateDummyAddresses(1)

        await expect(
          blockManager.initialize(addresses[0])
        ).to.be.revertedWithCustomError(blockManager, "InvalidInitialization")
      })
    })
  })

  describe("config", () => {
    describe("success", () => {
      it("set address info", async () => {
        const testBlockManagerFactory = await ethers.getContractFactory(
          "BlockManager"
        )
        const blockManager = await testBlockManagerFactory.deploy()
        const signers = await getSigners()
        await blockManager.initialize(signers.dummyConfig.address)
        const addresses = generateDummyAddresses(2)
        const role = await blockManager.OPERATOR()

        expect(await blockManager.hasRole(role, addresses[0])).to.equal(false)
        expect(await blockManager.liquidityManagerAddress()).to.not.equal(
          addresses[1]
        )

        await blockManager
          .connect(signers.dummyConfig)
          .config(addresses[0], addresses[1])

        expect(await blockManager.hasRole(role, addresses[0])).to.equal(true)
        expect(await blockManager.liquidityManagerAddress()).to.equal(
          addresses[1]
        )
      })
      it("post genesis block", async () => {
        const testBlockManagerFactory = await ethers.getContractFactory(
          "BlockManager"
        )
        const blockManager = await testBlockManagerFactory.deploy()
        const signers = await getSigners()
        await blockManager.initialize(signers.dummyConfig.address)
        const addresses = generateDummyAddresses(2)

        expect(await blockManager.totalDepositHash()).to.equal(ethers.ZeroHash)
        expect(await blockManager.lastBlockNumber()).to.equal(0)
        expect(await blockManager.lastBlockHash()).to.equal(ethers.ZeroHash)

        await blockManager
          .connect(signers.dummyConfig)
          .config(addresses[0], addresses[1])

        expect(await blockManager.totalDepositHash()).to.equal(totalDepositHash)
        expect(await blockManager.lastBlockNumber()).to.equal(0)
        expect(await blockManager.lastBlockHash()).to.equal(firstBlockHash)

        const blockPostedEvent = await getLatestBlockPostedEvent(blockManager)
        expect(blockPostedEvent.args.blockNumber).to.equal(0)
        expect(blockPostedEvent.args.prevBlockHash).to.equal(ethers.ZeroHash)
        expect(blockPostedEvent.args.transferRoot).to.equal(ethers.ZeroHash)
        expect(blockPostedEvent.args.totalDepositHash).to.equal(
          totalDepositHash
        )

        const depositedEvent = await getLatestDepositedEvent(blockManager)
        expect(depositedEvent.args.deposit).to.eql([[0n, 0n, 0n, 0n]])
        expect(depositedEvent.args.totalDeposit).to.eql([[0n, 0n, 0n, 0n]])
        expect(depositedEvent.args.totalDepositHash).to.equal(totalDepositHash)
      })
      it("if blocks is already set, events is not generated", async () => {
        const testBlockManagerFactory = await ethers.getContractFactory(
          "BlockManager"
        )
        const blockManager = await testBlockManagerFactory.deploy()
        const signers = await getSigners()
        await blockManager.initialize(signers.dummyConfig.address)
        const addresses = generateDummyAddresses(3)
        await blockManager
          .connect(signers.dummyConfig)
          .config(addresses[0], addresses[1])
        await blockManager
          .connect(signers.dummyConfig)
          .config(addresses[0], addresses[1])

        const blockPostedEvents = await getBlockPostedEvents(blockManager)
        expect(blockPostedEvents.length).to.equal(1)

        const depositedEvents = await getDepositedEvents(blockManager)
        expect(depositedEvents.length).to.equal(1)
      })
    })
    describe("fail", () => {
      it("only admin", async () => {
        const [blockManager] = await loadFixture(setup)
        const signers = await getSigners()
        const role = await blockManager.DEFAULT_ADMIN_ROLE()
        const addresses = generateDummyAddresses(3)
        await expect(
          blockManager
            .connect(signers.illegalUser)
            .config(addresses[0], addresses[1])
        )
          .to.be.revertedWithCustomError(
            blockManager,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, role)
      })
    })
  })

  describe("deposit", () => {
    describe("success", () => {
      it("update deposit info", async () => {
        const [blockManager, config] = await loadFixture(setup)
        const signers = await getSigners()
        const addressBook = await config.getAddressBook()
        await prepareLiquidityDeposit(addressBook)
        const totalDepositHashBefore = await blockManager.totalDepositHash()
        expect(totalDepositHashBefore).to.equal(totalDepositHash)

        await blockManager.connect(signers.operator).deposit({
          amounts: [1n, 2n, 3n, 4n],
        })
        const totalDepositHashAfter = await blockManager.totalDepositHash()
        expect(totalDepositHashAfter).to.eql(
          "0x392791df626408017a264f53fde61065d5a93a32b60171df9d8a46afdf82992d"
        )
      })
      it("receive assets", async () => {
        const [blockManager, config] = await loadFixture(setup)
        const signers = await getSigners()
        const addressBook = await config.getAddressBook()
        await prepareLiquidityDeposit(addressBook)
        const liquidityManagerBalances = await getTokenBalances(
          addressBook.tokenAddresses.addresses,
          addressBook.liquidityManager
        )
        expect(liquidityManagerBalances).to.eql([0n, 0n, 0n, 0n])
        const operatorBalances = await getTokenBalances(
          addressBook.tokenAddresses.addresses,
          signers.operator.address
        )
        expect(operatorBalances).to.eql([100n, 100n, 100n, 100n])

        await blockManager.connect(signers.operator).deposit({
          amounts: [1n, 2n, 3n, 4n],
        })

        const liquidityManagerBalancesAfter = await getTokenBalances(
          addressBook.tokenAddresses.addresses,
          addressBook.liquidityManager
        )
        expect(liquidityManagerBalancesAfter).to.eql([1n, 2n, 3n, 4n])
        const operatorBalancesAfter = await getTokenBalances(
          addressBook.tokenAddresses.addresses,
          signers.operator.address
        )
        expect(operatorBalancesAfter).to.eql([99n, 98n, 97n, 96n])
      })
      it("generate Deposited event", async () => {
        const [blockManager, config] = await loadFixture(setup)
        const signers = await getSigners()
        const addressBook = await config.getAddressBook()
        await prepareLiquidityDeposit(addressBook)
        await blockManager.connect(signers.operator).deposit({
          amounts: [1n, 2n, 3n, 4n],
        })
        const depositedEvent = await getLatestDepositedEvent(blockManager)
        expect(depositedEvent.args.deposit).to.eql([[1n, 2n, 3n, 4n]])
        expect(depositedEvent.args.totalDeposit).to.eql([[1n, 2n, 3n, 4n]])
        expect(depositedEvent.args.totalDepositHash).to.equal(
          "0x392791df626408017a264f53fde61065d5a93a32b60171df9d8a46afdf82992d"
        )
      })
      it("add amount", async () => {
        const [blockManager, config] = await loadFixture(setup)
        const signers = await getSigners()
        const addressBook = await config.getAddressBook()
        await prepareLiquidityDeposit(addressBook)
        await blockManager.connect(signers.operator).deposit({
          amounts: [1n, 2n, 3n, 4n],
        })
        await blockManager.connect(signers.operator).deposit({
          amounts: [5n, 6n, 7n, 8n],
        })
        const depositedEvent = await getLatestDepositedEvent(blockManager)
        expect(depositedEvent.args.deposit).to.eql([[5n, 6n, 7n, 8n]])
        expect(depositedEvent.args.totalDeposit).to.eql([[6n, 8n, 10n, 12n]])
      })
    })
    describe("fail", () => {
      it("only operator", async () => {
        const [blockManager] = await loadFixture(setup)
        const signers = await getSigners()
        const role = await blockManager.OPERATOR()
        await expect(
          blockManager.connect(signers.illegalUser).deposit({
            amounts: [1n, 2n, 3n, 4n],
          })
        )
          .to.be.revertedWithCustomError(
            blockManager,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, role)
      })
    })
  })

  describe("postBlocks", () => {
    describe("success", () => {
      it("update block info", async () => {
        const [blockManager] = await loadFixture(setup)
        const signers = await getSigners()
        const beforeLastBlockNumber = await blockManager.lastBlockNumber()
        expect(beforeLastBlockNumber).to.equal(0)
        const beforeLastBlockHash = await blockManager.lastBlockHash()
        expect(beforeLastBlockHash).to.equal(firstBlockHash)

        await blockManager.connect(signers.operator).postBlocks([testHash1])

        const afterLastBlockNumber = await blockManager.lastBlockNumber()
        expect(afterLastBlockNumber).to.equal(1)
        const afterLastBlockHash = await blockManager.lastBlockHash()
        expect(afterLastBlockHash).to.equal(
          "0xc8f07085227e109715b142e469841838b0078229e77d212435023bb4f5c57bdd"
        )
      })
      it("update check point", async () => {
        const [blockManager] = await loadFixture(setup)
        const testHash = generateDummyHashes(1)[0]
        const signers = await getSigners()

        const beforeLastCheckpointBlockNumber =
          await blockManager.getLastCheckpointBlockNumber()
        expect(beforeLastCheckpointBlockNumber).to.equal(0)

        const interval = await blockManager.BLOCK_HASH_CHECKPOINT_INTERVAL()
        for (let i = 0; i < interval; i++) {
          await blockManager.connect(signers.operator).postBlocks([testHash])
        }
        const lastCheckPointHash = await blockManager.blockHashCheckpoints(0)
        const lastHash = await blockManager.lastBlockHash()
        expect(lastHash).to.equal(lastCheckPointHash)
        const afterLastCheckpointBlockNumber =
          await blockManager.getLastCheckpointBlockNumber()
        expect(afterLastCheckpointBlockNumber).to.equal(interval)
      })
      it("generate BlockPosted event", async () => {
        const [blockManager] = await loadFixture(setup)
        const signers = await getSigners()
        const testHash = generateDummyHashes(1)[0]

        const beforeEvents = await getBlockPostedEvents(blockManager)
        expect(beforeEvents.length).to.equal(1)

        await blockManager.connect(signers.operator).postBlocks([testHash])
        const afterEvents = await getBlockPostedEvents(blockManager)
        expect(afterEvents.length).to.equal(2)

        const latestEvent = afterEvents[afterEvents.length - 1]
        expect(latestEvent.args.blockNumber).to.equal(1)
        expect(latestEvent.args.prevBlockHash).to.equal(firstBlockHash)
        expect(latestEvent.args.transferRoot).to.equal(testHash)
        expect(latestEvent.args.totalDepositHash).to.equal(totalDepositHash)
      })
      it("generate BlockPosted events", async () => {
        const [blockManager] = await loadFixture(setup)
        const signers = await getSigners()
        const testHashes = generateDummyHashes(2)

        const beforeEvents = await getBlockPostedEvents(blockManager)
        expect(beforeEvents.length).to.equal(1)

        await blockManager
          .connect(signers.operator)
          .postBlocks([testHashes[0], testHashes[1]])
        const afterEvents = await getBlockPostedEvents(blockManager)
        expect(afterEvents.length).to.equal(3)
      })
    })
    describe("fail", () => {
      it("only operator", async () => {
        const [blockManager] = await loadFixture(setup)
        const signers = await getSigners()
        const testHashes = generateDummyHashes(2)
        const role = await blockManager.OPERATOR()
        await expect(
          blockManager
            .connect(signers.illegalUser)
            .postBlocks([testHashes[0], testHashes[1]])
        )
          .to.be.revertedWithCustomError(
            blockManager,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, role)
      })
    })
  })

  describe("getBlockHashIfAvailable", () => {
    describe("success", () => {
      it("return last block hash", async () => {
        const signers = await getSigners()
        const blockManager = await getUupsContract<TestBlockManager2>(
          "TestBlockManager2",
          [signers.dummyConfig.address]
        )

        const testBlockNumber = 10
        const testHash = generateDummyHashes(1)[0]
        await blockManager.setLastBlockNumber(testBlockNumber)
        await blockManager.setLastBlockHash(testHash)
        const result = await blockManager.testGetBlockHashIfAvailable(
          testBlockNumber
        )
        expect(result).to.equal(testHash)
      })
      it("get checkpoint hash", async () => {
        const signers = await getSigners()
        const blockManager = await getUupsContract<TestBlockManager2>(
          "TestBlockManager2",
          [signers.dummyConfig.address]
        )
        const interval = await blockManager.BLOCK_HASH_CHECKPOINT_INTERVAL()
        const testBlockNumber = interval
        const testHashes = generateDummyHashes(2)
        await blockManager.setBlockHashCheckpoints(testHashes)
        const result = await blockManager.testGetBlockHashIfAvailable(
          testBlockNumber
        )
        expect(result).to.equal(testHashes[1])
      })
    })
    describe("fail", () => {
      it("block hash not available", async () => {
        const signers = await getSigners()
        const blockManager = await getUupsContract<TestBlockManager2>(
          "TestBlockManager2",
          [signers.dummyConfig.address]
        )
        const interval = await blockManager.BLOCK_HASH_CHECKPOINT_INTERVAL()
        const testBlockNumber = 35n
        const testHashes = generateDummyHashes(2)
        await blockManager.setBlockHashCheckpoints(testHashes)
        await blockManager.setLastBlockNumber(interval * 2n)
        await expect(blockManager.testGetBlockHashIfAvailable(testBlockNumber))
          .to.be.revertedWithCustomError(blockManager, "BlockHashNotAvailable")
          .withArgs(testBlockNumber, interval * 2n, testBlockNumber / interval)
      })
    })
  })

  describe("computeHashChain", () => {
    it("return last block hash", async () => {
      const signers = await getSigners()
      const blockManager = await getUupsContract<TestBlockManager2>(
        "TestBlockManager2",
        [signers.dummyConfig.address]
      )
      const startBlockNumber = 10
      const transferRoots = [testHash2, testHash3]
      const result = await blockManager.testComputeHashChain(
        startBlockNumber,
        testHash1,
        transferRoots,
        [testHash2, testHash3]
      )
      expect(result[0]).to.equal(startBlockNumber + transferRoots.length)
      expect(result[1]).to.equal(
        "0x4dcb1caf29fd90297c849d203a1454ef71be63fcb4e8d9bf3f2f2887dcf7429b"
      )
    })
  })
  describe("verifyInclusion", () => {
    describe("success", () => {
      it("verify", async () => {
        const signers = await getSigners()
        const blockManager = await getUupsContract<TestBlockManager4>(
          "TestBlockManager4",
          [signers.dummyConfig.address]
        )
        await blockManager.setBlockHashIfAvailableResult(
          "0x4dcb1caf29fd90297c849d203a1454ef71be63fcb4e8d9bf3f2f2887dcf7429b"
        )
        await blockManager.verifyInclusion(
          10,
          testHash1,
          [testHash2, testHash3],
          [testHash2, testHash3]
        )
        expect(true).to.equal(true)
      })
    })
    describe("fail", () => {
      it("not verify", async () => {
        const signers = await getSigners()
        const blockManager = await getUupsContract<TestBlockManager4>(
          "TestBlockManager4",
          [signers.dummyConfig.address]
        )
        await blockManager.setBlockHashIfAvailableResult(
          "0x9d43fa375dff5d1486e61bff7b7a5f059c09b561786d08bd1b7098bb13a70042"
        )
        await expect(
          blockManager.verifyInclusion(
            10,
            testHash1,
            [testHash2, testHash3],
            [testHash2, testHash3]
          )
        )
          .to.be.revertedWithCustomError(blockManager, "VerifyInclusionFailed")
          .withArgs(
            12,
            "0x4dcb1caf29fd90297c849d203a1454ef71be63fcb4e8d9bf3f2f2887dcf7429b",
            "0x9d43fa375dff5d1486e61bff7b7a5f059c09b561786d08bd1b7098bb13a70042"
          )
      })
    })
  })

  describe("upgrade", () => {
    it("contract is upgradable", async () => {
      const signers = await getSigners()
      const blockManager = await getUupsContract<BlockManager>("BlockManager", [
        signers.dummyConfig.address,
      ])
      const role = await blockManager.DEFAULT_ADMIN_ROLE()
      const result = await blockManager.hasRole(
        role,
        signers.dummyConfig.address
      )
      expect(result).to.equal(true)

      const factory = await ethers.getContractFactory(
        "TestBlockManager2",
        signers.dummyConfig
      )
      const next = await upgrades.upgradeProxy(
        await blockManager.getAddress(),
        factory
      )
      const result2 = await next.hasRole(role, signers.dummyConfig.address)
      expect(result).to.equal(result2)
      const val: number = (await next.getVal()) as number
      expect(val).to.equal(7)
    })
    it("Cannot upgrade except for a deployer.", async () => {
      const [blockManager] = await loadFixture(setup)
      const signers = await getSigners()
      const factory = await ethers.getContractFactory(
        "TestBlockManager2",
        signers.illegalUser
      )
      const role = await blockManager.DEFAULT_ADMIN_ROLE()
      await expect(
        upgrades.upgradeProxy(await blockManager.getAddress(), factory)
      )
        .to.be.revertedWithCustomError(
          blockManager,
          "AccessControlUnauthorizedAccount"
        )
        .withArgs(signers.illegalUser.address, role)
    })
  })
})

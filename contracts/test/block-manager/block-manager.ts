import { expect } from "chai"
import { ethers, upgrades } from "hardhat"
import { loadFixture } from "@nomicfoundation/hardhat-toolbox/network-helpers"
import {
  getSigners,
  deployAllContracts,
  generateDummyAddresses,
  getUupsContract,
  generateDummyHashes,
} from "../test-utils"
import { Config, BlockManager } from "../../typechain-types"
import { BlockPostedEvent } from "../../typechain-types/contracts/block-manager/BlockManager"
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

  const getLatestBlockPostedEvents = async (
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
  const genesisBlockHash =
    "0x893b06229450f956b7bd800d3f20f4298344a010ed94f9c767f9374cf4004513"
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
        expect(await blockManager.operator()).to.not.equal(addresses[0])
        expect(await blockManager.liquidityManagerAddress()).to.not.equal(
          addresses[1]
        )

        await blockManager
          .connect(signers.dummyConfig)
          .config(addresses[0], addresses[1])

        expect(await blockManager.hasRole(role, addresses[0])).to.equal(true)
        expect(await blockManager.operator()).to.equal(addresses[0])
        expect(await blockManager.liquidityManagerAddress()).to.equal(
          addresses[1]
        )
      })
      it("generate block info event", async () => {
        const testBlockManagerFactory = await ethers.getContractFactory(
          "BlockManager"
        )
        const blockManager = await testBlockManagerFactory.deploy()
        const signers = await getSigners()
        await blockManager.initialize(signers.dummyConfig.address)
        const addresses = generateDummyAddresses(2)

        await blockManager
          .connect(signers.dummyConfig)
          .config(addresses[0], addresses[1])

        expect(await blockManager.blocks(0)).to.equal(genesisBlockHash)
        const event = await getLatestBlockPostedEvents(blockManager)
        expect(event.args.blockNumber).to.equal(0)
        expect(event.args.blockHash).to.equal(genesisBlockHash)
        expect(event.args.prevBlockHash).to.equal(ethers.ZeroHash)
        expect(event.args.transferRoot).to.equal(ethers.ZeroHash)
        expect(event.args.totalDeposit.amounts).to.eql([0n, 0n, 0n, 0n])
      })
      it("if blocks is already set, block info is not generated", async () => {
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

        const events = await blockManager.queryFilter(
          blockManager.filters.BlockPosted()
        )
        expect(events.length).to.equal(1)
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
  describe("getLatetBlockNumber", () => {
    it("get latest block number", async () => {
      const [blockManager] = await loadFixture(setup)
      const latestBlockNumber = await blockManager.getLatestBlockNumber()
      expect(latestBlockNumber).to.equal(0)
    })
  })
  describe("getBlockHash", () => {
    describe("success", () => {
      it("get block hash", async () => {
        const [blockManager] = await loadFixture(setup)
        const blockHash = await blockManager.getBlockHash(0)
        expect(blockHash).to.equal(genesisBlockHash)
      })
    })
    describe("fail", () => {
      it("get block hash", async () => {
        const [blockManager] = await loadFixture(setup)
        await expect(blockManager.getBlockHash(1))
          .to.be.revertedWithCustomError(blockManager, "BlockNumberTooBig")
          .withArgs(1, 0)
      })
    })
  })
  describe("depositAndPostBlocks", () => {
    describe("success", () => {
      it("add operator deposit", async () => {
        const [blockManager, config] = await loadFixture(setup)
        const testHash = generateDummyHashes(1)[0]
        const signers = await getSigners()
        const event = await getLatestBlockPostedEvents(blockManager)
        const addressBook = await config.getAddressBook()
        await prepareLiquidityDeposit(addressBook)
        expect(event.args.totalDeposit.amounts).to.eql([0n, 0n, 0n, 0n])

        await blockManager
          .connect(signers.operator)
          .depositAndPostBlocks([testHash], {
            amounts: [1n, 2n, 3n, 4n],
          })

        const latestEvent = await getLatestBlockPostedEvents(blockManager)
        expect(latestEvent.args.totalDeposit.amounts).to.eql([1n, 2n, 3n, 4n])
      })
      it("receive assets", async () => {
        const [blockManager, config] = await loadFixture(setup)
        const signers = await getSigners()
        const addressBook = await config.getAddressBook()
        const testHash = generateDummyHashes(1)[0]
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

        await blockManager
          .connect(signers.operator)
          .depositAndPostBlocks([testHash], {
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
        const testHash = generateDummyHashes(1)[0]
        const addressBook = await config.getAddressBook()
        await prepareLiquidityDeposit(addressBook)
        await blockManager
          .connect(signers.operator)
          .depositAndPostBlocks([testHash], {
            amounts: [1n, 2n, 3n, 4n],
          })
        const event = (
          await blockManager.queryFilter(blockManager.filters.Deposited())
        )[0]
        expect(event.args.assets.amounts).to.eql([1n, 2n, 3n, 4n])
      })
      it("generate BlockPosted event", async () => {
        const [blockManager, config] = await loadFixture(setup)
        const signers = await getSigners()
        const addressBook = await config.getAddressBook()
        const testHashes = generateDummyHashes(2)
        await prepareLiquidityDeposit(addressBook)
        await blockManager
          .connect(signers.operator)
          .depositAndPostBlocks([testHashes[0], testHashes[1]], {
            amounts: [1n, 2n, 3n, 4n],
          })
        const event = await getBlockPostedEvents(blockManager)
        expect(event.length).to.equal(3)
        const blockHashIndex1 = await blockManager.blocks(1)
        const blockHashIndex2 = await blockManager.blocks(2)
        expect(event[1].args.blockNumber).to.equal(1)
        expect(event[1].args.blockHash).to.equal(blockHashIndex1)
        expect(event[1].args.prevBlockHash).to.equal(genesisBlockHash)
        expect(event[1].args.transferRoot).to.equal(testHashes[0])
        expect(event[1].args.totalDeposit.amounts).to.eql([1n, 2n, 3n, 4n])
        expect(event[2].args.blockNumber).to.equal(2)
        expect(event[2].args.blockHash).to.equal(blockHashIndex2)
        expect(event[2].args.prevBlockHash).to.equal(blockHashIndex1)
        expect(event[2].args.transferRoot).to.equal(testHashes[1])
        expect(event[2].args.totalDeposit.amounts).to.eql([1n, 2n, 3n, 4n])
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
            .depositAndPostBlocks([testHashes[0], testHashes[1]], {
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

import { expect } from "chai"
import { ethers, upgrades } from "hardhat"
import { loadFixture } from "@nomicfoundation/hardhat-toolbox/network-helpers"
import {
  getSigners,
  deployAllContracts,
  generateDummyAddresses,
} from "../test-utils"
import {
  Config,
  Verifier,
  RootManager,
  BlockManager,
  LiquidityManager,
  Main,
  Withdraw,
} from "../../typechain-types"

describe("Config", () => {
  const setup = async (): Promise<Config> => {
    const config = await deployAllContracts()
    return config
  }
  describe("initialize", () => {
    describe("success", () => {
      it("initialize was called", async () => {
        const config = await loadFixture(setup)
        const signers = await getSigners()
        const result = await config.hasRole(
          await config.DEPLOYER(),
          signers.deployer.address
        )
        expect(result).to.equal(true)
      })
    })
    describe("fail", () => {
      it("Initialization can only be done once", async () => {
        const config = await loadFixture(setup)
        await expect(config.initialize()).to.be.revertedWithCustomError(
          config,
          "InvalidInitialization"
        )
      })
    })
  })

  describe("configure", () => {
    describe("success", () => {
      it("Various contract config functions are running", async () => {
        const config = await loadFixture(setup)
        const addressBook = await config.getAddressBook()
        // Verifier
        const verifier = (await ethers.getContractFactory("Verifier")).attach(
          addressBook.verifier
        ) as Verifier
        expect(await verifier.halo2VerifyingKeyAddress()).to.equal(
          addressBook.halo2VerifyingKeyAddress
        )
        // RootManager
        const rootManager = (
          await ethers.getContractFactory("RootManager")
        ).attach(addressBook.rootManager) as RootManager
        expect(
          await rootManager.hasRole(
            await rootManager.DEFAULT_ADMIN_ROLE(),
            await config.getAddress()
          )
        ).to.equal(true)
        // BlockManager
        const blockManager = (
          await ethers.getContractFactory("BlockManager")
        ).attach(addressBook.blockManager) as BlockManager
        expect(
          await blockManager.hasRole(
            await blockManager.DEFAULT_ADMIN_ROLE(),
            await config.getAddress()
          )
        ).to.equal(true)
        // LiquidityManager
        const liquidityManager = (
          await ethers.getContractFactory("LiquidityManager")
        ).attach(addressBook.liquidityManager) as LiquidityManager
        expect(
          await liquidityManager.hasRole(
            await liquidityManager.DEFAULT_ADMIN_ROLE(),
            await config.getAddress()
          )
        ).to.equal(true)
        // Main
        const main = (await ethers.getContractFactory("Main")).attach(
          addressBook.main
        ) as Main
        expect(
          await main.hasRole(
            await main.DEFAULT_ADMIN_ROLE(),
            await config.getAddress()
          )
        ).to.equal(true)
        // Withdraw
        const withdraw = (await ethers.getContractFactory("Withdraw")).attach(
          addressBook.withdraw
        ) as Withdraw
        expect(
          await withdraw.hasRole(
            await withdraw.DEFAULT_ADMIN_ROLE(),
            await config.getAddress()
          )
        ).to.equal(true)
      })
      it("set address book", async () => {
        const config = await loadFixture(setup)
        const addressBook = await config.getAddressBook()
        expect(addressBook.operator).not.to.equal(ethers.ZeroAddress)
        expect(addressBook.halo2VerifyingKeyAddress).not.to.equal(
          ethers.ZeroAddress
        )
        expect(addressBook.halo2VerifierAddress).not.to.equal(
          ethers.ZeroAddress
        )
        expect(addressBook.verifier).not.to.equal(ethers.ZeroAddress)
        expect(addressBook.rootManager).not.to.equal(ethers.ZeroAddress)
        expect(addressBook.blockManager).not.to.equal(ethers.ZeroAddress)
        expect(addressBook.liquidityManager).not.to.equal(ethers.ZeroAddress)
        expect(addressBook.main).not.to.equal(ethers.ZeroAddress)
        expect(addressBook.withdraw).not.to.equal(ethers.ZeroAddress)
        expect(addressBook.tokenAddresses.addresses.length).to.equal(4)
        expect(addressBook.tokenAddresses.addresses[0]).not.to.equal(
          ethers.ZeroAddress
        )
        expect(addressBook.tokenAddresses.addresses[1]).not.to.equal(
          ethers.ZeroAddress
        )
        expect(addressBook.tokenAddresses.addresses[2]).not.to.equal(
          ethers.ZeroAddress
        )
        expect(addressBook.tokenAddresses.addresses[3]).not.to.equal(
          ethers.ZeroAddress
        )
      })
    })
    describe("fail", () => {
      it("only deployer", async () => {
        const config = await loadFixture(setup)
        const signers = await getSigners()
        const deployerRole = await config.DEPLOYER()
        const addresses = generateDummyAddresses(13)
        await expect(
          config.connect(signers.illegalUser).configure(
            addresses[0],
            {
              addresses: [
                addresses[1],
                addresses[2],
                addresses[3],
                addresses[4],
              ],
            },
            addresses[5],
            addresses[6],
            addresses[7],
            addresses[8],
            addresses[9],
            addresses[10],
            addresses[11],
            addresses[12]
          )
        )
          .to.be.revertedWithCustomError(
            config,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, deployerRole)
      })
    })
  })
  describe("upgrade", () => {
    it("contract is upgradable", async () => {
      const config = await loadFixture(setup)
      const addressBook = await config.getAddressBook()
      expect(addressBook.blockManager).not.to.equal(ethers.ZeroAddress)
      const factory = await ethers.getContractFactory("TestConfig2")
      const next = await upgrades.upgradeProxy(
        await config.getAddress(),
        factory
      )
      const addressBookAfter = await next.getAddressBook()
      expect(addressBookAfter.blockManager).not.to.equal(ethers.ZeroAddress)
      const val: number = (await next.getVal()) as number
      expect(val).to.equal(1)
    })
    it("Cannot upgrade except for a deployer.", async () => {
      const config = await loadFixture(setup)
      const signers = await getSigners()
      const factory = await ethers.getContractFactory(
        "TestConfig2",
        signers.illegalUser
      )
      const role = await config.DEPLOYER()
      await expect(upgrades.upgradeProxy(await config.getAddress(), factory))
        .to.be.revertedWithCustomError(
          config,
          "AccessControlUnauthorizedAccount"
        )
        .withArgs(signers.illegalUser.address, role)
    })
  })
})

import { expect } from "chai"
import { ethers, upgrades } from "hardhat"
import { loadFixture } from "@nomicfoundation/hardhat-toolbox/network-helpers"
import {
  getSigners,
  deployAllContracts,
  generateDummyAddresses,
  generateERC20,
  getUupsContract,
} from "../test-utils"
import { Config, LiquidityManager, TestToken } from "../../typechain-types"

describe("LiquidityManager", () => {
  const setup = async (): Promise<[LiquidityManager, Config]> => {
    const config = await deployAllContracts()
    const addressBook = await config.getAddressBook()
    const liquidityManagerFactory = await ethers.getContractFactory(
      "LiquidityManager"
    )
    const liquidityManager = liquidityManagerFactory.attach(
      addressBook.liquidityManager
    )
    return [liquidityManager as LiquidityManager, config]
  }
  describe("initialize", () => {
    describe("success", () => {
      it("initialize was called", async () => {
        const [liquidityManager, config] = await loadFixture(setup)
        const result = await liquidityManager.hasRole(
          await liquidityManager.DEFAULT_ADMIN_ROLE(),
          await config.getAddress()
        )
        expect(result).to.equal(true)
      })
    })
    describe("fail", () => {
      it("Initialization can only be done once", async () => {
        const [liquidityManager] = await loadFixture(setup)
        await expect(
          liquidityManager.initialize(ethers.ZeroAddress)
        ).to.be.revertedWithCustomError(
          liquidityManager,
          "InvalidInitialization"
        )
      })
    })
  })

  describe("config", () => {
    describe("success", () => {
      it("set address info", async () => {
        const liquidityManagerFactory = await ethers.getContractFactory(
          "LiquidityManager"
        )
        const liquidityManager = await liquidityManagerFactory.deploy()
        const signers = await getSigners()
        await liquidityManager.initialize(signers.dummyConfig.address)
        const addresses = generateDummyAddresses(7)
        await liquidityManager.connect(signers.dummyConfig).config(
          {
            addresses: [addresses[0], addresses[1], addresses[2], addresses[3]],
          },
          addresses[4],
          addresses[5],
          addresses[6]
        )

        const role = await liquidityManager.INNER_GROUP()
        expect(await liquidityManager.hasRole(role, addresses[4])).to.equal(
          true
        )
        expect(await liquidityManager.hasRole(role, addresses[5])).to.equal(
          true
        )
        expect(await liquidityManager.hasRole(role, addresses[6])).to.equal(
          true
        )
      })
    })
    describe("fail", () => {
      it("only admin", async () => {
        const [liquidityManager] = await loadFixture(setup)
        const signers = await getSigners()
        const role = await liquidityManager.DEFAULT_ADMIN_ROLE()
        const testAddress = generateDummyAddresses(7)
        await expect(
          liquidityManager.connect(signers.illegalUser).config(
            {
              addresses: [
                testAddress[0],
                testAddress[1],
                testAddress[2],
                testAddress[3],
              ],
            },
            testAddress[4],
            testAddress[5],
            testAddress[6]
          )
        )
          .to.be.revertedWithCustomError(
            liquidityManager,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, role)
      })
    })
  })

  const setupTokenTransfer = async (): Promise<
    [LiquidityManager, TestToken[]]
  > => {
    const liquidityManagerFactory = await ethers.getContractFactory(
      "LiquidityManager"
    )
    const liquidityManager = await liquidityManagerFactory.deploy()
    const signers = await getSigners()
    await liquidityManager.initialize(signers.dummyConfig.address)
    const erc20s = await generateERC20()
    const erc20Addresses = await Promise.all(
      erc20s.map(async (erc20) => {
        return await erc20.getAddress()
      })
    )
    const addresses = generateDummyAddresses(2)
    await liquidityManager.connect(signers.dummyConfig).config(
      {
        addresses: [
          erc20Addresses[0],
          erc20Addresses[1],
          erc20Addresses[2],
          erc20Addresses[3],
        ],
      },
      signers.dummyInnerGroup.address,
      addresses[0],
      addresses[1]
    )
    return [liquidityManager, erc20s]
  }
  describe("receiveAssets", () => {
    it("receive token", async () => {
      const [liquidityManager, erc20s] = await setupTokenTransfer()
      const signers = await getSigners()
      await Promise.all(
        erc20s.map(async (erc20) => {
          await erc20.mint(signers.user.address, 100n)
          await erc20
            .connect(signers.user)
            .approve(await liquidityManager.getAddress(), 100n)
        })
      )

      expect(await erc20s[0].balanceOf(signers.user.address)).to.equal(100n)
      expect(await erc20s[1].balanceOf(signers.user.address)).to.equal(100n)
      expect(await erc20s[2].balanceOf(signers.user.address)).to.equal(100n)
      expect(await erc20s[3].balanceOf(signers.user.address)).to.equal(100n)
      expect(
        await erc20s[0].balanceOf(await liquidityManager.getAddress())
      ).to.equal(0n)
      expect(
        await erc20s[1].balanceOf(await liquidityManager.getAddress())
      ).to.equal(0n)
      expect(
        await erc20s[2].balanceOf(await liquidityManager.getAddress())
      ).to.equal(0n)
      expect(
        await erc20s[3].balanceOf(await liquidityManager.getAddress())
      ).to.equal(0n)
      await liquidityManager
        .connect(signers.dummyInnerGroup)
        .receiveAssets(signers.user.address, { amounts: [100n, 50n, 0n, 20n] })
      expect(await erc20s[0].balanceOf(signers.user.address)).to.equal(0n)
      expect(await erc20s[1].balanceOf(signers.user.address)).to.equal(50n)
      expect(await erc20s[2].balanceOf(signers.user.address)).to.equal(100n)
      expect(await erc20s[3].balanceOf(signers.user.address)).to.equal(80n)

      expect(
        await erc20s[0].balanceOf(await liquidityManager.getAddress())
      ).to.equal(100n)
      expect(
        await erc20s[1].balanceOf(await liquidityManager.getAddress())
      ).to.equal(50n)
      expect(
        await erc20s[2].balanceOf(await liquidityManager.getAddress())
      ).to.equal(0n)
      expect(
        await erc20s[3].balanceOf(await liquidityManager.getAddress())
      ).to.equal(20n)
    })
  })
  describe("sendAssets", () => {
    it("receive token", async () => {
      const [liquidityManager, erc20s] = await setupTokenTransfer()
      const signers = await getSigners()
      const liquidityManagerAddress = await liquidityManager.getAddress()
      await Promise.all(
        erc20s.map(async (erc20) => {
          await erc20.mint(liquidityManagerAddress, 100n)
        })
      )

      expect(await erc20s[0].balanceOf(signers.user.address)).to.equal(0n)
      expect(await erc20s[1].balanceOf(signers.user.address)).to.equal(0n)
      expect(await erc20s[2].balanceOf(signers.user.address)).to.equal(0n)
      expect(await erc20s[3].balanceOf(signers.user.address)).to.equal(0n)

      expect(await erc20s[0].balanceOf(liquidityManagerAddress)).to.equal(100n)
      expect(await erc20s[1].balanceOf(liquidityManagerAddress)).to.equal(100n)
      expect(await erc20s[2].balanceOf(liquidityManagerAddress)).to.equal(100n)
      expect(await erc20s[3].balanceOf(liquidityManagerAddress)).to.equal(100n)
      await liquidityManager
        .connect(signers.dummyInnerGroup)
        .sendAssets(signers.user.address, { amounts: [100n, 50n, 0n, 20n] })
      expect(await erc20s[0].balanceOf(signers.user.address)).to.equal(100n)
      expect(await erc20s[1].balanceOf(signers.user.address)).to.equal(50n)
      expect(await erc20s[2].balanceOf(signers.user.address)).to.equal(0n)
      expect(await erc20s[3].balanceOf(signers.user.address)).to.equal(20n)

      expect(
        await erc20s[0].balanceOf(await liquidityManager.getAddress())
      ).to.equal(0n)
      expect(
        await erc20s[1].balanceOf(await liquidityManager.getAddress())
      ).to.equal(50n)
      expect(
        await erc20s[2].balanceOf(await liquidityManager.getAddress())
      ).to.equal(100n)
      expect(
        await erc20s[3].balanceOf(await liquidityManager.getAddress())
      ).to.equal(80n)
    })
  })

  describe("upgrade", () => {
    it("contract is upgradable", async () => {
      const signers = await getSigners()
      const liquidityManager = await getUupsContract<LiquidityManager>(
        "LiquidityManager",
        [signers.dummyConfig.address]
      )
      const role = await liquidityManager.DEFAULT_ADMIN_ROLE()
      const result = await liquidityManager.hasRole(
        role,
        signers.dummyConfig.address
      )
      expect(result).to.equal(true)

      const factory = await ethers.getContractFactory(
        "TestLiquidityManager2",
        signers.dummyConfig
      )
      const next = await upgrades.upgradeProxy(
        await liquidityManager.getAddress(),
        factory
      )
      const result2 = await next.hasRole(role, signers.dummyConfig.address)
      expect(result).to.equal(result2)
      const val: number = (await next.getVal()) as number
      expect(val).to.equal(2)
    })
    it("Cannot upgrade except for a deployer.", async () => {
      const [liquidityManager] = await loadFixture(setup)
      const signers = await getSigners()
      const factory = await ethers.getContractFactory(
        "TestLiquidityManager2",
        signers.illegalUser
      )
      const role = await liquidityManager.DEFAULT_ADMIN_ROLE()
      await expect(
        upgrades.upgradeProxy(await liquidityManager.getAddress(), factory)
      )
        .to.be.revertedWithCustomError(
          liquidityManager,
          "AccessControlUnauthorizedAccount"
        )
        .withArgs(signers.illegalUser.address, role)
    })
  })
})

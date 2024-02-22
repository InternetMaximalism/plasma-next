import { expect } from "chai"
import { ethers } from "hardhat"
import { loadFixture } from "@nomicfoundation/hardhat-toolbox/network-helpers"
import { TestAssetLib } from "../../typechain-types"

describe("AssetLib", () => {
  const setup = async (): Promise<TestAssetLib> => {
    const factory = await ethers.getContractFactory("TestAssetLib")
    const assetLib = await factory.deploy()
    return assetLib
  }

  describe("isLe", () => {
    it("return false", async () => {
      const assetLib = await loadFixture(setup)
      const result = await assetLib.isLe(
        {
          amounts: [2n, 3n, 4n, 5n],
        },
        {
          amounts: [1n, 2n, 3n, 4n],
        }
      )
      expect(result).to.equal(false)
    })
    it("return true", async () => {
      const assetLib = await loadFixture(setup)
      const result = await assetLib.isLe(
        {
          amounts: [2n, 3n, 4n, 5n],
        },
        {
          amounts: [3n, 4n, 5n, 6n],
        }
      )
      expect(result).to.equal(true)
    })
  })

  describe("isZero", () => {
    it("return false", async () => {
      const assetLib = await loadFixture(setup)
      const result = await assetLib.isZero({
        amounts: [0n, 0n, 1n, 0n],
      })
      expect(result).to.equal(false)
    })
    it("return true", async () => {
      const assetLib = await loadFixture(setup)
      const result = await assetLib.isZero({
        amounts: [0n, 0n, 0n, 0n],
      })
      expect(result).to.equal(true)
    })
  })

  describe("isEq", () => {
    it("return false", async () => {
      const assetLib = await loadFixture(setup)
      const result = await assetLib.isEq(
        {
          amounts: [0n, 0n, 1n, 0n],
        },
        {
          amounts: [1n, 0n, 1n, 0n],
        }
      )
      expect(result).to.equal(false)
    })
    it("return true", async () => {
      const assetLib = await loadFixture(setup)
      const result = await assetLib.isEq(
        {
          amounts: [1n, 0n, 1n, 0n],
        },
        {
          amounts: [1n, 0n, 1n, 0n],
        }
      )
      expect(result).to.equal(true)
    })
  })
  describe("add", () => {
    it("add amounts", async () => {
      const assetLib = await loadFixture(setup)
      const result = await assetLib.add(
        {
          amounts: [0n, 2n, 3n, 5n],
        },
        {
          amounts: [1n, 0n, 1n, 0n],
        }
      )
      expect(result.amounts).to.deep.equal([1n, 2n, 4n, 5n])
    })
  })
  describe("singleAsset", () => {
    it("add amounts", async () => {
      const assetLib = await loadFixture(setup)
      const result = await assetLib.singleAsset(2, 4)
      expect(result.amounts).to.deep.equal([0n, 0n, 4n, 0n])
    })
  })
  describe("sub", () => {
    it("sub amounts", async () => {
      const assetLib = await loadFixture(setup)
      const result = await assetLib.sub(
        {
          amounts: [5n, 6n, 3n, 2n],
        },
        {
          amounts: [1n, 2n, 3n, 1n],
        }
      )
      expect(result.amounts).to.deep.equal([4n, 4n, 0n, 1n])
    })
  })
})

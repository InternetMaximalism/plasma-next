import { expect } from "chai"
import { ethers } from "hardhat"
import { loadFixture } from "@nomicfoundation/hardhat-toolbox/network-helpers"
import { TestLeafLib } from "../../typechain-types"
import { testAddress1, testHash1 } from "../test-utils"

describe("LeafLib", () => {
  const setup = async (): Promise<TestLeafLib> => {
    const factory = await ethers.getContractFactory("TestLeafLib")
    const leafLib = await factory.deploy()
    return leafLib
  }

  describe("hashEvidenceLeaf", () => {
    it("get hash", async () => {
      const leafLib = await loadFixture(setup)
      const result = await leafLib.hashEvidenceLeaf({
        transferCommitment: testHash1,
        ebn: 6n,
      })
      expect(result).to.equal(
        "0xfff9856f44c0c5b0f4c8e4159301fd2ec0d6e7c069cc29102489b3fa345d8f68"
      )
    })
  })
  describe("hashWithdrawLeaf", () => {
    it("get hash", async () => {
      const leafLib = await loadFixture(setup)
      const result = await leafLib.hashWithdrawLeaf({
        recipient: testAddress1,
        amount: { amounts: [1n, 2n, 3n, 4n] },
        startEbn: 2n,
        endEbn: 6n,
      })
      expect(result).to.equal(
        "0x1669221c9ce8836b278f0cbe3129c29e72199f3076fc2fa6b4263f2368ef7941"
      )
    })
  })
})

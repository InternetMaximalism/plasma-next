import { expect } from "chai"
import { ethers } from "hardhat"
import { loadFixture } from "@nomicfoundation/hardhat-toolbox/network-helpers"
import { TestTransferLib } from "../../typechain-types"
import { testAddress1 } from "../test-utils"

describe("TransferLib", () => {
  const setup = async (): Promise<TestTransferLib> => {
    const factory = await ethers.getContractFactory("TestTransferLib")
    const testTransferLib = await factory.deploy()
    return testTransferLib
  }

  describe("transferCommitment(Withdraw)", () => {
    it("get hash", async () => {
      const testTransferLib = await loadFixture(setup)
      const result = await testTransferLib.transferCommitment({
        recipient: testAddress1,
        amount: 12,
        assetId: 1,
      })
      expect(result).to.equal(
        "0x82f2ae276e0d5ab225977d1edca8a49236fccb5e95ff6f6d182122e6e045013f"
      )
    })
  })
})

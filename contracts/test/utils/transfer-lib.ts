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
        nonce: 2,
      })
      expect(result).to.equal(
        "0x6b7fff8df3d87e5eeee1909bf3ab64c0a50fadad9f3b80ca908f55a750c34bb0"
      )
    })
  })
})

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

  describe("hashLeaf(Evidence)", () => {
    it("get hash", async () => {
      const leafLib = await loadFixture(setup)
      const result = await leafLib.hashSettlementLeaf({
        withdrawLeaf: {
          recipient: testAddress1,
          amount: { amounts: [1n, 2n, 3n, 4n] },
          startEbn: 2n,
          endEbn: 6n,
        },
        evidenceLeaf: {
          transferCommitment: testHash1,
          ebn: 6n,
        },
      })
      expect(result).to.equal(
        "0x3a76ef5212e62eeaeac15439a96d8c1ba184c6b82cdf1ff111b82fd649f0f332"
      )
    })
  })
})

import { expect } from "chai"
import { ethers } from "hardhat"
import { loadFixture } from "@nomicfoundation/hardhat-toolbox/network-helpers"
import { TestMerkleProofLib } from "../../typechain-types"
import { testHash1 } from "../test-utils"

describe("MerkleProofLib", () => {
  const setup = async (): Promise<TestMerkleProofLib> => {
    const factory = await ethers.getContractFactory("TestMerkleProofLib")
    const merkleProofLib = await factory.deploy()
    return merkleProofLib
  }

  describe("getRootFromMerkleProof", () => {
    it("get proof", async () => {
      const merkleProofLib = await loadFixture(setup)
      const result = await merkleProofLib.getRootFromMerkleProof(testHash1, 1, [
        testHash1,
        testHash1,
      ])
      expect(result).to.equal(
        "0xe6383f3dc437d84c9cb80a919cf82d728755be99be291e85ce24dfe7de3325e2"
      )
    })
  })
})

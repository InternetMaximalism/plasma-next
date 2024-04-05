import { expect } from "chai"
import { ethers } from "hardhat"
import { loadFixture } from "@nomicfoundation/hardhat-toolbox/network-helpers"
import { DefaultZKPTLC } from "../../typechain-types"

describe("DefaultZKPTLC", () => {
  const setup = async (): Promise<[DefaultZKPTLC, string]> => {
    const testTestRootManager3Factory = await ethers.getContractFactory(
      "TestRootManager3"
    )
    const testRootManager3 = await testTestRootManager3Factory.deploy()
    const rootManagerAddress = await testRootManager3.getAddress()
    const testDefaultZKPTLCFactory = await ethers.getContractFactory(
      "DefaultZKPTLC"
    )
    const defaultZKPTLC = await testDefaultZKPTLCFactory.deploy(
      rootManagerAddress
    )
    return [defaultZKPTLC, rootManagerAddress]
  }
  const initialTransfer = {
    recipient: ethers.ZeroAddress,
    amount: 0,
    assetId: 0,
    nonce: 0,
  }
  const initialTransferCommitment =
    "0x2af357fc2ab2964b76482ec0fcac3b86f5aca1a8292676023c8b9ec392d821a0"
  const initialWitnessEncoded =
    "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000"
  describe("constructor", () => {
    it("set address", async () => {
      const [defaultZKPTLC, rootManagerAddress] = await loadFixture(setup)
      expect(await defaultZKPTLC.rootManagerAddress()).to.equal(
        rootManagerAddress
      )
    })
  })

  describe("computeInstance", () => {
    it("compute instance", async () => {
      const [defaultZKPTLC] = await loadFixture(setup)
      const commitment = await defaultZKPTLC.computeInstance(initialTransfer)
      expect(commitment).to.equal(initialTransferCommitment)
    })
  })

  describe("encodeWitness", () => {
    it("encode Witness", async () => {
      const [defaultZKPTLC] = await loadFixture(setup)
      const result = await defaultZKPTLC.encodeWitness({
        transfer: initialTransfer,
        proof: {
          leaf: {
            transferCommitment: ethers.ZeroHash,
            ebn: 0,
          },
          index: 0,
          siblings: [],
        },
      })
      expect(result).to.equal(initialWitnessEncoded)
    })
  })

  describe("encodeWitness", () => {
    describe("success", () => {
      it("verify", async () => {
        const [defaultZKPTLC] = await loadFixture(setup)
        await defaultZKPTLC.verifyCondition(
          initialTransferCommitment,
          "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a02af357fc2ab2964b76482ec0fcac3b86f5aca1a8292676023c8b9ec392d821a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000"
        )
      })
    })
    describe("fail", () => {
      it("invalid instance", async () => {
        const [defaultZKPTLC] = await loadFixture(setup)

        await expect(
          defaultZKPTLC.verifyCondition(
            ethers.ZeroHash,
            "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a02af357fc2ab2964b76482ec0fcac3b86f5aca1a8292676023c8b9ec392d821a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000"
          )
        ).to.be.revertedWith("Invalid instance")
      })
      it("transfer commitment does not match", async () => {
        const [defaultZKPTLC] = await loadFixture(setup)

        await expect(
          defaultZKPTLC.verifyCondition(
            initialTransferCommitment,
            initialWitnessEncoded
          )
        ).to.be.revertedWith("Transfer commitment does not match")
      })
    })
  })
})

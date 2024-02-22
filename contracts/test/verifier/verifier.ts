import { expect } from "chai"
import { ethers } from "hardhat"
import { loadFixture } from "@nomicfoundation/hardhat-toolbox/network-helpers"
import {
  getSigners,
  deployAllContracts,
  generateDummyAddresses,
  testHash1,
  testHash2,
} from "../test-utils"
import { Config, Verifier } from "../../typechain-types"

describe("Verifier", () => {
  const setup = async (): Promise<[Verifier, Config]> => {
    const config = await deployAllContracts()
    const addressBook = await config.getAddressBook()
    const testVerifierFactory = await ethers.getContractFactory("Verifier")
    const verifier = testVerifierFactory.attach(addressBook.verifier)
    return [verifier as Verifier, config]
  }

  describe("constructor", () => {
    it("verify", async () => {
      const verifierFactory = await ethers.getContractFactory("Verifier")
      const testAddress = generateDummyAddresses(1)[0]
      const verifier = await verifierFactory.deploy(testAddress)
      const role = await verifier.DEFAULT_ADMIN_ROLE()
      const result = await verifier.hasRole(role, testAddress)
      expect(result).to.equal(true)
    })
  })

  describe("config", () => {
    describe("success", () => {
      it("set address", async () => {
        const verifierFactory = await ethers.getContractFactory("Verifier")
        const signers = await getSigners()
        const verifier = await verifierFactory.deploy(
          signers.dummyConfig.address
        )
        const halo2VerifyingKeyAddress =
          await verifier.halo2VerifyingKeyAddress()
        expect(halo2VerifyingKeyAddress).to.equal(ethers.ZeroAddress)
        const halo2VerifierAddress = await verifier.halo2VerifierAddress()
        expect(halo2VerifierAddress).to.equal(ethers.ZeroAddress)
        const testAddress = generateDummyAddresses(2)
        await verifier
          .connect(signers.dummyConfig)
          .config(testAddress[0], testAddress[1])
        const halo2VerifyingKeyAddressAfter =
          await verifier.halo2VerifyingKeyAddress()
        expect(halo2VerifyingKeyAddressAfter).to.equal(testAddress[0])
        const halo2VerifierAddressAfter = await verifier.halo2VerifierAddress()
        expect(halo2VerifierAddressAfter).to.equal(testAddress[1])
      })
    })
    describe("fail", () => {
      it("only admin", async () => {
        const [verifier] = await loadFixture(setup)
        const signers = await getSigners()
        const adminRole = await verifier.DEFAULT_ADMIN_ROLE()
        const testAddress = generateDummyAddresses(1)[0]
        await expect(
          verifier.connect(signers.illegalUser).config(testAddress, testAddress)
        )
          .to.be.revertedWithCustomError(
            verifier,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, adminRole)
      })
    })
  })

  describe("verifyProof", () => {
    const test = async (_result: boolean): Promise<void> => {
      const signers = await getSigners()
      const verifierFactory = await ethers.getContractFactory("Verifier")
      const verifier = await verifierFactory.deploy(signers.dummyConfig.address)
      const testHalo2VerifierFactory = await ethers.getContractFactory(
        "TestHalo2Verifier"
      )
      const halo2Verifier = await testHalo2VerifierFactory.deploy()
      const testAddress = generateDummyAddresses(1)[0]

      await halo2Verifier.setVerifyProofResult(_result)

      await verifier
        .connect(signers.dummyConfig)
        .config(testAddress, await halo2Verifier.getAddress())
      const result = await verifier.verifyProof(
        {
          blockHash: testHash1,
          settlementRoot: testHash2,
        },
        "0x"
      )
      expect(result).to.equal(_result)
    }
    it("return success", async () => {
      await test(true)
    })
    it("return false", async () => {
      await test(false)
    })
  })
})

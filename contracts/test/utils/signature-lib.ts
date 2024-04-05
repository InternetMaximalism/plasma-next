import { expect } from "chai"
import { ethers } from "hardhat"
import { loadFixture } from "@nomicfoundation/hardhat-toolbox/network-helpers"
import { TestSignature } from "../../typechain-types"
import { getSigners, testAddress1, testHash1 } from "../test-utils"
import {
  initialPayment,
  signPayment,
  getUniqueIdentifier,
} from "../../scripts/utils/payment"

describe("SignatureLib", () => {
  const setup = async (): Promise<TestSignature> => {
    const factory = await ethers.getContractFactory("TestSignature")
    const signatureLib = await factory.deploy()
    return signatureLib
  }

  describe("verifyPaymentSignature", () => {
    describe("success", () => {
      it("verify", async () => {
        const signatureLib = await loadFixture(setup)
        const signers = await getSigners()
        const identifier = await getUniqueIdentifier(
          await signatureLib.getAddress()
        )
        const payment = initialPayment(signers.user.address, 10n, 0)
        payment.zkptlcAddress = testAddress1
        payment.zkptlcInstance = testHash1
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        await signatureLib.verifyPaymentSignature(
          {
            payment: payment,
            userSignature: signed.userSignature,
            operatorSignature: signed.operatorSignature,
          },
          signers.operator.address,
          signers.user.address
        )
        expect(true).to.equal(true)
      })
    })
    describe("fail", () => {
      it("user mismatch", async () => {
        const signatureLib = await loadFixture(setup)
        const signers = await getSigners()
        const identifier = await getUniqueIdentifier(
          await signatureLib.getAddress()
        )
        const payment = initialPayment(signers.user.address, 10n, 0)
        payment.zkptlcAddress = testAddress1
        payment.zkptlcInstance = testHash1
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        await expect(
          signatureLib.verifyPaymentSignature(
            {
              payment,
              userSignature: signed.userSignature,
              operatorSignature: signed.operatorSignature,
            },
            signers.operator.address,
            signers.illegalUser.address
          )
        )
          .to.be.revertedWithCustomError(signatureLib, "UserMismatch")
          .withArgs(signers.illegalUser.address, payment.user)
      })
      it("invalid user signature", async () => {
        const signatureLib = await loadFixture(setup)
        const signers = await getSigners()
        const identifier = await getUniqueIdentifier(
          await signatureLib.getAddress()
        )
        const payment = initialPayment(signers.user.address, 10n, 0)
        payment.zkptlcAddress = testAddress1
        payment.zkptlcInstance = testHash1
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        await expect(
          signatureLib.verifyPaymentSignature(
            {
              payment,
              userSignature: signed.operatorSignature,
              operatorSignature: signed.operatorSignature,
            },
            signers.operator.address,
            signers.user.address
          )
        )
          .to.be.revertedWithCustomError(signatureLib, "InvalidUserSignature")
          .withArgs(signers.user.address, signers.operator.address)
      })
      it("invalid operator signature", async () => {
        const signatureLib = await loadFixture(setup)
        const signers = await getSigners()
        const identifier = await getUniqueIdentifier(
          await signatureLib.getAddress()
        )
        const payment = initialPayment(signers.user.address, 10n, 0)
        payment.zkptlcAddress = testAddress1
        payment.zkptlcInstance = testHash1
        const signed = await signPayment(
          signers.user,
          signers.operator,
          identifier,
          payment
        )
        await expect(
          signatureLib.verifyPaymentSignature(
            {
              payment,
              userSignature: signed.userSignature,
              operatorSignature: signed.userSignature,
            },
            signers.operator.address,
            signers.user.address
          )
        )
          .to.be.revertedWithCustomError(
            signatureLib,
            "InvalidOperatorSignature"
          )
          .withArgs(signers.operator.address, signers.user.address)
      })
    })
  })
  describe("getUniqueIdentifier", () => {
    it("get identifier", async () => {
      const signatureLib = await loadFixture(setup)
      const identifier = await getUniqueIdentifier(
        await signatureLib.getAddress()
      )
      const identifierFromContract = await signatureLib.getUniqueIdentifier()
      expect(identifier).to.equal(identifierFromContract)
    })
  })
})

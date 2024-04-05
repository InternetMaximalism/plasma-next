import { expect } from "chai"
import { ethers, upgrades } from "hardhat"
import { loadFixture } from "@nomicfoundation/hardhat-toolbox/network-helpers"
import {
  getSigners,
  deployAllContracts,
  generateDummyAddresses,
  generateDummyHashes,
  getUupsContract,
  testHash1,
  testHash2,
  testHash3,
  testAddress1,
} from "../test-utils"
import { Config, RootManager, TestVerifier2 } from "../../typechain-types"

describe("RootManager", () => {
  const setup = async (): Promise<[RootManager, Config]> => {
    const config = await deployAllContracts()
    const addressBook = await config.getAddressBook()
    const testRootManagerFactory = await ethers.getContractFactory(
      "RootManager"
    )
    const rootManager = testRootManagerFactory.attach(addressBook.rootManager)
    return [rootManager as RootManager, config]
  }
  const setupWithVerifier = async (): Promise<[RootManager, TestVerifier2]> => {
    const testRootManagerFactory = await ethers.getContractFactory(
      "RootManager"
    )
    const rootManager = await testRootManagerFactory.deploy()
    const testVerifier2Factory = await ethers.getContractFactory(
      "TestVerifier2"
    )
    const verifier = await testVerifier2Factory.deploy()

    const testBlockManager3Factory = await ethers.getContractFactory(
      "TestBlockManager3"
    )
    const blockManager = await testBlockManager3Factory.deploy()

    const signers = await getSigners()
    await rootManager.initialize(signers.dummyConfig.address)
    //const addresses = generateDummyAddresses(1)
    await rootManager
      .connect(signers.dummyConfig)
      .config(await verifier.getAddress(), await blockManager.getAddress())
    return [rootManager, verifier]
  }
  describe("initialize", () => {
    describe("success", () => {
      it("initialize was called", async () => {
        const [rootManager, config] = await loadFixture(setup)
        const result = await rootManager.hasRole(
          await rootManager.DEFAULT_ADMIN_ROLE(),
          await config.getAddress()
        )
        expect(result).to.equal(true)
      })
    })
    describe("fail", () => {
      it("Initialization can only be done once", async () => {
        const [rootManager] = await loadFixture(setup)
        const testAddresses = generateDummyAddresses(1)
        await expect(
          rootManager.initialize(testAddresses[0])
        ).to.be.revertedWithCustomError(rootManager, "InvalidInitialization")
      })
    })
  })

  describe("config", () => {
    describe("success", () => {
      it("set address info", async () => {
        const testRootManagerFactory = await ethers.getContractFactory(
          "RootManager"
        )
        const rootManager = await testRootManagerFactory.deploy()
        const signers = await getSigners()
        await rootManager.initialize(signers.dummyConfig.address)
        const addresses = generateDummyAddresses(2)
        expect(await rootManager.verifierAddress()).to.not.equal(addresses[1])
        expect(await rootManager.blockManagerAddress()).to.not.equal(
          addresses[2]
        )

        await rootManager
          .connect(signers.dummyConfig)
          .config(addresses[0], addresses[1])
        expect(await rootManager.verifierAddress()).to.equal(addresses[0])
        expect(await rootManager.blockManagerAddress()).to.equal(addresses[1])
      })
    })
    describe("fail", () => {
      it("only admin", async () => {
        const [rootManager] = await loadFixture(setup)
        const signers = await getSigners()
        const role = await rootManager.DEFAULT_ADMIN_ROLE()
        const testAddresses = generateDummyAddresses(2)
        await expect(
          rootManager
            .connect(signers.illegalUser)
            .config(testAddresses[0], testAddresses[1])
        )
          .to.be.revertedWithCustomError(
            rootManager,
            "AccessControlUnauthorizedAccount"
          )
          .withArgs(signers.illegalUser.address, role)
      })
    })
  })

  describe("postRoot", () => {
    describe("success", () => {
      it("generated RootPosted event", async () => {
        const [rootManager, verifier] = await setupWithVerifier()
        const testHashes = generateDummyHashes(4)
        await verifier.setResult(true)
        await expect(
          rootManager.postRoot(
            1,
            [ethers.ZeroHash],
            [ethers.ZeroHash],
            {
              blockHash: testHashes[0],
              evidenceRoot: testHashes[1],
              withdrawRoot: testHashes[2],
            },
            testHashes[3]
          )
        )
          .to.emit(rootManager, "RootPosted")
          .withArgs(testHashes[2], testHashes[1])
      })
      it("update exist flg", async () => {
        const [rootManager, verifier] = await setupWithVerifier()
        const testHashes = generateDummyHashes(4)

        await verifier.setResult(true)
        const withdrawExistFlg = await rootManager.doesWithdrawRootExist(
          testHashes[2]
        )
        await expect(withdrawExistFlg).to.equal(false)
        const evidenceExistFlg = await rootManager.doesEvidenceRootExist(
          testHashes[1]
        )
        await expect(evidenceExistFlg).to.equal(false)
        await rootManager.postRoot(
          1,
          [ethers.ZeroHash],
          [ethers.ZeroHash],
          {
            blockHash: testHashes[0],
            evidenceRoot: testHashes[1],
            withdrawRoot: testHashes[2],
          },
          testHashes[3]
        )

        const withdrawExistFlgAfter = await rootManager.doesWithdrawRootExist(
          testHashes[2]
        )
        await expect(withdrawExistFlgAfter).to.equal(true)
        const evidenceExistFlgAfter = await rootManager.doesEvidenceRootExist(
          testHashes[1]
        )
        await expect(evidenceExistFlgAfter).to.equal(true)
      })
    })
    describe("fail", () => {
      it("not success verify proof", async () => {
        const [rootManager, verifier] = await setupWithVerifier()
        await verifier.setResult(false)
        const testHashes = generateDummyHashes(4)
        await expect(
          rootManager.postRoot(
            1,
            [ethers.ZeroHash],
            [ethers.ZeroHash],
            {
              blockHash: testHashes[0],
              evidenceRoot: testHashes[1],
              withdrawRoot: testHashes[2],
            },
            testHashes[3]
          )
        ).to.be.revertedWithCustomError(rootManager, "ProofVerificationFailed")
      })
      it("proof verification failed", async () => {
        const [rootManager, verifier] = await setupWithVerifier()
        const testHashes = generateDummyHashes(4)
        await verifier.setRevert(true)
        await verifier.setResult(true)

        await expect(
          rootManager.postRoot(
            1,
            [ethers.ZeroHash],
            [ethers.ZeroHash],
            {
              blockHash: testHashes[0],
              evidenceRoot: testHashes[1],
              withdrawRoot: testHashes[2],
            },
            testHashes[3]
          )
        ).to.be.revertedWithCustomError(rootManager, "ProofVerificationFailed")
      })
    })
  })

  describe("verifyWithdrawMerkleProof", () => {
    describe("success", () => {
      it("set root flg", async () => {
        const [rootManager, verifier] = await setupWithVerifier()

        await verifier.setResult(true)

        await rootManager.postRoot(
          1,
          [ethers.ZeroHash],
          [ethers.ZeroHash],
          {
            blockHash: testHash1,
            evidenceRoot: testHash2,
            withdrawRoot:
              "0xcebebaaae83ff866e50d5979e1708329fa76da7587211ad68552994c4f059f2d",
          },
          testHash3
        )
        await rootManager.verifyWithdrawMerkleProof({
          leaf: {
            recipient: testAddress1,
            amount: { amounts: [1n, 2n, 3n, 4n] },
            startEbn: 1n,
            endEbn: 10n,
          },
          index: 0,
          siblings: [testHash1, testHash2],
        })
        expect(true).to.equal(true)
      })
    })
    describe("fail", () => {
      it("not set root flg", async () => {
        const [rootManager] = await loadFixture(setup)
        await expect(
          rootManager.verifyWithdrawMerkleProof({
            leaf: {
              recipient: testAddress1,
              amount: { amounts: [1n, 2n, 3n, 4n] },
              startEbn: 1n,
              endEbn: 10n,
            },
            index: 0,
            siblings: [testHash1, testHash2],
          })
        )
          .to.be.revertedWithCustomError(
            rootManager,
            "InvalidWithdrawMerkleProof"
          )
          .withArgs(
            "0xcebebaaae83ff866e50d5979e1708329fa76da7587211ad68552994c4f059f2d"
          )
      })
    })
  })

  describe("verifyEvidenceMerkleProof", () => {
    describe("success", () => {
      it("set root flg", async () => {
        const [rootManager, verifier] = await setupWithVerifier()

        await verifier.setResult(true)

        await rootManager.postRoot(
          1,
          [ethers.ZeroHash],
          [ethers.ZeroHash],
          {
            blockHash: testHash1,
            evidenceRoot:
              "0xee40fed8b089ef2379f921ac857dbfd231ec811e97f8ac80149f0b18ea7aeb45",
            withdrawRoot: testHash2,
          },
          testHash3
        )
        await rootManager.verifyEvidenceMerkleProof({
          leaf: {
            transferCommitment: testHash1,
            ebn: 1n,
          },
          index: 0,
          siblings: [testHash2, testHash3],
        })
        expect(true).to.equal(true)
      })
    })
    describe("fail", () => {
      it("not set root flg", async () => {
        const [rootManager] = await loadFixture(setup)
        await expect(
          rootManager.verifyEvidenceMerkleProof({
            leaf: {
              transferCommitment: testHash1,
              ebn: 1n,
            },
            index: 0,
            siblings: [testHash1, testHash2],
          })
        )
          .to.be.revertedWithCustomError(
            rootManager,
            "InvalidEvidenceMerkleProof"
          )
          .withArgs(
            "0x03fdd5cbd981224c9a24f8c6505a58d23e3c40578b5d627f010a5a96c3542743"
          )
      })
    })
  })

  describe("upgrade", () => {
    it("contract is upgradable", async () => {
      const signers = await getSigners()
      const rootManager = await getUupsContract<RootManager>("RootManager", [
        signers.dummyConfig.address,
      ])
      const role = await rootManager.DEFAULT_ADMIN_ROLE()
      const result = await rootManager.hasRole(
        role,
        signers.dummyConfig.address
      )
      expect(result).to.equal(true)

      const factory = await ethers.getContractFactory(
        "TestRootManager2",
        signers.dummyConfig
      )
      const next = await upgrades.upgradeProxy(
        await rootManager.getAddress(),
        factory
      )
      const result2 = await next.hasRole(role, signers.dummyConfig.address)
      expect(result).to.equal(result2)
      const val: number = (await next.getVal()) as number
      expect(val).to.equal(9)
    })
    it("Cannot upgrade except for a deployer.", async () => {
      const [rootManager] = await loadFixture(setup)
      const signers = await getSigners()
      const factory = await ethers.getContractFactory(
        "TestRootManager2",
        signers.illegalUser
      )
      const role = await rootManager.DEFAULT_ADMIN_ROLE()
      await expect(
        upgrades.upgradeProxy(await rootManager.getAddress(), factory)
      )
        .to.be.revertedWithCustomError(
          rootManager,
          "AccessControlUnauthorizedAccount"
        )
        .withArgs(signers.illegalUser.address, role)
    })
  })
})

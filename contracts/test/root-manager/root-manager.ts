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
import {
  Config,
  RootManager,
  TestVerifier2,
  TestBlockManager3,
} from "../../typechain-types"

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
  const setupWithVerifier = async (): Promise<
    [RootManager, TestVerifier2, TestBlockManager3]
  > => {
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
      .config(
        signers.operator.address,
        await verifier.getAddress(),
        await blockManager.getAddress()
      )
    return [rootManager, verifier, blockManager]
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
        const addresses = generateDummyAddresses(3)
        const role = await rootManager.OPERATOR()
        expect(await rootManager.hasRole(role, addresses[0])).to.equal(false)

        expect(await rootManager.verifierAddress()).to.not.equal(addresses[1])
        expect(await rootManager.blockManagerAddress()).to.not.equal(
          addresses[2]
        )

        await rootManager
          .connect(signers.dummyConfig)
          .config(addresses[0], addresses[1], addresses[2])
        expect(await rootManager.hasRole(role, addresses[0])).to.equal(true)
        expect(await rootManager.verifierAddress()).to.equal(addresses[1])
        expect(await rootManager.blockManagerAddress()).to.equal(addresses[2])
      })
    })
    describe("fail", () => {
      it("only admin", async () => {
        const [rootManager] = await loadFixture(setup)
        const signers = await getSigners()
        const role = await rootManager.DEFAULT_ADMIN_ROLE()
        const testAddresses = generateDummyAddresses(3)
        await expect(
          rootManager
            .connect(signers.illegalUser)
            .config(testAddresses[0], testAddresses[1], testAddresses[2])
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
        const [rootManager, verifier, blockManager] = await setupWithVerifier()
        const testHashes = generateDummyHashes(3)
        await verifier.setResult(true)
        await blockManager.setGetBlockHashResult(1, testHashes[0])

        await expect(
          rootManager.postRoot(
            1,
            {
              blockHash: testHashes[0],
              settlementRoot: testHashes[1],
            },
            testHashes[2]
          )
        )
          .to.emit(rootManager, "RootPosted")
          .withArgs(testHashes[1])
      })
      it("update exist flg", async () => {
        const [rootManager, verifier, blockManager] = await setupWithVerifier()
        const testHashes = generateDummyHashes(3)

        await verifier.setResult(true)
        await blockManager.setGetBlockHashResult(1, testHashes[0])
        const existFlg = await rootManager.doesSettlementRootExist(
          testHashes[1]
        )
        await expect(existFlg).to.equal(false)
        await rootManager.postRoot(
          1,
          {
            blockHash: testHashes[0],
            settlementRoot: testHashes[1],
          },
          testHashes[2]
        )

        const existFlgAfter = await rootManager.doesSettlementRootExist(
          testHashes[1]
        )
        await expect(existFlgAfter).to.equal(true)
      })
    })
    describe("fail", () => {
      it("not success verify proof", async () => {
        const [rootManager, verifier, blockManager] = await setupWithVerifier()
        await verifier.setResult(false)
        const testHashes = generateDummyHashes(3)
        await blockManager.setGetBlockHashResult(1, testHashes[0])
        await expect(
          rootManager.postRoot(
            1,
            {
              blockHash: testHashes[0],
              settlementRoot: testHashes[1],
            },
            testHashes[2]
          )
        ).to.be.revertedWithCustomError(rootManager, "ProofVerificationFailed")
      })
      it("block hash mismatch", async () => {
        const [rootManager, verifier] = await setupWithVerifier()
        await verifier.setResult(true)
        const testHashes = generateDummyHashes(3)

        await expect(
          rootManager.postRoot(
            1,
            {
              blockHash: testHashes[0],
              settlementRoot: testHashes[1],
            },
            testHashes[2]
          )
        )
          .to.be.revertedWithCustomError(rootManager, "BlockHashMismatch")
          .withArgs(testHashes[0], ethers.ZeroHash)
      })
      it("proof verification failed", async () => {
        const [rootManager, verifier, blockManager] = await setupWithVerifier()
        const testHashes = generateDummyHashes(3)
        await verifier.setRevert(true)
        await verifier.setResult(true)
        await blockManager.setGetBlockHashResult(1, testHashes[0])

        await expect(
          rootManager.postRoot(
            1,
            {
              blockHash: testHashes[0],
              settlementRoot: testHashes[1],
            },
            testHashes[2]
          )
        ).to.be.revertedWithCustomError(rootManager, "ProofVerificationFailed")
      })
    })
  })

  describe("verifySettlementMerkleProof", () => {
    describe("success", () => {
      it("set root flg", async () => {
        const [rootManager, verifier, blockManager] = await setupWithVerifier()

        await verifier.setResult(true)
        await blockManager.setGetBlockHashResult(1, testHash1)

        await rootManager.postRoot(
          1,
          {
            blockHash: testHash1,
            settlementRoot:
              "0x34d4138e7e3683853535aafb2da51ad1574d77e77241ce6f9b9f2965444c4917",
          },
          testHash3
        )
        await rootManager.verifySettlementMerkleProof({
          leaf: {
            withdrawLeaf: {
              recipient: testAddress1,
              amount: { amounts: [1n, 2n, 3n, 4n] },
              startEbn: 1n,
              endEbn: 10n,
            },
            evidenceLeaf: {
              transferCommitment: testHash3,
              ebn: 4n,
            },
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
        const testAddresses = generateDummyAddresses(1)
        const [testHash, testHash2, testHash3] = generateDummyHashes(3)
        await expect(
          rootManager.verifySettlementMerkleProof({
            leaf: {
              withdrawLeaf: {
                recipient: testAddresses[0],
                amount: { amounts: [1n, 2n, 3n, 4n] },
                startEbn: 1n,
                endEbn: 10n,
              },
              evidenceLeaf: {
                transferCommitment: testHash3,
                ebn: 4n,
              },
            },
            index: 0,
            siblings: [testHash, testHash2],
          })
        ).to.be.revertedWithCustomError(
          rootManager,
          "InvalidWithdrawMerkleProof"
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

import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers"
import { ethers, upgrades } from "hardhat"
import { keccak256, randomBytes } from "ethers"
import type {
  Config,
  RootManager,
  BlockManager,
  LiquidityManager,
  Main,
  Withdraw,
  TestToken,
} from "../../typechain-types"

type Signers = {
  deployer: HardhatEthersSigner
  operator: HardhatEthersSigner
  user: HardhatEthersSigner
  illegalUser: HardhatEthersSigner
  dummyConfig: HardhatEthersSigner
  dummyInnerGroup: HardhatEthersSigner
  dummySetter: HardhatEthersSigner
  dummyWithdraw: HardhatEthersSigner
}

export const getSigners = async (): Promise<Signers> => {
  const signers = await ethers.getSigners()
  return {
    deployer: signers[0],
    operator: signers[1],
    user: signers[2],
    illegalUser: signers[3],
    dummyConfig: signers[4],
    dummyInnerGroup: signers[5],
    dummySetter: signers[6],
    dummyWithdraw: signers[7],
  }
}

export const deployAllContracts = async (): Promise<Config> => {
  const signers = await getSigners()
  const config = await getUupsContract<Config>("Config", [])
  const configAddress = await config.getAddress()
  const main = await getUupsContract<Main>("Main", [configAddress])
  const rootManager = await getUupsContract<RootManager>("RootManager", [
    configAddress,
  ])
  const blockManager = await getUupsContract<BlockManager>("BlockManager", [
    configAddress,
  ])
  const liquidityManager = await getUupsContract<LiquidityManager>(
    "LiquidityManager",
    [configAddress]
  )
  const withdraw = await getUupsContract<Withdraw>("Withdraw", [configAddress])
  const testHalo2VerifierFactory = await ethers.getContractFactory(
    "Halo2Verifier"
  )
  const halo2Verifier = await testHalo2VerifierFactory.deploy()
  const testHalo2VerifierkeyFactory = await ethers.getContractFactory(
    "Halo2VerifyingKey"
  )
  const halo2VerifierKey = await testHalo2VerifierkeyFactory.deploy()
  const testVerifierFactory = await ethers.getContractFactory("Verifier")
  const verifier = await testVerifierFactory.deploy(configAddress)
  const erc20s = await generateERC20()
  const erc20Addresses = await Promise.all(
    erc20s.map(async (erc20) => {
      return await erc20.getAddress()
    })
  )
  await config.configure(
    signers.operator.address,
    {
      addresses: erc20Addresses,
    },
    await halo2VerifierKey.getAddress(),
    await halo2Verifier.getAddress(),
    await verifier.getAddress(),
    await rootManager.getAddress(),
    await blockManager.getAddress(),
    await liquidityManager.getAddress(),
    await main.getAddress(),
    await withdraw.getAddress()
  )
  return config
}

export const getUupsContract = async <T>(
  name: string,
  args: unknown[]
): Promise<T> => {
  const testFactory = await ethers.getContractFactory(name)

  const contract = await upgrades.deployProxy(testFactory, args, {
    kind: "uups",
  })
  return contract as unknown as T
}

export const generateDummyAddresses = (index: number): string[] => {
  return Array.from(Array(index).keys()).map(() => {
    return ethers.Wallet.createRandom().address
  })
}

export const generateDummyHashes = (index: number): string[] => {
  return Array.from(Array(index).keys()).map(() => {
    return keccak256(randomBytes(32))
  })
}

export const generateERC20 = async (): Promise<TestToken[]> => {
  return await Promise.all([
    generateERC20Contract(),
    generateERC20Contract(),
    generateERC20Contract(),
    generateERC20Contract(),
  ])
}

const generateERC20Contract = async (): Promise<TestToken> => {
  const tokenFactory = await ethers.getContractFactory("TestToken")
  return await tokenFactory.deploy("0")
}

export const testHash1 =
  "0xe30703a88ca1d002bf2d26b7a5773e9163ce5bc583637565ac374a6c41c5fa62"
export const testHash2 =
  "0x75a868a356b64ed934f0b964d13f7d980ec6d09c109457749f9ff14ce8c43f3c"
export const testHash3 =
  "0x073e5a0dd36bdba15d89588f7776d85fb37ab3bf66e5423abc70a53f8608b9de"
export const testAddress1 = "0x8db97C7cEcE249c2b98bDC0226Cc4C2A57BF52FC"

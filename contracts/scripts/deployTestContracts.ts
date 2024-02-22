import tokens from "../deploy-result/tokens.json"
import { ethers, upgrades } from "hardhat"
import { saveJsonToFile } from "./utils/saveJsonToFile"

async function main() {
  const deployer = (await ethers.getSigners())[0]
  const operator = (await ethers.getSigners())[1]

  const configFactory = await ethers.getContractFactory("Config")
  const config = await upgrades.deployProxy(configFactory, [], {
    kind: "uups",
  })
  const admin = await config.getAddress()
  const halo2VerifyingKeyFactory = await ethers.getContractFactory(
    "Halo2VerifyingKey"
  )
  const halo2VerifyingKey = await halo2VerifyingKeyFactory.deploy()
  const halo2VerifierFactory = await ethers.getContractFactory("Halo2Verifier")
  const halo2Verifier = await halo2VerifierFactory.deploy()
  const verifierFactory = await ethers.getContractFactory("Verifier")
  const verifier = await verifierFactory.deploy(admin)
  const rootManagerFactory = await ethers.getContractFactory("TestRootManager")
  const rootManager = await upgrades.deployProxy(rootManagerFactory, [admin], {
    kind: "uups",
  })
  console.log("admin", admin)
  const blockManagerFactory = await ethers.getContractFactory("BlockManager")
  const blockManager = await upgrades.deployProxy(
    blockManagerFactory,
    [admin],
    {
      kind: "uups",
    }
  )
  const liquidityManagerFactory = await ethers.getContractFactory(
    "LiquidityManager"
  )
  const liquidityManager = await upgrades.deployProxy(
    liquidityManagerFactory,
    [admin],
    {
      kind: "uups",
    }
  )
  const mainFactory = await ethers.getContractFactory("Main")
  const main = await upgrades.deployProxy(mainFactory, [admin], {
    kind: "uups",
  })
  const withdrawFactory = await ethers.getContractFactory("Withdraw")
  const withdraw = await upgrades.deployProxy(withdrawFactory, [admin], {
    kind: "uups",
  })
  const tokensAddresses = {
    addresses: Object.values(tokens) as [string, string, string, string],
  }
  const deployerAddress = await deployer.getAddress()
  const operatorAddress = await operator.getAddress()
  const contracts = {
    deployer: deployerAddress,
    operator: operatorAddress,
    config: await config.getAddress(),
    halo2VerifyingKey: await halo2VerifyingKey.getAddress(),
    halo2Verifier: await halo2Verifier.getAddress(),
    verifier: await verifier.getAddress(),
    blockManager: await blockManager.getAddress(),
    rootManager: await rootManager.getAddress(),
    liquidityManager: await liquidityManager.getAddress(),
    main: await main.getAddress(),
    withdraw: await withdraw.getAddress(),
  }

  saveJsonToFile(
    "./deploy-result/contracts.json",
    JSON.stringify(contracts, null, 2)
  )

  await config.waitForDeployment()
  await config.configure(
    operatorAddress,
    tokensAddresses,
    contracts.halo2VerifyingKey,
    contracts.halo2Verifier,
    contracts.verifier,
    contracts.rootManager,
    contracts.blockManager,
    contracts.liquidityManager,
    contracts.main,
    contracts.withdraw
  )
  console.log("Config contract deployed to:", contracts.config)
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})

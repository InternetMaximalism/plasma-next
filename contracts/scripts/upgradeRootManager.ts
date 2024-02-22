import { ethers, upgrades } from "hardhat"
import contracts from "../deploy-result/contracts.json"
import tokens from "../deploy-result/tokens.json"
import { saveJsonToFile } from "./utils/saveJsonToFile"
import { Config__factory } from "../typechain-types"

async function main() {
  const signers = await ethers.getSigners()
  const operator = signers[0]

  const config = Config__factory.connect(contracts.config, operator)

  const newRootManagerFactory = await ethers.getContractFactory("RootManager")
  console.log("Upgrading RootManager...")
  const newRootManager = await upgrades.deployProxy(
    newRootManagerFactory,
    [contracts.config],
    {
      kind: "uups",
    }
  )

  const newContracts = {
    ...contracts,
    rootManager: await newRootManager.getAddress(),
  }

  saveJsonToFile(
    "./deploy-result/contracts.json",
    JSON.stringify(newContracts, null, 2)
  )

  const tokensAddresses = {
    addresses: Object.values(tokens) as [string, string, string, string],
  }
  await config.configure(
    contracts.operator,
    tokensAddresses,
    contracts.halo2VerifyingKey,
    contracts.halo2Verifier,
    contracts.verifier,
    newContracts.rootManager,
    contracts.blockManager,
    contracts.liquidityManager,
    contracts.main,
    contracts.withdraw
  )
  console.log("New Contract Deployed To:", newContracts.rootManager)
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})

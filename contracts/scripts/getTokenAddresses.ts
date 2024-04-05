import { ethers } from "hardhat"
import { saveJsonToFile } from "./utils/saveJsonToFile"
import { Config__factory } from "../typechain-types"

import contracts from "../deploy-result/contracts.json"

async function main() {
  const configAddress = contracts.config
  const deployer = (await ethers.getSigners())[0]
  console.log("deployer", deployer)

  const config = Config__factory.connect(configAddress, deployer)

  const addresses = await config.getAddressBook()
  console.log("addresses", addresses)
  const tokenAddresses: { [key: string]: string } = {}
  for (let i = 0; i < addresses.tokenAddresses.length; i++) {
    tokenAddresses[`token${i}`] = addresses.tokenAddresses.addresses[i]
  }

  saveJsonToFile(
    "./deploy-result/tokens.json",
    JSON.stringify(tokenAddresses, null, 2)
  )
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})

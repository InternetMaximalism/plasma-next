import { ethers } from "hardhat"
import { AddressLike } from "ethers"
import { saveJsonToFile } from "./utils/saveJsonToFile"
import { TestToken__factory } from "../typechain-types"

async function main() {
  const numAssets = 4
  const tokenAddresses: Record<string, AddressLike> = {}
  for (let i = 0; i < numAssets; i++) {
    const tokenFactory = await ethers.getContractFactory("TestToken")
    const token = await tokenFactory.deploy(`Dai Stablecoin`)
    await token.waitForDeployment()
    tokenAddresses[`token${i}`] = await token.getAddress()
  }
  saveJsonToFile(
    "./deploy-result/tokens.json",
    JSON.stringify(tokenAddresses, null, 2)
  )

  const deployer = (await ethers.getSigners())[0]
  const operator = (await ethers.getSigners())[1]
  for (let i = 0; i < numAssets; i++) {
    const token = TestToken__factory.connect(
      tokenAddresses[`token${i}`] as string,
      deployer
    )
    const tx = await token.transfer(operator.address, ethers.parseEther("5000"))
    await tx.wait()
    console.log(`transfered 5000 token${i} to ${operator.address}`)
  }
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})

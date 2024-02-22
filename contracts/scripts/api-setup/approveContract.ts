import { ethers } from "hardhat"
import { AddressLike } from "ethers"

import { Config__factory, TestToken__factory } from "../../typechain-types"

import tokens from "../../deploy-result/tokens.json"
import contracts from "../../deploy-result/contracts.json"

async function main() {
  const tokenAddressArray = Object.values(tokens) as AddressLike[]

  const signers = await ethers.getSigners()
  const operator = signers[1]
  const operatorAddress = await operator.getAddress()
  console.log("signer", operatorAddress)

  const config = Config__factory.connect(contracts.config, operator)
  const addressBook = await config.getAddressBook()
  console.log("addressBook", addressBook)

  for (let i = 0; i < tokenAddressArray.length; i++) {
    if (tokenAddressArray[i] === ethers.ZeroAddress) {
      continue
    }

    const token = TestToken__factory.connect(
      tokenAddressArray[i] as string,
      operator
    )
    const balance = await token.balanceOf(operatorAddress)
    console.log(`token${i} balance: ${balance.toString()}`)
    const tx = await token.approve(
      addressBook.liquidityManager,
      ethers.MaxUint256
    )
    await tx.wait()
  }
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})

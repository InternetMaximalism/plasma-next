import tokens from "../../deploy-result/tokens.json"
import { TestToken__factory } from "../../typechain-types"
import { Address, AssetsFormatted } from "../types/common"
import { ethers } from "hardhat"

export async function getTokenBalance(user: Address): Promise<AssetsFormatted> {
  const amounts: string[] = []
  for (const token of Object.values(tokens)) {
    const tokenContract = TestToken__factory.connect(token, ethers.provider)
    const balance = await tokenContract.balanceOf(user)
    amounts.push(ethers.formatEther(balance))
  }
  return { amounts } as AssetsFormatted
}

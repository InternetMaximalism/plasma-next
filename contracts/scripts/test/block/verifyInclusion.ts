import { ethers } from "hardhat"
import { getBlocks, prepareRoots } from "../../utils/block"

async function main() {
  const blockManagerFactory = await ethers.getContractFactory(
    "TestVerifyInclusion"
  )
  const blockManager = await blockManagerFactory.deploy()
  const randomRoots = Array.from({ length: 100 }, () => ethers.randomBytes(32))
  const tx = await blockManager.postBlocks(randomRoots)
  const receipt = await tx.wait()
  console.log("Gas used:", receipt?.gasUsed.toString())
  const blocks = await getBlocks(blockManager)
  const lastBlockNumber = await blockManager.lastBlockNumber()
  const { transferRoots, totalDepositHashes } = prepareRoots(
    0,
    Number(lastBlockNumber),
    blocks
  )
  await blockManager.verifyInclusion(
    blocks[0].blockNumber,
    blocks[1].prevBlockHash,
    transferRoots,
    totalDepositHashes
  )
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})

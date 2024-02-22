import { ethers } from "hardhat"
import { Block } from "../../types/common"
import { getBlockHash } from "../../utils/block"

async function main() {
  const block = {
    prevBlockHash:
      "0x2512ccb4a4c3ca5d8a4d344a8ffeb04dee61b70aaaa75103c594ca88f97f0f0c",
    transferRoot:
      "0xe0467e272ecce78e726b745712defff3e4954139e71b18f7ab5c91b3398608d4",
    totalDeposit: {
      amounts: [10000000000000000000n, 0n, 0n, 0n],
    },
    blockNumber: 7,
  } as Block
  const blockHash = getBlockHash(block)
  console.log(blockHash)

  // Expected output:
  const testGetBlockHashFactory = await ethers.getContractFactory(
    "TestGetBlockHash"
  )
  const testGetBlockHash = await testGetBlockHashFactory.deploy()
  const res = await testGetBlockHash.getBlockHash(block)
  console.log(res)
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})

import contracts from "../../../deploy-result/contracts.json"
import { ethers } from "hardhat"
import { createPaymentService } from "../../service/paymentService"
import { getTokenBalance } from "../../utils/getTokenBalance"
import { zeroAssets } from "../../utils/assets"

async function main() {
  const signers = await ethers.getSigners()
  const operator = signers[1]
  const user = signers[2]
  const userAddress = await user.getAddress()
  console.log("user:", userAddress)

  const service = await createPaymentService(operator, user, contracts.config)
  await service.approveAll()
  await service.airdrop(100n, 0)
  await service.airdrop(200n, 1)

  console.log("before:", await getTokenBalance(userAddress))
  const settlementMerkleProof = await service.postSettlementRoot()
  const payment = service.payments[service.payments.length - 1]
  console.log(payment)
  console.log(settlementMerkleProof)
  console.log(settlementMerkleProof.leaf.withdrawLeaf.amount)
  console.log(zeroAssets())
  await service.main.closeChannel(payment, settlementMerkleProof, zeroAssets())
  console.log("after:", await getTokenBalance(userAddress))
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})

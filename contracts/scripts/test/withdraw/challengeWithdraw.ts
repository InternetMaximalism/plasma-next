import contracts from "../../../deploy-result/contracts.json"
import { ethers } from "hardhat"
import { createPaymentService } from "../../service/paymentService"
import { getTokenBalance } from "../../utils/getTokenBalance"
import { zeroAssets } from "../../utils/assets"

async function main() {
  const signers = await ethers.getSigners()
  const operator = signers[1]
  const user = signers[2]
  const someone = signers[3].address

  const userAddress = await user.getAddress()
  console.log("operator:", await operator.getAddress())
  console.log("user:", userAddress)

  const service = await createPaymentService(operator, user, contracts.config)
  await service.approveAll()
  await service.airdrop(100n, 0)
  await service.send(someone, 50n, 0)
  await service.airdrop(200n, 1)

  console.log("prevPayment:", await service.getPrevPayment())

  // do withdraw request
  console.log("before request withdraw:", await getTokenBalance(userAddress))
  const settlementMerkleProof = await service.postSettlementRoot()
  await service.withdraw
    .connect(user)
    .requestWithdrawal(settlementMerkleProof, zeroAssets()) // request
  const latestPaymentWithSignature =
    service.payments[service.payments.length - 1]
  await service.withdraw.challengeWithdrawal(
    userAddress,
    latestPaymentWithSignature,
    settlementMerkleProof
  )
  console.log("after challenge withdraw:", await getTokenBalance(userAddress))
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})

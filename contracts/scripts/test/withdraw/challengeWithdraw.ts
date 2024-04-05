import contracts from "../../../deploy-result/contracts.json"
import { ethers } from "hardhat"
import { createPaymentService } from "../../service/paymentService"
import { getTokenBalance } from "../../utils/getTokenBalance"
import { zeroAssets } from "../../utils/assets"

async function main() {
  const signers = await ethers.getSigners()
  const operator = signers[1]
  const user = signers[2]
  const someoneAddress = signers[3].address

  const userAddress = await user.getAddress()
  console.log("operator:", await operator.getAddress())
  console.log("user:", userAddress)

  const service = await createPaymentService(
    operator,
    user,
    contracts.config,
    contracts.defaultZKPTLC
  )
  await service.approveAll()
  await service.airdrop(100n, 0)
  await service.send(someoneAddress, 50n, 0)

  console.log("before:", await getTokenBalance(userAddress))
  const withdrawalRequest = await service.withdraw.withdrawalRequests(
    userAddress
  )
  // if withdrawal request is empty, then request withdrawal
  if (withdrawalRequest.requestedAt === 0n) {
    const { withdrawProof, wrapPis, blockNumber } =
      await service.zkpService.computeWithdrawProof(service.spentAirdrops)
    await service.postRoot(blockNumber, wrapPis)
    await service.withdraw
      .connect(user)
      .requestWithdrawal(withdrawProof, zeroAssets()) // request
  }
  await service.challengeWithdrawal() // challenge
  console.log("after:", await getTokenBalance(userAddress))
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})

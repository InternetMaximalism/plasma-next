import contracts from "../../../deploy-result/contracts.json"
import { ethers } from "hardhat"
import { createPaymentService } from "../../service/paymentService"
import { getTokenBalance } from "../../utils/getTokenBalance"

async function main() {
  const signers = await ethers.getSigners()
  const operator = signers[1]
  const user = signers[2]
  const someoneAddress = signers[3].address
  const userAddress = await user.getAddress()
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
  await service.closeChannelForce()
  console.log("after:", await getTokenBalance(userAddress))
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})

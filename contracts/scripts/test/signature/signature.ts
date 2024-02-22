import { ethers } from "hardhat"
import { getUniqueIdentifier, signPayment } from "../../utils/payment"
import { Payment } from "../../types/common"

async function main() {
  const signers = await ethers.getSigners()
  const operator = signers[0]
  const user = signers[1]

  const TestSignature = await ethers.getContractFactory("TestSignature")
  const testSignature = await TestSignature.deploy()

  const uniqueIdentifier = await getUniqueIdentifier(
    await testSignature.getAddress()
  )

  const payment = {
    uniqueIdentifier,
    user: await user.getAddress(),
    round: 0,
    nonce: 0,
    userBalance: {
      amounts: [100n, 200n, 300n, 500n],
    },
    operatorBalance: {
      amounts: [100n, 200n, 300n, 500n],
    },
    airdropped: {
      amounts: [100n, 200n, 300n, 500n],
    },
    spentDeposit: {
      amounts: [100n, 200n, 300n, 500n],
    },
    latestTransferCommitment: ethers.ZeroHash,
    latestEbn: 0n,
    customData: "0x",
  } as Payment

  const ps = await signPayment(user, operator, payment)

  await testSignature.verifyPaymentSignature(
    await operator.getAddress(),
    await user.getAddress(),
    ps
  )
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})

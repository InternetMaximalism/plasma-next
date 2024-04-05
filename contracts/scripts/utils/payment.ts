import { Signer } from "ethers"
import {
  Address,
  Bytes32,
  Payment,
  PaymentWithSignature,
  U32,
  U64,
} from "../types/common"
import { ethers } from "hardhat"

export function initialPayment(
  user: Address,
  curEbn: U64,
  round: U32
): Payment {
  return {
    user,
    round,
    nonce: 0,
    userBalance: {
      amounts: [0n, 0n, 0n, 0n],
    },
    operatorBalance: {
      amounts: [0n, 0n, 0n, 0n],
    },
    airdropped: {
      amounts: [0n, 0n, 0n, 0n],
    },
    spentDeposit: {
      amounts: [0n, 0n, 0n, 0n],
    },
    latestEbn: curEbn,
    zkptlcAddress: ethers.ZeroAddress,
    zkptlcInstance: ethers.ZeroHash,
  }
}

export async function getUniqueIdentifier(contractAddress: Address) {
  const chainId = (await ethers.provider.getNetwork()).chainId
  const uniqueIdentifier = ethers.solidityPackedKeccak256(
    ["uint256", "address"],
    [chainId, contractAddress]
  )
  return uniqueIdentifier
}

export async function signPayment(
  user: Signer,
  operator: Signer,
  uniqueIdentifier: Bytes32,
  payment: Payment
): Promise<PaymentWithSignature> {
  const hash = ethers.solidityPackedKeccak256(
    [
      "bytes32",
      "address",
      "uint32",
      "uint32",
      "uint256[4]",
      "uint256[4]",
      "uint256[4]",
      "uint256[4]",
      "uint64",
      "address",
      "bytes32",
    ],
    [
      uniqueIdentifier,
      payment.user,
      payment.round,
      payment.nonce,
      payment.userBalance.amounts,
      payment.operatorBalance.amounts,
      payment.airdropped.amounts,
      payment.spentDeposit.amounts,
      payment.latestEbn,
      payment.zkptlcAddress,
      payment.zkptlcInstance,
    ]
  )

  const userSignature = await user.signMessage(ethers.getBytes(hash))
  const operatorSignature = await operator.signMessage(ethers.getBytes(hash))

  return { payment, userSignature, operatorSignature }
}

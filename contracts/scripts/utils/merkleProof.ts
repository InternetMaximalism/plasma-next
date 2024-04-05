import { ethers } from "hardhat"
import {
  Address,
  Assets,
  Bytes32,
  U256,
  U64,
  EvidenceMerkleProof,
  WithdrawMerkleProof,
} from "../types/common"
import { getRandomInt } from "./random"
import { hashEvidenceLeaf, hashWithdrawLeaf } from "./leaf"

export function generateDummyWithdrawMerkleProof(
  height: number,
  recipient: Address,
  assets: Assets,
  startEbn: U64,
  endEbn: U64
): WithdrawMerkleProof {
  const siblings = []
  for (let i = 0; i < height; i++) {
    siblings.push(ethers.hexlify(ethers.randomBytes(32)))
  }
  const index = getRandomInt(0, 2 ** height - 1)
  const withdrawMerkleProof = {
    leaf: {
      recipient,
      amount: assets,
      startEbn,
      endEbn,
    },
    index: BigInt(index),
    siblings,
  }
  return withdrawMerkleProof
}

export function generateDummyEvidenceMerkleProof(
  height: number,
  transferCommitment: Bytes32,
  ebn: U64
): EvidenceMerkleProof {
  const siblings = []
  for (let i = 0; i < height; i++) {
    siblings.push(ethers.hexlify(ethers.randomBytes(32)))
  }
  const index = getRandomInt(0, 2 ** height - 1)
  const evidenceMerkleProof = {
    leaf: {
      transferCommitment,
      ebn,
    },
    index: BigInt(index),
    siblings,
  }
  return evidenceMerkleProof
}

export function getWithdrawRoot(w: WithdrawMerkleProof) {
  return getRoot(w.index, hashWithdrawLeaf(w.leaf), w.siblings)
}

export function getEvidenceRoot(e: EvidenceMerkleProof) {
  return getRoot(e.index, hashEvidenceLeaf(e.leaf), e.siblings)
}

function getRoot(index: U256, hash: Bytes32, siblings: Bytes32[]): Bytes32 {
  let computedHash = hash
  for (let i = 0; i < siblings.length; i++) {
    const sibling = siblings[i]
    if (index % 2n == 0n) {
      computedHash = ethers.solidityPackedKeccak256(
        ["bytes32", "bytes32"],
        [computedHash, sibling]
      )
    } else {
      computedHash = ethers.solidityPackedKeccak256(
        ["bytes32", "bytes32"],
        [sibling, computedHash]
      )
    }
    index = index >> 1n
  }
  return computedHash
}

// function padToPowerOf2(leaves: Bytes32[]): Bytes32[] {
//   const length = leaves.length
//   const nextPowerOf2 = 2 ** Math.ceil(Math.log2(length))
//   const paddedLeaves = leaves.concat(
//     Array(nextPowerOf2 - length).fill(ethers.ZeroHash)
//   )
//   return paddedLeaves
// }

// function computeRootFromLeaves(leaves: Bytes32[]): Bytes32 {
//   // pad leaves to be the power of 2
//   let paddedLeaves = padToPowerOf2(leaves)
//   const height = Math.log2(paddedLeaves.length)
//   for (let i = 0; i < height; i++) {
//     const nextPaddedLeaves = []
//     for (let j = 0; j < paddedLeaves.length; j += 2) {
//       nextPaddedLeaves.push(
//         ethers.solidityPackedKeccak256(
//           ["bytes32", "bytes32"],
//           [paddedLeaves[j], paddedLeaves[j + 1]]
//         )
//       )
//     }
//     paddedLeaves = nextPaddedLeaves
//   }
//   return paddedLeaves[0]
// }

import { ethers } from "hardhat"
import {
  Address,
  Assets,
  Bytes32,
  U256,
  U64,
  SettlementMerkleProof,
} from "../types/common"
import { getRandomInt } from "./random"
import { hashSettlementLeaf } from "./leaf"

export function generateDummySettlementMerkleProof(
  height: number,
  recipient: Address,
  assets: Assets,
  startEbn: U64,
  endEbn: U64,
  transferCommitment: Bytes32,
  ebn: U64
): SettlementMerkleProof {
  const siblings = []
  for (let i = 0; i < height; i++) {
    siblings.push(ethers.hexlify(ethers.randomBytes(32)))
  }
  const index = getRandomInt(0, 2 ** height - 1)
  const settlementMerkleProof = {
    leaf: {
      withdrawLeaf: {
        recipient,
        amount: assets,
        startEbn,
        endEbn,
      },
      evidenceLeaf: {
        transferCommitment,
        ebn,
      },
    },
    index: BigInt(index),
    siblings,
  }
  return settlementMerkleProof
}

export function getSettlementRoot(w: SettlementMerkleProof) {
  return getRoot(w.index, hashSettlementLeaf(w.leaf), w.siblings)
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

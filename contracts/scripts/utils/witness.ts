import { AbiCoder } from "ethers"
import { Bytes, EvidenceMerkleProof, Transfer } from "../types/common"

interface Witness {
  transfer: Transfer
  evidenceProof: EvidenceMerkleProof
}

export function encodeWitness(witness: Witness): Bytes {
  const abi = new AbiCoder()
  return abi.encode(
    ["((address,uint256,uint32,uint32), ((bytes32,uint64),uint256,bytes32[]))"],
    [
      [
        [
          witness.transfer.recipient,
          witness.transfer.amount,
          witness.transfer.assetId,
          witness.transfer.nonce,
        ],
        [
          [
            witness.evidenceProof.leaf.transferCommitment,
            witness.evidenceProof.leaf.ebn,
          ],
          witness.evidenceProof.index,
          witness.evidenceProof.siblings,
        ],
      ],
    ]
  )
}

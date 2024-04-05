import { AddressLike } from "ethers"

export type U256 = bigint

export type U64 = bigint

export type U32 = number

export interface Assets {
  amounts: [U256, U256, U256, U256]
}

export interface AssetsFormatted {
  amounts: [string, string, string, string]
}

export type Address = AddressLike

export type Bytes = string

export type Bytes32 = string

export interface EvidenceLeaf {
  transferCommitment: Bytes32
  ebn: U64
}

export interface WithdrawLeaf {
  recipient: Address
  amount: Assets
  startEbn: U64
  endEbn: U64
}

export interface WithdrawMerkleProof {
  leaf: WithdrawLeaf
  index: U256
  siblings: Bytes32[]
}

export interface EvidenceMerkleProof {
  leaf: EvidenceLeaf
  index: U256
  siblings: Bytes32[]
}

export interface WrapPublicInputs {
  blockHash: Bytes32
  evidenceRoot: Bytes32
  withdrawRoot: Bytes32
}

export interface Payment {
  user: Address
  round: U32
  nonce: U32
  userBalance: Assets
  operatorBalance: Assets
  airdropped: Assets
  spentDeposit: Assets
  latestEbn: U64
  zkptlcAddress: Address
  zkptlcInstance: Bytes32
}

export interface PaymentWithSignature {
  payment: Payment
  userSignature: Bytes
  operatorSignature: Bytes
}

export interface Transfer {
  recipient: Address
  amount: U256
  assetId: U32
  nonce: U32
}

export interface TransferInfo {
  transfer: Transfer
  transferIndex: U32
  transferMerkleProof: Bytes32[]
  block: Block
}

export interface Block {
  prevBlockHash: Bytes32
  transferRoot: Bytes32
  totalDepositHash: Bytes32
  blockNumber: U32
}

export interface BlockWithAmounts {
  block: Block
  totalDeposit: Assets
}

export interface BlockInfo {
  block: BlockWithAmounts
  transferInfo: TransferInfo[]
}

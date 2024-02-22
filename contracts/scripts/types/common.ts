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

export interface SettlementLeaf {
  withdrawLeaf: WithdrawLeaf
  evidenceLeaf: EvidenceLeaf
}

export interface SettlementMerkleProof {
  leaf: SettlementLeaf
  index: U256
  siblings: Bytes32[]
}

export interface WrapPublicInputs {
  blockHash: Bytes32
  settlementRoot: Bytes32
}

export interface Payment {
  uniqueIdentifier: Bytes32
  user: Address
  round: U32
  nonce: U32
  userBalance: Assets
  operatorBalance: Assets
  airdropped: Assets
  spentDeposit: Assets
  latestTransferCommitment: Bytes32
  latestEbn: U64
  customData: Bytes
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
}

export interface Block {
  prevBlockHash: Bytes32
  transferRoot: Bytes32
  totalDeposit: Assets
  blockNumber: U32
}

# Plasma Next Contracts

The contracts of Plasma Next mainly consist of the following five components:

- [Block Manager](./contracts/block-manager/) is a contract that manages the blocks of Plasma Next. It accepts block submissions from the Operator.
- [Root Manager](./contracts/root-manager/) is a contract that manages the settlement roots. A settlement root is the Merkle root of a settlement tree batched with Zero-Knowledge Proofs (ZKPs), which is necessary for the settlement of payment channels or for emergency withdrawals when the operator stops. The Root Manager verifies the consistency between the ZKP related to the settlement root and the block hash obtained from the Block Manager. It also has the role of verifying the Merkle proofs of the settlement tree.
- [Liquidity Manager](./contracts/liquidity-manager/) is a contract that collectively manages the Layer 1 liquidity of Plasma Next. It supports other contracts in receiving and sending tokens.
- [Main Payment Channel](./contracts/payment-channel/main/) is a contract that manages the state of payment channels and the settlement of channels. In Plasma Next, on-chain transactions are not necessary when opening a payment channel. It is only called when closing a channel or withdrawing to L1.
- [Withdraw](./contracts/payment-channel/withdraw/) is used by users to withdraw assets from Plasma Next on their own in case the Operator stops.

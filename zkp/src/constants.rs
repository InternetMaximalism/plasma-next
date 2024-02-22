pub const TRANSFER_TREE_HEIGHT: usize = 11;
pub const NUM_ASSETS: usize = 4;

// Cyclic proof constants. They depends on the `TRANSFER_TREE_HEIGHT` and `NUM_ASSETS`.
pub const WITHDRAW_PADDING_DEGREE: usize = 13;
pub const BALANCE_PROOF_PADDING_DEGREE: usize = 15;
pub const BLOCK_TREE_PADDING_DEGREE: usize = 15;
pub const WITHDRAW_TREE_PADDING_DEGREE: usize = 15;
pub const EVIDENCE_TREE_PADDING_DEGREE: usize = 15;
pub const SETTLEMENT_TREE_PADDING_DEGREE: usize = 15;

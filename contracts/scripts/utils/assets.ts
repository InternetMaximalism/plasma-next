import { Assets, Transfer, U256, U32 } from "../types/common"

export function zeroAssets(): Assets {
  return {
    amounts: [0n, 0n, 0n, 0n],
  }
}

export function addAssets(left: Assets, right: Assets): Assets {
  return {
    amounts: [
      left.amounts[0] + right.amounts[0],
      left.amounts[1] + right.amounts[1],
      left.amounts[2] + right.amounts[2],
      left.amounts[3] + right.amounts[3],
    ],
  }
}

export function subAssets(left: Assets, right: Assets): Assets {
  return {
    amounts: [
      left.amounts[0] - right.amounts[0],
      left.amounts[1] - right.amounts[1],
      left.amounts[2] - right.amounts[2],
      left.amounts[3] - right.amounts[3],
    ],
  }
}

export function addSingleAsset(assets: Assets, amount: U256, assetId: U32) {
  const newAssets = { ...assets }
  newAssets.amounts[Number(assetId)] += amount
  return newAssets
}

export function subSingleAsset(assets: Assets, amount: U256, assetId: U32) {
  const newAssets = { ...assets }
  newAssets.amounts[Number(assetId)] -= amount
  return newAssets
}

export function isLe(left: Assets, right: Assets): boolean {
  return (
    left.amounts[0] <= right.amounts[0] &&
    left.amounts[1] <= right.amounts[1] &&
    left.amounts[2] <= right.amounts[2] &&
    left.amounts[3] <= right.amounts[3]
  )
}

export function sumTransfers(transfers: Transfer[]): Assets {
  return transfers.reduce(
    (acc, t) => addSingleAsset(acc, t.amount, t.assetId),
    zeroAssets()
  )
}

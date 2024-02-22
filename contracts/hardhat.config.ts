import { type HardhatUserConfig } from "hardhat/config"
import "@nomicfoundation/hardhat-toolbox"
import "hardhat-contract-sizer"
import "dotenv/config"
import "@openzeppelin/hardhat-upgrades"
import "hardhat-docgen"

import { testAccounts } from "./test-accounts"

// eslint-disable-next-line @typescript-eslint/naming-convention
const {
  PRIVATE_KEY,
  OPERATOR_PRIVATE_KEY,
  ALCHEMY_KEY,
  ALCHEMY_GOERLI_URL,
  SEPOLIA_RPC_URL,
  SCROLL_RPC_URL,
  ARBITRUM_SEPOLIA_RPC_URL,
  ARBITRUM_RPC_URL,
  ETHERSCAN_API_KEY,
} = process.env
const goerliRpcUrl = ALCHEMY_GOERLI_URL ?? ""
const sepoliaRpcUrl = SEPOLIA_RPC_URL
  ? SEPOLIA_RPC_URL
  : ALCHEMY_KEY
  ? `https://eth-sepolia.g.alchemy.com/v2/${ALCHEMY_KEY}`
  : ""
const scrollRpcUrl = SCROLL_RPC_URL ?? ""
const arbitrumSepoliaRpcUrl = ARBITRUM_SEPOLIA_RPC_URL
  ? ARBITRUM_SEPOLIA_RPC_URL
  : ALCHEMY_KEY
  ? `https://arb-sepolia.g.alchemy.com/v2/${ALCHEMY_KEY}`
  : ""
const arbitrumRpcUrl = ARBITRUM_RPC_URL
  ? ARBITRUM_RPC_URL
  : ALCHEMY_KEY
  ? `https://arb-mainnet.g.alchemy.com/v2/${ALCHEMY_KEY}`
  : ""
const deployerPrivateKey =
  PRIVATE_KEY ??
  "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
const operatorPrivateKey =
  OPERATOR_PRIVATE_KEY ??
  PRIVATE_KEY ??
  "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
const etherscanApiKey = ETHERSCAN_API_KEY ?? ""

const additionalAccounts = testAccounts.map((account) => ({
  privateKey: account.privateKey,
  balance: "10000000000000000000000",
}))
const deployerAccount = {
  privateKey: deployerPrivateKey,
  balance: "10000000000000000000000",
}
const operatorAccount = {
  privateKey: operatorPrivateKey,
  balance: "10000000000000000000000",
}

const config: HardhatUserConfig = {
  solidity: {
    version: "0.8.23",
    settings: {
      optimizer: {
        enabled: true,
      },
    },
  },
  // contractSizer: {
  //   alphaSort: true,
  //   runOnCompile: true,
  //   disambiguatePaths: false,
  // },
  networks: {
    localhost: {
      chainId: 31337,
      url: "http://127.0.0.1:8545",
    },
    goerli: {
      url: goerliRpcUrl,
      accounts: [deployerPrivateKey, operatorPrivateKey],
    },
    sepolia: {
      url: sepoliaRpcUrl,
      accounts: [deployerPrivateKey, operatorPrivateKey],
    },
    arbitrumSepolia: {
      url: arbitrumSepoliaRpcUrl,
      accounts: [deployerPrivateKey, operatorPrivateKey],
    },
    arbitrum: {
      url: arbitrumRpcUrl,
      accounts: [deployerPrivateKey, operatorPrivateKey],
    },
    scroll: {
      url: scrollRpcUrl,
      accounts: [deployerPrivateKey, operatorPrivateKey],
    },
    hardhat: {
      accounts: [deployerAccount, operatorAccount, ...additionalAccounts],
    },
  },
  etherscan: {
    apiKey: etherscanApiKey,
  },
}

export default config

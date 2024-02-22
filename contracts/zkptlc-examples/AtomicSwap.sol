/// SPDX-License-Identifier: MIT
pragma solidity 0.8.23;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IAdditionalZKPTLC} from "../contracts/common-interface/IAdditionalZKPTLC.sol";

/**
 * @title AtomicSwap
 * @dev execute atomic swap between operator to user on the base chain
 * and user to operator on Plasma Next using additional ZKPTLC.
 */
contract AtomicSwap is IAdditionalZKPTLC {
    using SafeERC20 for IERC20;

    address public operator;
    mapping(bytes32 => bool) isDone;

    constructor() {
        operator = msg.sender;
    }

    /// @dev Deposit token to the contract by operator
    function deposit(address tokenAddress, uint256 amount) external {
        IERC20(tokenAddress).safeTransferFrom(
            msg.sender,
            address(this),
            amount
        );
    }

    /// @dev Send token to user
    function send(
        address to,
        address tokenAddress,
        uint256 amount,
        uint256 nonce
    ) external {
        require(msg.sender == operator, "Invalid operator");
        IERC20(tokenAddress).safeTransfer(to, amount);
        // Mark the transfer as done
        bytes32 commitment = keccak256(
            abi.encodePacked(to, tokenAddress, amount, nonce)
        );
        isDone[commitment] = true;
    }

    /// @dev Verify that the transfer is done
    function verifyAdditionalZKPTLC(
        bytes memory customData
    ) external view returns (bool) {
        bytes32 commitment = abi.decode(customData, (bytes32));
        return isDone[commitment];
    }
}

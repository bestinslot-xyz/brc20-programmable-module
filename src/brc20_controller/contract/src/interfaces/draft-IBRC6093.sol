// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @dev Standard BRC-20 Controller Errors
 */
interface IBRC20_ControllerErrors {
    /**
     * @dev Indicates an error related to the current `balance` of a `sender`. Used in transfers.
     * @param ticker Ticker of tokens.
     * @param sender Address whose tokens are being transferred.
     * @param balance Current balance for the interacting account.
     * @param needed Minimum amount required to perform a transfer.
     */
    error BRC20InsufficientBalance(bytes ticker, address sender, uint256 balance, uint256 needed);

    /**
     * @dev Indicates a failure with the token `sender`. Used in transfers.
     * @param ticker Ticker of tokens.
     * @param sender Address whose tokens are being transferred.
     */
    error BRC20InvalidSender(bytes ticker, address sender);

    /**
     * @dev Indicates a failure with the token `receiver`. Used in transfers.
     * @param ticker Ticker of tokens.
     * @param receiver Address to which tokens are being transferred.
     */
    error BRC20InvalidReceiver(bytes ticker, address receiver);

    /**
     * @dev Indicates a failure with the `spender`’s `allowance`. Used in transfers.
     * @param ticker Ticker of tokens.
     * @param spender Address that may be allowed to operate on tokens without being their owner.
     * @param allowance Amount of tokens a `spender` is allowed to operate with.
     * @param needed Minimum amount required to perform a transfer.
     */
    error BRC20InsufficientAllowance(bytes ticker, address spender, uint256 allowance, uint256 needed);

    /**
     * @dev Indicates a failure with the `approver` of a token to be approved. Used in approvals.
     * @param ticker Ticker of tokens.
     * @param approver Address initiating an approval operation.
     */
    error BRC20InvalidApprover(bytes ticker, address approver);

    /**
     * @dev Indicates a failure with the `spender` to be approved. Used in approvals.
     * @param ticker Ticker of tokens.
     * @param spender Address that may be allowed to operate on tokens without being their owner.
     */
    error BRC20InvalidSpender(bytes ticker, address spender);
}
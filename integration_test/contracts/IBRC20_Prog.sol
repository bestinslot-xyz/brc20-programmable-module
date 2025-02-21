// SPDX-License-Identifier: MIT

pragma solidity ^0.8.19;

/**
 * @dev BIP322 verification method
 */
interface IBIP322_Verifier {
    function verify(
        address addr,
        string calldata message_base64,
        string calldata signature_base64
    ) external returns (bool);
}

/**
 * @dev Get non-module BRC-20 balance of a given Bitcoin wallet script and BRC-20 ticker.
 */
interface IBRC20_Balance {    
    function balanceOf(string calldata ticker, address account) external view returns (uint256);
}

/**
 * Get Bitcoin transaction details using tx ids.
 */
interface IBTC_Transaction {
    function getTxDetails(string calldata txid) external view returns (string memory, uint256, uint256);
}

/**
 * @dev Interface for the BRC-20 Prog helper functions.
 */
interface IBRC20_Prog {
    /**
     * @dev Verifies BIP322 signature, given address, message and the signature.
     */
    function verify(
        address addr,
        string calldata message_base64,
        string calldata signature_base64
    ) external returns (bool);

    /**
     * @dev Get non-module BRC-20 balance of a given Bitcoin wallet script and BRC-20 ticker.
     */
    function balanceOf(string calldata ticker, address account) external view returns (uint256);

    /**
     * @dev Get Bitcoin transaction details using tx ids.
     */
    function getTxDetails(string calldata txid) external view returns (string memory, uint256, uint256);
}

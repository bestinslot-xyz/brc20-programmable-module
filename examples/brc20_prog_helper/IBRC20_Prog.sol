// SPDX-License-Identifier: MIT

pragma solidity ^0.8.19;

/**
 * @dev BIP322 verification method
 */
interface IBIP322_Verifier {
    function verify(
        string calldata addr,
        string calldata message_base64,
        string calldata signature_base64
    ) external returns (bool);
}

/**
 * @dev Get non-module BRC-20 balance of a given Bitcoin wallet script and BRC-20 ticker.
 */
interface IBRC20_Balance {
    function balanceOf(
        string calldata ticker,
        string calldata address_pkscript
    ) external view returns (uint256);
}

/**
 * Get Bitcoin transaction details using tx ids.
 */
interface IBTC_Transaction {
    function getTxDetails(
        string calldata txid
    )
        external
        view
        returns (
            uint256 block_height,
            string memory vin_txid,
            uint256 vin_vout,
            string memory vin_scriptPubKey_hex,
            uint256 vin_value,
            string memory vout_scriptPubKey_hex,
            uint256 vout_value
        );
}

/**
 * @dev Interface for the BRC-20 Prog helper functions.
 */
interface IBRC20_Prog {
    /**
     * @dev Verifies BIP322 signature, given address, message and the signature.
     */
    function verifyBIP322Signature(
        string calldata addr,
        string calldata message_base64,
        string calldata signature_base64
    ) external returns (bool verified);

    /**
     * @dev Get non-module BRC-20 balance of a given Bitcoin wallet script and BRC-20 ticker.
     */
    function getBrc20BalanceOf(
        string calldata ticker,
        string calldata address_pkscript
    ) external view returns (uint256 balance);

    /**
     * @dev Get Bitcoin transaction details using tx ids.
     */
    function getBitcoinTxDetails(
        string calldata txid
    )
        external
        view
        returns (
            uint256 block_height,
            string memory vin_txid,
            uint256 vin_vout,
            string memory vin_scriptPubKey_hex,
            uint256 vin_value,
            string memory vout_scriptPubKey_hex,
            uint256 vout_value
        );
}

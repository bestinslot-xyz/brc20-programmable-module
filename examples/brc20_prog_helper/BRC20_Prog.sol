// SPDX-License-Identifier: MIT

pragma solidity ^0.8.19;

import {IBRC20_Prog} from "./IBRC20_Prog.sol";

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
            string[] memory vin_txids,
            uint256[] memory vin_vouts,
            string[] memory vin_scriptPubKey_hexes,
            uint256[] memory vin_values,
            string[] memory vout_scriptPubKey_hexes,
            uint256[] memory vout_values
        );
}

contract BRC20_Prog is IBRC20_Prog {
    address private _btc_transaction_address =
        0x00000000000000000000000000000000000000fd;
    address private _bip322_address =
        0x00000000000000000000000000000000000000fe;
    address private _brc20_controller_address =
        0x00000000000000000000000000000000000000ff;

    /**
     * @dev Verifies BIP322 signature, given address, message and the signature.
     */
    function verifyBIP322Signature(
        string calldata addr,
        string calldata message_base64,
        string calldata signature_base64
    ) external override returns (bool verified) {
        return
            IBIP322_Verifier(_bip322_address).verify(
                addr,
                message_base64,
                signature_base64
            );
    }

    /**
     * @dev Get non-module BRC-20 balance of a given Bitcoin wallet script and BRC-20 ticker.
     */
    function getBrc20BalanceOf(
        string calldata ticker,
        string calldata address_pkscript
    ) external view returns (uint256 balance) {
        return
            IBRC20_Balance(_brc20_controller_address).balanceOf(
                ticker,
                address_pkscript
            );
    }

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
            string[] memory vin_txids,
            uint256[] memory vin_vouts,
            string[] memory vin_scriptPubKey_hexes,
            uint256[] memory vin_values,
            string[] memory vout_scriptPubKey_hexes,
            uint256[] memory vout_values
        )
    {
        return IBTC_Transaction(_btc_transaction_address).getTxDetails(txid);
    }
}

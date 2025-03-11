// SPDX-License-Identifier: MIT

pragma solidity ^0.8.19;

import {IBRC20_Prog, IBIP322_Verifier, IBTC_Transaction, IBRC20_Balance} from "./IBRC20_Prog.sol";

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
     *
     * Returns (string, uint256, uint256) where the first element is the txid, the second element is the block number and the third element is the block timestamp.
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
        )
    {
        return IBTC_Transaction(_btc_transaction_address).getTxDetails(txid);
    }
}

// SPDX-License-Identifier: MIT

pragma solidity ^0.8.19;

import {IBRC20_Prog, IBIP322_Verifier, IBTC_Transaction, IBRC20_Balance} from "./IBRC20_Prog.sol";

contract BRC20_Prog is IBRC20_Prog {
    address private _btc_transaction_address = 0x00000000000000000000000000000000000000fd;
    address private _bip322_address = 0x00000000000000000000000000000000000000fe;
    address private _brc20_controller_address = 0x00000000000000000000000000000000000000ff;

    /**
     * @dev Verifies BIP322 signature, given address, message and the signature.
     */
    function verify(
        address addr,
        string calldata message_base64,
        string calldata signature_base64
    ) external override returns (bool) {
        return IBIP322_Verifier(_bip322_address).verify(addr, message_base64, signature_base64);
    }

    /**
     * @dev Get non-module BRC-20 balance of a given Bitcoin wallet script and BRC-20 ticker.
     */
    function balanceOf(string calldata ticker, address account) external view returns (uint256) {
        return IBRC20_Balance(_brc20_controller_address).balanceOf(ticker, account);
    }

    /**
     * @dev Get Bitcoin transaction details using tx ids.
     */
    function getTxDetails(string calldata txid) external view returns (string memory, uint256, uint256) {
        return IBTC_Transaction(_btc_transaction_address).getTxDetails(txid);
    }
}

// SPDX-License-Identifier: MIT

pragma solidity ^0.8.19;

/**
 * @dev BRC-20 Prog helper functions.
 */
contract BRC20_Prog {
    address private _btc_locked_pkscript_address =
        0x00000000000000000000000000000000000000fb;
    address private _btc_last_sat_loc_address =
        0x00000000000000000000000000000000000000fc;
    address private _btc_tx_details_address =
        0x00000000000000000000000000000000000000fd;
    address private _bip322_verify_address =
        0x00000000000000000000000000000000000000fe;
    address private _brc20_controller_address =
        0x00000000000000000000000000000000000000ff;

    constructor() {}

    /**
     * @dev Verifies BIP322 signature, given address, message and the signature.
     */
    function verifyBIP322Signature(
        string calldata addr,
        string calldata message_base64,
        string calldata signature_base64
    ) external view returns (bool verified) {
        (bool success, bytes memory data) = _bip322_verify_address.staticcall(
            abi.encodeWithSignature(
                "verify(string,string,string)",
                addr,
                message_base64,
                signature_base64
            )
        );
        require(success, "Failed to verify BIP322 signature");
        return abi.decode(data, (bool));
    }

    /**
     * @dev Get non-module BRC-20 balance of a given Bitcoin wallet script and BRC-20 ticker.
     */
    function getBrc20BalanceOf(
        string calldata ticker,
        string calldata address_pkscript
    ) external view returns (uint256 balance) {
        (bool success, bytes memory data) = _brc20_controller_address
            .staticcall(
                abi.encodeWithSignature(
                    "balanceOf(string,string)",
                    ticker,
                    address_pkscript
                )
            );
        require(success, "Failed to get BRC20 balance");
        return abi.decode(data, (uint256));
    }

    /**
     * @dev Get Bitcoin transaction details using tx ids.
     */
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
        )
    {
        (bool success, bytes memory data) = _btc_tx_details_address.staticcall(
            abi.encodeWithSignature("getTxDetails(string)", txid)
        );
        require(success, "Failed to get transaction details");
        return abi.decode(
                data,
                (
                    uint256,
                    string[],
                    uint256[],
                    string[],
                    uint256[],
                    string[],
                    uint256[]
                )
            );
    }

    /**
     * @dev Get last satoshi location of a given sat location in a transaction.
     */
    function getLastSatLocation(
        string calldata txid,
        uint256 vout,
        uint256 sat
    )
        external
        view
        returns (
            string memory last_txid,
            uint256 last_vout,
            uint256 last_sat,
            string memory old_pkscript,
            string memory new_pkscript
        )
    {
        (bool success, bytes memory data) = _btc_last_sat_loc_address
            .staticcall(
                abi.encodeWithSignature(
                    "getLastSatLocation(string,uint256,uint256)",
                    txid,
                    vout,
                    sat
                )
            );
        require(success, "Failed to get last satoshi location");
        return abi.decode(data, (string, uint256, uint256, string, string));
    }

    /**
     * @dev Get locked pkscript of a given Bitcoin wallet script.
     */
    function getLockedPkscript(
        string calldata address_pkscript,
        uint256 lock_block_count
    ) external view returns (string memory locked_pkscript) {
        (bool success, bytes memory data) = _btc_locked_pkscript_address
            .staticcall(
                abi.encodeWithSignature(
                    "getLockedPkscript(string,uint256)",
                    address_pkscript,
                    lock_block_count
                )
            );
        require(success, "Failed to get locked pkscript");
        return abi.decode(data, (string));
    }

    /**
     * @dev Sha256 hash of a given message. For testing precompiles.
     */
    function getSha256(string calldata message) external view returns (bytes32) {
        (bool success, bytes memory data) = address(0x02).staticcall(abi.encodePacked(message));
        require(success, "Failed to get SHA256");
        return abi.decode(data, (bytes32));
    }

    /**
     * @dev Get random number.
     */
    function getRandomNumber() external pure returns (uint256) {
        return 42;
    }
}

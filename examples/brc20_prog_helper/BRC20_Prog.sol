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
    function verify(
        bytes calldata pkscript,
        bytes calldata message,
        bytes calldata signature
    ) external view returns (bool verified) {
        (bool success, bytes memory data) = _bip322_verify_address.staticcall(
            abi.encodeWithSignature(
                "verify(bytes,bytes,bytes)",
                pkscript,
                message,
                signature
            )
        );
        require(success, "Failed to verify BIP322 signature");
        return abi.decode(data, (bool));
    }

    /**
     * @dev Get non-module BRC-20 balance of a given Bitcoin wallet script and BRC-20 ticker.
     */
    function balanceOf(
        bytes calldata ticker,
        bytes calldata pkscript
    ) external view returns (uint256 balance) {
        (bool success, bytes memory data) = _brc20_controller_address
            .staticcall(
                abi.encodeWithSignature(
                    "balanceOf(bytes,bytes)",
                    ticker,
                    pkscript
                )
            );
        require(success, "Failed to get BRC20 balance");
        return abi.decode(data, (uint256));
    }

    /**
     * @dev Get Bitcoin transaction details using tx ids.
     */
    function getTxDetails(
        bytes32 txid
    )
        external
        view
        returns (
            uint256 block_height,
            bytes32[] memory vin_txids,
            uint256[] memory vin_vouts,
            bytes[] memory vin_scriptPubKeys,
            uint256[] memory vin_values,
            bytes[] memory vout_scriptPubKeys,
            uint256[] memory vout_values
        )
    {
        (bool success, bytes memory data) = _btc_tx_details_address.staticcall(
            abi.encodeWithSignature("getTxDetails(bytes32)", txid)
        );
        require(success, "Failed to get transaction details");
        return abi.decode(
                data,
                (
                    uint256,
                    bytes32[],
                    uint256[],
                    bytes[],
                    uint256[],
                    bytes[],
                    uint256[]
                )
            );
    }

    /**
     * @dev Get last satoshi location of a given sat location in a transaction.
     */
    function getLastSatLocation(
        bytes32 txid,
        uint256 vout,
        uint256 sat
    )
        external
        view
        returns (
            bytes32 last_txid,
            uint256 last_vout,
            uint256 last_sat,
            bytes memory old_pkscript,
            bytes memory new_pkscript
        )
    {
        (bool success, bytes memory data) = _btc_last_sat_loc_address
            .staticcall(
                abi.encodeWithSignature(
                    "getLastSatLocation(bytes32,uint256,uint256)",
                    txid,
                    vout,
                    sat
                )
            );
        require(success, "Failed to get last satoshi location");
        return abi.decode(data, (bytes32, uint256, uint256, bytes, bytes));
    }

    /**
     * @dev Get locked pkscript of a given Bitcoin pkscript.
     */
    function getLockedPkscript(
        bytes calldata pkscript,
        uint256 lock_block_count
    ) external view returns (bytes memory locked_pkscript) {
        (bool success, bytes memory data) = _btc_locked_pkscript_address
            .staticcall(
                abi.encodeWithSignature(
                    "getLockedPkscript(bytes,uint256)",
                    pkscript,
                    lock_block_count
                )
            );
        require(success, "Failed to get locked pkscript");
        return abi.decode(data, (bytes));
    }

    /**
     * @dev Sha256 hash of a given message. For testing precompiles.
     */
    function getSha256(bytes calldata message) external view returns (bytes32) {
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

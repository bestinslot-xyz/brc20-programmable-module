# Precompiles

Execution engine has precompiled contracts deployed at given addresses to make it easier to work with bitcoin transactions.

Precompiled contracts require environment variables to work properly, otherwise they fail. See [Configuration](configuration.md) for the variables, and [Differences from Ethereum](differences-from-ethereum.md) for how precompile calls differ from Ethereum.

| Precompile         | Address                                    |
| ------------------ | ------------------------------------------ |
| Reserved.          | 0x00000000000000000000000000000000000000ff |
| BIP322_Verifier    | 0x00000000000000000000000000000000000000fe |
| BTC_Transaction    | 0x00000000000000000000000000000000000000fd |
| BTC_LastSatLoc     | 0x00000000000000000000000000000000000000fc |
| BTC_LockedPkScript | 0x00000000000000000000000000000000000000fb |
| Current_Txid       | 0x00000000000000000000000000000000000000fa |

## BIP322 Verifier Contract

`BIP322_Verifier` contract can be used to verify a BIP322 signature. This precompile uses the [rust-bitcoin/bip322](https://github.com/rust-bitcoin/bip322) library.

**Contract interface**:

```solidity
/**
 * @dev BIP322 verification method
 */
interface IBIP322_Verifier {
    function verify(
        bytes calldata pkscript,
        bytes calldata message,
        bytes calldata signature
    ) external returns (bool success);
}
```

> [!WARNING]
> Currently [rust-bitcoin/bip322](https://github.com/rust-bitcoin/bip322) and this precompile only supports `P2TR`, `P2WPKH` and `P2SH-P2WPKH` single-sig addresses.

> [!WARNING]
> This precompile supports up to 32 KB input size for combined `pkscript`, `message` and `signature` parameters. This is to avoid excessive resource usage and potential denial of service attacks.

## Bitcoin Contracts

BRC2.0 has a set of precompiles that make it easier to work with bitcoin transactions within a smart contract. These can be used to retrieve transaction details, track satoshis across transactions and calculate locked pkscripts. These allow BRC2.0 smart contracts to be aware of the transactions, ordinals and ordinal lockers that happen outside the execution engine.

> [!WARNING]
> `BTC_Transaction` and `BTC_LastSatLoc` precompiles use Bitcoin JSON-RPC calls to calculate results, so an RPC server needs to be specified in the environment variables.
> 
> Associated environment variables are `BITCOIN_RPC_URL`, `BITCOIN_RPC_USER`, `BITCOIN_RPC_PASSWORD` and `BITCOIN_RPC_NETWORK`. See [env.sample](../env.sample) for a sample environment for Signet.

### Get Current Txid

`Current_Txid` contract can be used to retrieve the current Bitcoin transaction id being executed. This can be useful to enable contracts to be aware of the transaction they are being executed in.

**Contract interface**:

```solidity
/**
 * @dev Get current Bitcoin transaction id being executed.
 */
interface IBTC_CurrentTxid {
    function getTxId() external view returns (bytes32 txid);
}
```

> [!WARNING]
> This precompile is only usable after block 275_000 on Signet, and 923_369 on Mainnet. This coincides with the Prague upgrade activation on Signet and Mainnet.
> Proposal: https://github.com/bestinslot-xyz/brc20-proposals/blob/main/004-prog-tx-id-precompile/index.md

### Transaction details

`BTC_Transaction` contract can be used to retrieve details for a bitcoin transaction. Returns block height, and `vin`, `vout` txids, scriptPubKeys and values as arrays.

**Contract interface**:

```solidity
/**
 * Get Bitcoin transaction details using tx ids.
 */
interface IBTC_Transaction {
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
        );
}
```

### Last sat location

`BTC_LastSatLoc` contract can be used to retrieve previous location of a satoshi at given `txid`, `vout` and `sat` number using the rules detailed at [ordinals/ord/blob/master/bip.mediawiki](https://github.com/ordinals/ord/blob/master/bip.mediawiki).

**Contract interface**:

```solidity
/**
 * @dev Get last satoshi location of a given sat location in a transaction.
 */
interface IBTC_LastSatLoc {
    function getLastSatLocation(
        bytes32 txid,
        uint256 vout,
        uint256 sat
    ) external view returns (
        bytes32 last_txid,
        uint256 last_vout,
        uint256 last_sat,
        bytes memory old_pkscript,
        bytes memory new_pkscript
    );
}
```

### Get locked pkscript

`BTC_LockedPkScript` contract can be used to calculate lock pkscripts for given pkscript and block count.

**Contract interface**:

```solidity
/**
 * @dev Get locked pkscript of a given Bitcoin wallet script.
 */
interface IBTC_LockedPkscript {
    function getLockedPkscript(
        bytes calldata pkscript,
        uint256 lock_block_count
    ) external view returns (bytes memory locked_pkscript);
}
```

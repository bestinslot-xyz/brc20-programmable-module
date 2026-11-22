use bitcoin::Network;
use revm::primitives::hardfork::SpecId;

use crate::engine::precompiles::get_bitcoin_network;

const PRAGUE_ACTIVATION_HEIGHT_MAINNET: u64 = 923_369;
const PRAGUE_ACTIVATION_HEIGHT_SIGNET: u64 = 275_000;

pub fn get_evm_spec(block_number: u64) -> SpecId {
    let network = get_bitcoin_network();
    match network {
        Network::Bitcoin => {
            if block_number >= PRAGUE_ACTIVATION_HEIGHT_MAINNET {
                SpecId::PRAGUE
            } else {
                SpecId::CANCUN
            }
        }
        Network::Signet => {
            if block_number >= PRAGUE_ACTIVATION_HEIGHT_SIGNET {
                SpecId::PRAGUE
            } else {
                SpecId::CANCUN
            }
        }
        Network::Regtest => SpecId::PRAGUE,
        _ => SpecId::PRAGUE,
    }
}

const RLP_HASH_ACTIVATION_HEIGHT_MAINNET: u64 = 929_000; // 22 Dec 2025, reindexing not required
const RLP_HASH_ACTIVATION_HEIGHT_SIGNET: u64 = 0; // Always use RLP hash on Signet, reindexing required

pub fn use_rlp_hash_for_tx_hash(block_number: u64) -> bool {
    let network = get_bitcoin_network();
    match network {
        Network::Bitcoin => block_number >= RLP_HASH_ACTIVATION_HEIGHT_MAINNET,
        Network::Signet => block_number >= RLP_HASH_ACTIVATION_HEIGHT_SIGNET,
        _ => true,
    }
}

// bip322 v0.0.11 accepts short DER signatures, P2PKH and multisig proofs that v0.0.10 rejects.
// Blocks below these heights are verified with the v0.0.10 rules, reindexing not required.
const BIP322_V0_0_11_ACTIVATION_HEIGHT_MAINNET: u64 = 960_847; // 3 Aug 2026
const BIP322_V0_0_11_ACTIVATION_HEIGHT_SIGNET: u64 = 328_057; // 26 Oct 2026

pub fn use_bip322_v0_0_11(block_number: u64) -> bool {
    bip322_v0_0_11_active(get_bitcoin_network(), block_number)
}

fn bip322_v0_0_11_active(network: Network, block_number: u64) -> bool {
    match network {
        Network::Bitcoin => block_number >= BIP322_V0_0_11_ACTIVATION_HEIGHT_MAINNET,
        Network::Signet => block_number >= BIP322_V0_0_11_ACTIVATION_HEIGHT_SIGNET,
        _ => true,
    }
}

// BTC_TxWitness precompile at 0xf9. The mainnet and signet heights are placeholders that keep it
// inactive until the maintainers choose activation heights.
const BTC_TX_WITNESS_ACTIVATION_HEIGHT_MAINNET: u64 = u64::MAX;
const BTC_TX_WITNESS_ACTIVATION_HEIGHT_SIGNET: u64 = u64::MAX;
const BTC_TX_WITNESS_ACTIVATION_HEIGHT_REGTEST: u64 = 0;

pub fn btc_tx_witness_active(block_number: u64) -> bool {
    btc_tx_witness_active_on(get_bitcoin_network(), block_number)
}

fn btc_tx_witness_active_on(network: Network, block_number: u64) -> bool {
    match network {
        Network::Bitcoin => block_number >= BTC_TX_WITNESS_ACTIVATION_HEIGHT_MAINNET,
        Network::Signet => block_number >= BTC_TX_WITNESS_ACTIVATION_HEIGHT_SIGNET,
        Network::Regtest => block_number >= BTC_TX_WITNESS_ACTIVATION_HEIGHT_REGTEST,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bip322_v0_0_11_activation() {
        assert!(!bip322_v0_0_11_active(Network::Bitcoin, 913_183));
        assert!(!bip322_v0_0_11_active(Network::Bitcoin, 960_846));
        assert!(bip322_v0_0_11_active(Network::Bitcoin, 960_847));
        assert!(!bip322_v0_0_11_active(Network::Signet, 267_703));
        assert!(!bip322_v0_0_11_active(Network::Signet, 328_056));
        assert!(bip322_v0_0_11_active(Network::Signet, 328_057));
        assert!(bip322_v0_0_11_active(Network::Regtest, 0));
    }

    #[test]
    fn test_btc_tx_witness_activation() {
        assert!(btc_tx_witness_active_on(Network::Regtest, 0));
        assert!(!btc_tx_witness_active_on(Network::Bitcoin, 2_000_000));
        assert!(!btc_tx_witness_active_on(Network::Signet, 2_000_000));
        assert!(!btc_tx_witness_active_on(Network::Testnet4, 2_000_000));
    }
}

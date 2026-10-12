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
// Mainnet blocks below this height are verified with the v0.0.10 rules, reindexing not required.
const BIP322_V0_0_11_ACTIVATION_HEIGHT_MAINNET: u64 = 960_847; // 3 Aug 2026

pub fn use_bip322_v0_0_11(block_number: u64) -> bool {
    bip322_v0_0_11_active(get_bitcoin_network(), block_number)
}

fn bip322_v0_0_11_active(network: Network, block_number: u64) -> bool {
    match network {
        Network::Bitcoin => block_number >= BIP322_V0_0_11_ACTIVATION_HEIGHT_MAINNET,
        _ => true,
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
        assert!(bip322_v0_0_11_active(Network::Signet, 0));
        assert!(bip322_v0_0_11_active(Network::Regtest, 0));
    }
}

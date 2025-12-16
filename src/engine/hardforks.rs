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

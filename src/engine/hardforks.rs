use bitcoin::Network;
use revm::primitives::hardfork::SpecId;

use crate::engine::precompiles::get_bitcoin_network;

const PRAGUE_ACTIVATION_HEIGHT_MAINNET: u64 = u64::MAX;
const PRAGUE_ACTIVATION_HEIGHT_SIGNET: u64 = u64::MAX;

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

use bitcoin::Network;
use revm::primitives::hardfork::SpecId;

use crate::global::CONFIG;

const PRAGUE_ACTIVATION_HEIGHT_MAINNET: u64 = u64::MAX;
const PRAGUE_ACTIVATION_HEIGHT_TESTNETS: u64 = u64::MAX;

pub fn get_bitcoin_network() -> bitcoin::Network {
    use bitcoin::Network;
    match CONFIG.read().bitcoin_rpc_network.as_str() {
        "bitcoin" => Network::Bitcoin,
        "signet" => Network::Signet,
        "testnet" => Network::Testnet,
        "regtest" => Network::Regtest,
        "testnet4" => Network::Testnet4,
        _ => Network::Regtest,
    }
}

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
            if block_number >= PRAGUE_ACTIVATION_HEIGHT_TESTNETS {
                SpecId::PRAGUE
            } else {
                SpecId::CANCUN
            }
        }
        Network::Regtest => SpecId::CANCUN,
        _ => SpecId::CANCUN,
    }
}

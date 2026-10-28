use std::collections::HashMap;

use alloy::primitives::{Address, Bytes};
use revm::context::{Block, Cfg, ContextTr};
use revm::handler::PrecompileProvider;
use revm::interpreter::{CallInputs, Gas, InstructionResult, InterpreterResult};
use revm::precompile::{PrecompileSpecId, PrecompileStatus, Precompiles};
use revm::primitives::{AddressSet, B256, U256};

use crate::engine::precompiles::btc_utils::{BitcoinDataUnavailable, BitcoinLookup};
use crate::engine::precompiles::{
    bip322_verify_precompile, btc_tx_details_precompile, get_locked_pkscript_precompile,
    get_op_return_tx_id_precompile, last_sat_location_precompile,
};
use crate::types::PrecompileData;

lazy_static::lazy_static! {
    static ref BIP322_PRECOMPILE_ADDRESS: Address = "0x00000000000000000000000000000000000000fe".parse().expect("Invalid BIP322 precompile address");
    static ref BTC_TX_DETAILS_PRECOMPILE_ADDRESS: Address = "0x00000000000000000000000000000000000000fd".parse().expect("Invalid BTC transaction details precompile address");
    static ref LAST_SAT_LOCATION_PRECOMPILE_ADDRESS: Address = "0x00000000000000000000000000000000000000fc".parse().expect("Invalid last sat location precompile address");
    static ref GET_LOCKED_PK_SCRIPT_PRECOMPILE_ADDRESS: Address = "0x00000000000000000000000000000000000000fb".parse().expect("Invalid get locked pk script precompile address");
    static ref GET_OP_RETURN_TX_ID_PRECOMPILE_ADDRESS: Address = "0x00000000000000000000000000000000000000fa".parse().expect("Invalid get op return tx id precompile address");
}

pub struct PrecompileCall {
    pub bytes: Bytes,
    pub gas_limit: u64,
    pub block_height: U256,
    pub current_op_return_tx_id: B256,
    pub btc_tx_hexes_data: HashMap<B256, Bytes>,
    pub bitcoin_lookup: BitcoinLookup,
}

/// A BRC2.0 precompile. `Err` aborts the whole call, which happens only when a Bitcoin lookup
/// fails during a public call.
pub type CustomPrecompile =
    fn(&PrecompileCall) -> Result<InterpreterResult, BitcoinDataUnavailable>;

pub struct BRC20Precompiles {
    pub eth_precompiles: &'static Precompiles,
    pub custom_precompiles: HashMap<Address, CustomPrecompile>,
    pub all_addresses: AddressSet,
    pub op_return_tx_id: B256,
    pub btc_tx_hexes_data: HashMap<B256, Bytes>,
    pub bitcoin_lookup: BitcoinLookup,
    /// Set when a Bitcoin lookup aborted the call, so the caller can report
    /// [`BitcoinDataUnavailable`] instead of the string error revm passes up.
    pub bitcoin_unavailable: bool,
}

impl BRC20Precompiles {
    pub fn new(
        precompile_spec: PrecompileSpecId,
        op_return_tx_id: B256,
        precompile_data: &Option<PrecompileData>,
        bitcoin_lookup: BitcoinLookup,
    ) -> Self {
        let eth_precompiles = Precompiles::new(precompile_spec);
        let mut all_addresses = eth_precompiles.addresses_set().clone();
        all_addresses.insert(*BIP322_PRECOMPILE_ADDRESS);
        all_addresses.insert(*BTC_TX_DETAILS_PRECOMPILE_ADDRESS);
        all_addresses.insert(*LAST_SAT_LOCATION_PRECOMPILE_ADDRESS);
        all_addresses.insert(*GET_LOCKED_PK_SCRIPT_PRECOMPILE_ADDRESS);

        let mut custom_precompiles: HashMap<Address, CustomPrecompile> = HashMap::new();
        custom_precompiles.insert(*BIP322_PRECOMPILE_ADDRESS, |call| {
            Ok(bip322_verify_precompile(call))
        });
        custom_precompiles.insert(
            *BTC_TX_DETAILS_PRECOMPILE_ADDRESS,
            btc_tx_details_precompile,
        );
        custom_precompiles.insert(
            *LAST_SAT_LOCATION_PRECOMPILE_ADDRESS,
            last_sat_location_precompile,
        );
        custom_precompiles.insert(*GET_LOCKED_PK_SCRIPT_PRECOMPILE_ADDRESS, |call| {
            Ok(get_locked_pkscript_precompile(call))
        });
        if precompile_spec >= PrecompileSpecId::PRAGUE {
            custom_precompiles.insert(*GET_OP_RETURN_TX_ID_PRECOMPILE_ADDRESS, |call| {
                Ok(get_op_return_tx_id_precompile(call))
            });
        }

        let btc_tx_hexes_data = if let Some(data) = precompile_data {
            data.bitcoin_tx_hexes
                .iter()
                .map(|(k, v)| (k.bytes, v.value().unwrap_or_default()))
                .collect()
        } else {
            HashMap::new()
        };

        Self {
            eth_precompiles,
            all_addresses,
            custom_precompiles,
            op_return_tx_id,
            btc_tx_hexes_data,
            bitcoin_lookup,
            bitcoin_unavailable: false,
        }
    }
}

impl<CTX: ContextTr> PrecompileProvider<CTX> for BRC20Precompiles {
    type Output = InterpreterResult;

    fn set_spec(&mut self, _: <CTX::Cfg as Cfg>::Spec) -> bool {
        // No-op
        true
    }

    fn run(&mut self, ctx: &mut CTX, inputs: &CallInputs) -> Result<Option<Self::Output>, String> {
        if let Some(eth_precompile) = self.eth_precompiles.get(&inputs.target_address) {
            let output = eth_precompile
                .execute(&inputs.input.bytes(ctx), inputs.gas_limit, inputs.reservoir)
                .map_err(|e| e.to_string())?;
            // Any unsuccessful Ethereum precompile call, including out of gas, aborts the whole
            // transaction. This is consensus behaviour, so it must not become a failed subcall.
            match &output.status {
                PrecompileStatus::Success => {}
                PrecompileStatus::Halt(reason) => return Err(reason.to_string()),
                PrecompileStatus::Revert => return Err("precompile reverted".to_string()),
            }
            let mut gas = Gas::new(inputs.gas_limit);
            if !gas.record_regular_cost(output.gas_used) {
                return Ok(Some(InterpreterResult::new(
                    InstructionResult::OutOfGas,
                    Bytes::new(),
                    gas,
                )));
            } else {
                return Ok(Some(InterpreterResult::new(
                    InstructionResult::Stop,
                    output.bytes,
                    gas,
                )));
            }
        } else if let Some(custom_precompile) = self.custom_precompiles.get(&inputs.target_address)
        {
            // A Bitcoin lookup that failed during a public call aborts the whole call. As a failed
            // subcall, a contract could catch it and answer with a result block execution never
            // produces.
            let output = custom_precompile(&PrecompileCall {
                bytes: inputs.input.bytes(ctx),
                gas_limit: inputs.gas_limit,
                block_height: ctx.block().number(),
                current_op_return_tx_id: self.op_return_tx_id,
                btc_tx_hexes_data: self.btc_tx_hexes_data.clone(),
                bitcoin_lookup: self.bitcoin_lookup,
            });
            return match output {
                Ok(output) => Ok(Some(output)),
                Err(error) => {
                    self.bitcoin_unavailable = true;
                    Err(error.to_string())
                }
            };
        } else {
            return Ok(None);
        }
    }

    fn warm_addresses(&self) -> &AddressSet {
        &self.all_addresses
    }

    fn contains(&self, address: &Address) -> bool {
        self.all_addresses.contains(address)
    }
}

/// Records a `gas` cost and fails the instruction if it would exceed the available gas.
pub fn use_gas(interpreter_result: &mut InterpreterResult, gas: u64) -> bool {
    if !interpreter_result.gas.record_regular_cost(gas) {
        interpreter_result.result = revm::interpreter::InstructionResult::OutOfGas;
        return false;
    }
    true
}

/// Fails the instruction with a `PrecompileError` result.
pub fn precompile_error(
    mut interpreter_result: InterpreterResult,
    error: &'static str,
) -> InterpreterResult {
    interpreter_result.result = revm::interpreter::InstructionResult::PrecompileError;
    interpreter_result.output = Bytes::from_static(&error.as_bytes());
    interpreter_result
}

// Returns output for the instruction
pub fn precompile_output(
    mut interpreter_result: InterpreterResult,
    output: Vec<u8>,
) -> InterpreterResult {
    interpreter_result.result = revm::interpreter::InstructionResult::Stop;
    interpreter_result.output = output.into();
    interpreter_result
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_prague_spec_has_bls_precompiles() {
        // A sanity test to ensure that the Prague spec includes the BLS precompiles.
        let precompiles = BRC20Precompiles::new(
            PrecompileSpecId::PRAGUE,
            [0u8; 32].into(),
            &None,
            BitcoinLookup::BlockExecution,
        );
        assert!(precompiles
            .all_addresses
            .contains(&Address::from_str("0x000000000000000000000000000000000000000b").unwrap()));
        assert!(precompiles
            .all_addresses
            .contains(&Address::from_str("0x000000000000000000000000000000000000000c").unwrap()));
        assert!(precompiles
            .all_addresses
            .contains(&Address::from_str("0x000000000000000000000000000000000000000d").unwrap()));
        assert!(precompiles
            .all_addresses
            .contains(&Address::from_str("0x000000000000000000000000000000000000000e").unwrap()));
        assert!(precompiles
            .all_addresses
            .contains(&Address::from_str("0x000000000000000000000000000000000000000f").unwrap()));
        assert!(precompiles
            .all_addresses
            .contains(&Address::from_str("0x0000000000000000000000000000000000000010").unwrap()));
        assert!(precompiles
            .all_addresses
            .contains(&Address::from_str("0x0000000000000000000000000000000000000011").unwrap()));

        // Make sure it does not contain precompiles that are not part of the Prague spec
        assert!(!precompiles
            .all_addresses
            .contains(&Address::from_str("0x0000000000000000000000000000000000000012").unwrap()));
    }
}

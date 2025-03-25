use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use revm::context::{Cfg, ContextTr};
use revm::handler::PrecompileProvider;
use revm::interpreter::{Gas, InstructionResult, InterpreterResult};
use revm::precompile::Precompiles;
use revm::primitives::{Address, Bytes};

use crate::evm::precompiles::{
    bip322_verify_precompile, brc20_balance_precompile, btc_tx_details_precompile,
    get_locked_pkscript_precompile, last_sat_location_precompile,
};

lazy_static::lazy_static! {
    static ref BRC20_BALANCE_PRECOMPILE_ADDRESS: Address = Address::from_str("0x00000000000000000000000000000000000000ff").unwrap();
    static ref BIP322_PRECOMPILE_ADDRESS: Address = Address::from_str("0x00000000000000000000000000000000000000fe").unwrap();
    static ref BTC_TX_DETAILS_PRECOMPILE_ADDRESS: Address = Address::from_str("0x00000000000000000000000000000000000000fd").unwrap();
    static ref LAST_SAT_LOCATION_PRECOMPILE_ADDRESS: Address = Address::from_str("0x00000000000000000000000000000000000000fc").unwrap();
    static ref GET_LOCKED_PK_SCRIPT_PRECOMPILE_ADDRESS: Address = Address::from_str("0x00000000000000000000000000000000000000fb").unwrap();
}

pub struct BRC20Precompiles {
    pub eth_precompiles: &'static Precompiles,
    pub custom_precompiles: HashMap<Address, fn(&Bytes, u64) -> InterpreterResult>,
    pub all_addresses: HashSet<Address>,
}

impl Default for BRC20Precompiles {
    fn default() -> Self {
        let eth_precompiles = Precompiles::cancun();
        let mut all_addresses = eth_precompiles
            .addresses()
            .map(|x| x.clone())
            .collect::<HashSet<Address>>();
        all_addresses.insert(*BRC20_BALANCE_PRECOMPILE_ADDRESS);
        all_addresses.insert(*BIP322_PRECOMPILE_ADDRESS);
        all_addresses.insert(*BTC_TX_DETAILS_PRECOMPILE_ADDRESS);
        all_addresses.insert(*LAST_SAT_LOCATION_PRECOMPILE_ADDRESS);
        all_addresses.insert(*GET_LOCKED_PK_SCRIPT_PRECOMPILE_ADDRESS);

        let mut custom_precompiles: HashMap<Address, fn(&Bytes, u64) -> InterpreterResult> =
            HashMap::new();
        custom_precompiles.insert(*BRC20_BALANCE_PRECOMPILE_ADDRESS, brc20_balance_precompile);
        custom_precompiles.insert(*BIP322_PRECOMPILE_ADDRESS, bip322_verify_precompile);
        custom_precompiles.insert(*BTC_TX_DETAILS_PRECOMPILE_ADDRESS, btc_tx_details_precompile);
        custom_precompiles.insert(*LAST_SAT_LOCATION_PRECOMPILE_ADDRESS, last_sat_location_precompile);
        custom_precompiles.insert(
            *GET_LOCKED_PK_SCRIPT_PRECOMPILE_ADDRESS,
            get_locked_pkscript_precompile,
        );

        Self {
            eth_precompiles,
            all_addresses,
            custom_precompiles,
        }
    }
}

impl<CTX: ContextTr> PrecompileProvider<CTX> for BRC20Precompiles {
    type Output = InterpreterResult;

    fn set_spec(&mut self, _: <CTX::Cfg as Cfg>::Spec) {}

    fn run(
        &mut self,
        _: &mut CTX,
        address: &Address,
        bytes: &Bytes,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, String> {
        #[cfg(debug_assertions)]
        if self.all_addresses.contains(address) {
            println!("BRC20Precompiles handling: {:?}", address);
        }

        let result;
        if self.eth_precompiles.contains(address) {
            let cancun_result = self.eth_precompiles.get(address).unwrap()(bytes, gas_limit);
            match cancun_result {
                Ok(output) => {
                    let mut gas = Gas::new(gas_limit);
                    if !gas.record_cost(output.gas_used) {
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
                }
                Err(e) => return Err(e.to_string()),
            }
        } else if self.custom_precompiles.contains_key(address) {
            let function = self.custom_precompiles.get(address).unwrap();
            result = function(bytes, gas_limit);
        } else {
            return Ok(None);
        }
        Ok(Some(result))
    }

    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        println!("BRC20Precompiles warm_addresses");
        Box::new(self.all_addresses.iter().cloned())
    }

    fn contains(&self, address: &Address) -> bool {
        self.all_addresses.contains(address)
    }
}

/// Records a `gas` cost and fails the instruction if it would exceed the available gas.
pub fn use_gas(interpreter_result: &mut InterpreterResult, gas: u64) -> bool {
    if !interpreter_result.gas.record_cost(gas) {
        interpreter_result.result = revm::interpreter::InstructionResult::OutOfGas;
        return false;
    }
    true
}

/// Fails the instruction with a `PrecompileError` result.
pub fn precompile_error(mut interpreter_result: InterpreterResult) -> InterpreterResult {
    interpreter_result.result = revm::interpreter::InstructionResult::PrecompileError;
    interpreter_result
}

// Returns output for the instruction
pub fn precompile_output(
    mut interpreter_result: InterpreterResult,
    output: Vec<u8>,
) -> InterpreterResult {
    interpreter_result.result = revm::interpreter::InstructionResult::Stop;
    interpreter_result.output = Bytes::from(output);
    interpreter_result
}

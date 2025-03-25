use revm::interpreter::{Gas, InstructionResult, InterpreterResult};
use revm::primitives::Bytes;
use solabi::{selector, FunctionEncoder};

lazy_static::lazy_static! {
    static ref BRC20_CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::new();
    static ref BRC20_PROG_BALANCE_SERVER_URL: String = std::env::var("BRC20_PROG_BALANCE_SERVER_URL")
            .unwrap_or("http://localhost:18546".to_string());
}

const BALANCE_OF: FunctionEncoder<(String, String), (solabi::U256,)> =
    FunctionEncoder::new(selector!("balanceOf(string,string)"));

pub fn brc20_balance_precompile(bytes: &Bytes, gas_limit: u64) -> InterpreterResult {
    let gas_used = 100000;
    if gas_used > gas_limit {
        return InterpreterResult::new(
            InstructionResult::OutOfGas,
            Bytes::new(),
            Gas::new_spent(gas_used),
        );
    }

    let result = BALANCE_OF.decode_params(&bytes);

    if result.is_err() {
        return InterpreterResult::new(
            InstructionResult::PrecompileError,
            Bytes::new(),
            Gas::new_spent(gas_used),
        );
    }

    let (ticker, address) = result.unwrap();

    let response = BRC20_CLIENT
        .get(&*BRC20_PROG_BALANCE_SERVER_URL)
        .query(&[("ticker", ticker), ("address", address)])
        .send()
        .unwrap();

    let balance = response.text().unwrap();
    let balance = balance.parse::<u64>().unwrap_or(0);
    let balance = solabi::U256::from(balance);
    let bytes = BALANCE_OF.encode_returns(&(balance,));

    InterpreterResult::new(
        InstructionResult::Stop,
        Bytes::from(bytes),
        Gas::new_spent(gas_used),
    )
}

use revm::precompile::{ Precompile, PrecompileWithAddress, StandardPrecompileFn, PrecompileResult, Precompiles, PrecompileSpecId, Error as PrecompileError };
use revm::primitives::specification::SpecId;

pub fn load_precompiles() -> Precompiles {
  let mut precompiles = Precompiles::new(PrecompileSpecId::from_spec_id(SpecId::SHANGHAI)).clone(); // NOTE: also change get_evm while changing this
  precompiles.extend([
    PrecompileWithAddress::from((
      "0x00000000000000000000000000000000000000ff".parse().unwrap(), 
      Precompile::Standard(identity_run as StandardPrecompileFn)
    )),
    PrecompileWithAddress::from((
      "0x00000000000000000000000000000000000000fe".parse().unwrap(), 
      Precompile::Standard(identity_run as StandardPrecompileFn)
    ))
  ]);
  precompiles
}
pub fn identity_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
  println!("Running identity precompile");
  let gas_used = 100000;
  if gas_used > gas_limit {
    return Err(PrecompileError::OutOfGas);
  }
  Ok((gas_used, input.to_vec()))
}
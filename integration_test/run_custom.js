/* note run apt install build-essential before npm init to get faster bigint support */
// TODO: REMOVE deploy2, call2

var solc_fixed = require('solc_0_8_24');
const { exit } = require('node:process');
const { ethers } = require('ethers');
// const { Address } = require('ethereumjs-util')
var fs = require('fs');
var express = require('express');

const indexer_addr = '0x0000000000000000000000000000000000003Ca6'
const brc20_controller_addr = '0xc54dd4581af2dbf18e4d90840226756e9d2b3cdb'
// const block_hash_after_initialization = '0xf10af31fab134991a3241ef302446cc206260cfbecae5e9c361d3db5aa5be635'
const block_hash_after_initialization = '0x0000000000000000000000000000000000000000000000000000000000000000'
const module_activation_height = 779832
// const module_activation_height = 827473
const extra_arg_types = [
  "btc_address"
]
if (solc_fixed.version() != '0.8.24+commit.e11b9ed9.Emscripten.clang') {
  console.error("wrong solc version")
  exit(1)
}


/* const customPrecompiles = [
  {
    address: Address.fromString("0x00000000000000000000000000000000000000ff"),
    function: async () => {
      console.log("WE'RE IN CUSTOM PRECOMPILE!!")

      return {
        executionGasUsed: 0,
        returnValue: 420,
      }
    }
  },
  {
    address: Address.fromString("0x00000000000000000000000000000000000000fe"),
    function: async () => {
      console.log("WE'RE IN CUSTOM PRECOMPILE #2!!")

      return {
        executionGasUsed: 0,
        returnValue: 421,
      }
    }
  }
] */



// HELPER FUNCTIONS
async function provider_send(method, params) {
  let obj = {
    jsonrpc: "2.0",
    method: method,
    params: params,
    id: 1,
  }
  console.log("Calling " + method + " with params: " + JSON.stringify(params))
  let r = await fetch('http://localhost:18545', { method: 'POST', body: JSON.stringify(obj), headers: { 'Content-Type': 'application/json' } })
  let j = await r.json()
  if (j != null && j.hasOwnProperty("jsonrpc") && j["jsonrpc"] == "2.0" && j.hasOwnProperty("result")) {
    console.log(j["result"])
    return j["result"]
  }
  console.log(j)
  return j
}

async function get_info_of_block(block_number) {
  block_number = parseInt(block_number)
  block_number = "0x" + block_number.toString(16)
  let res = await provider_send(
    "eth_getBlockByNumber",
    { block: block_number },
  );

  return res
}
async function brc20_mine(n, timestamp) {
  await provider_send("brc20_mine", { block_cnt: parseInt(n), timestamp: parseInt(timestamp) });
}
/* async function eth_mine_with_txes(timestamp, hash, txes) {
  let responses = []
  for (const tx of txes) {
    let r = await provider_send("eth_addTxToBlock", { timestamp: parseInt(timestamp), hash: hash, tx_idx: responses.length, from: tx.from, to: tx.to, data: tx.data });
    responses.push(r);
  }

  await provider_send("eth_finaliseBlock", { timestamp: parseInt(timestamp), hash: hash, block_tx_cnt: responses.length });
  return responses
} */
async function brc20_mine_with_txes(timestamp, hash, txes) {
  return await provider_send("brc20_finaliseBlockWithTxes", { timestamp: parseInt(timestamp), hash: hash, txes: txes });
}



async function initialise_chain() {
  let init_st_tm = +(new Date())

  let current_block_height = parseInt(await provider_send("eth_blockNumber", {}))
  if (current_block_height == 0) {
    // console.log("deploying BRC20_Controller")
    // const deploy_brc20_controller_tx = await get_deploy_brc20_controller_tx();
    // let resp = await brc20_mine_with_txes(0, "0x0000000000000000000000000000000000000000000000000000000000000000", [deploy_brc20_controller_tx]);
    // let brc20_controller_deploy_receipt = resp[0];

    // if (brc20_controller_deploy_receipt.contractAddress != brc20_controller_addr) {
    //   console.error(`BRC20_Controller deployed to ${brc20_controller_deploy_receipt.contractAddress} instead of ${brc20_controller_addr}`);
    //   throw new Error("BRC20_Controller address does not match")
    // }

    // current_block_height = 1
  }

  let st_tm = +(new Date())
  for (let i = current_block_height; i < module_activation_height - 1;) {
    console.log(`initialising block ${i} time per block ${(+(new Date()) - st_tm) / (i - current_block_height + 1)}`)
    let block_count = Math.min(1000, module_activation_height - i - 1)
    await brc20_mine(block_count, 0);
    i += block_count
  }

  let res = await get_info_of_block(module_activation_height - 1)
  if (res["hash"] != block_hash_after_initialization) {
    console.error(`Block hash after initialization is ${res["hash"]} instead of ${block_hash_after_initialization}`);

    throw new Error("Block hash after initialization does not match")
  }

  console.log(`initialisation took ${(+(new Date()) - init_st_tm) / 1000} seconds`)
}

async function get_deploy_brc20_controller_tx() {
  let BRC20_Controller_sol = fs.readFileSync("./contracts/BRC20_Controller.sol", "utf8");
  let IBRC20_Controller_sol = fs.readFileSync("./contracts/IBRC20_Controller.sol", "utf8");
  let Context_sol = fs.readFileSync("./contracts/utils/Context.sol", "utf8");
  let draft_IBRC6093_sol = fs.readFileSync("./contracts/interfaces/draft-IBRC6093.sol", "utf8");
  let Ownable_sol = fs.readFileSync("./contracts/access/Ownable.sol", "utf8");
  let input = {
    language: 'Solidity',
    sources: {
      'BRC20_Controller.sol': { content: BRC20_Controller_sol },
      'IBRC20_Controller.sol': { content: IBRC20_Controller_sol },
      'utils/Context.sol': { content: Context_sol },
      'interfaces/draft-IBRC6093.sol': { content: draft_IBRC6093_sol },
      'access/Ownable.sol': { content: Ownable_sol },
    },
    settings: {
      evmVersion: "cancun",
      outputSelection: {
        '*': {
          '*': ['*']
        }
      }
    }
  };
  let compiled = JSON.parse(solc_fixed.compile(JSON.stringify(input)))
  let contract = compiled.contracts['BRC20_Controller.sol']["BRC20_Controller"]
  let bytecode = contract.evm.bytecode.object
  let abi = contract.abi

  let contract_factory = new ethers.ContractFactory(abi, bytecode, null)
  let deploy_tx = await contract_factory.getDeployTransaction([])

  return {
    from: indexer_addr,
    to: null,
    data: deploy_tx.data,
  }
}

async function mine_next_block(timestamp, hash, block_txes) {
  let txes = []
  for (const tx of block_txes) {
    let [, to_send, error,] = tx.resp
    if (error == null) {
      txes.push(to_send)
    }
  }
  let receipts = await brc20_mine_with_txes(timestamp, hash, txes);

  return check_block_receipts(block_txes, receipts)
}

function check_block_receipts(block_txes, receipts) {
  let responses = []
  let receipt_idx = 0;
  for (const tx of block_txes) {
    try {
      let [type, to_send, error, sender] = tx.resp
      if (error != null) {
        console.log("sender: " + sender + " type: " + type + " error: " + error + "to_send: " + JSON.stringify(to_send))
        responses.push({
          // txhash: null, // NOTE: txhash is in receipt
          sender: sender,
          receipt: null,
          error: "type: " + type + " error: " + error,
        })
        continue;
      }

      let receipt = receipts[receipt_idx];
      receipt_idx += 1;
      if (receipt["txResult"] != "Success") {
        throw { shortMessage: "transaction execution reverted", receipt: receipt } // TODO: check if this can fail any other way!!
      }

      // console.log("used gas: " + parseInt(receipt["gasUsed"]))

      responses.push({
        // txhash: receipt["hash"], // NOTE: txhash is in receipt
        sender: sender,
        receipt: receipt,
        error: null,
      })
    } catch (e) {
      if (e.shortMessage != 'transaction execution reverted') { // TODO: unknown error, repeat!!
        console.log(e)
        exit(1)
      }

      let [type, , , sender] = tx.resp

      console.log("failed, used gas: " + parseInt(e.receipt["gasUsed"]))
      if (type == PROCESS_TYPES.DEPOSIT) {
        console.log("deposit failed") // NOTE: this should never ever happen
        exit(1)
      }
      if (type == PROCESS_TYPES.WITHDRAW) {
        console.log("withdraw failed") // NOTE: this should never ever happen
        exit(1)
      }

      responses.push({
        // txhash: e.receipt["hash"], // NOTE: txhash is in receipt
        sender: sender,
        receipt: e.receipt,
        error: 'transaction execution reverted',
      })
    }
  }

  return responses
}

async function restore_to_last_ok_height(block_height) {
  const reverted = await provider_send(
    "brc20_reorg",
    { last_ok_height: parseInt(block_height) },
  );

  if (typeof reverted !== "boolean") {
    throw "Assertion error: the value returned by evm_revert should be a boolean"
  }

  if (!reverted) {
    throw "InvalidSnapshotError: the snapshot id is invalid or has been already used"
  }
}

var btc_pkscript_to_eth_address_cache = {}
function btc_pkscript_to_eth_address(btc_pkscript) {
  if (btc_pkscript_to_eth_address_cache[btc_pkscript]) return btc_pkscript_to_eth_address_cache[btc_pkscript]

  if (typeof btc_pkscript != "string") throw new Error("btc_pkscript is not a string")
  btc_pkscript = btc_pkscript.toLowerCase()
  if (btc_pkscript.match(/[^0-9a-f]/gi)) throw new Error("btc_pkscript contains non-hex characters")

  // keccak256 of the pkscript
  const pkscript_hash = ethers.keccak256("0x" + btc_pkscript)
  // last 20 bytes of the pkscript_hash
  const eth_address = pkscript_hash.slice(-40)

  btc_pkscript_to_eth_address_cache[btc_pkscript] = "0x" + eth_address
  return btc_pkscript_to_eth_address_cache[btc_pkscript]
}

// only view function call!!
async function call_a_function_of_a_smart_contract_as_a_specific_address(contract_addr, btc_pkscript, data) {
  const address = btc_pkscript_to_eth_address(btc_pkscript)

  const tx = {
    to: contract_addr,
    from: address,
    data: data,
  }
  let res = await provider_send("brc20_call", tx)

  return res
}

// send tx
function get_a_smart_contract_call_as_a_specific_address(contract_addr, btc_pkscript, data) {
  return {
    to: contract_addr,
    from: btc_pkscript_to_eth_address(btc_pkscript),
    data: data,
  }
}
function get_a_smart_contract_call_as_a_specific_eth_address(contract_addr, eth_address, data) {
  return {
    to: contract_addr,
    from: eth_address,
    data: data,
  }
}

class PROCESS_TYPES {
  static CALL = "call"
  static DEPLOY = "deploy"
  static DEPOSIT = "deposit"
  static WITHDRAW = "withdraw"
}
class PROCESS_ERRORS {
  static ERROR_NO_OP = "no_op"
  static ERROR_FAULTY_OP = "faulty_op"
}
class CALL_ERRORS {
  static ERROR_NO_C = "no_c"
  static ERROR_NO_F = "no_f"
  static ERROR_C_NOT_STRING = "c_not_string"
  static ERROR_F_NOT_STRING = "f_not_string"
  static ERROR_A_NOT_ARRAY = "a_not_array"
  static ERROR_ARG_NOT_OBJECT = "arg_not_object"
  static ERROR_ARG_NO_T = "arg_no_t"
  static ERROR_ARG_NO_V = "arg_no_v"
  static ERROR_ARG_T_NOT_STRING = "arg_t_not_string"
  static ERROR_ARG_V_NOT_STRING = "arg_v_not_string"
  static ERROR_ARG_T_NOT_PARAM_TYPE = "arg_t_not_param_type"
  static ERROR_EXCEPTION = "exception"
}
class DEPLOY_ERRORS {
  static ERROR_NO_SC = "no_sc"
  static ERROR_NO_CLS = "no_cls"
  static ERROR_SC_NOT_STRING = "sc_not_string"
  static ERROR_CLS_NOT_STRING = "cls_not_string"
  static ERROR_A_NOT_ARRAY = "a_not_array"
  static ERROR_ARG_NOT_OBJECT = "arg_not_object"
  static ERROR_ARG_NO_T = "arg_no_t"
  static ERROR_ARG_NO_V = "arg_no_v"
  static ERROR_ARG_T_NOT_STRING = "arg_t_not_string"
  static ERROR_ARG_V_NOT_STRING = "arg_v_not_string"
  static ERROR_ARG_T_NOT_PARAM_TYPE = "arg_t_not_param_type"
  static ERROR_CONTRACT_COMPILE = "contract_compile"
  static ERROR_EXCEPTION = "exception"
}

var get_ethers_interface_cache = {}
function get_ethers_interface(function_sig) {
  if (get_ethers_interface_cache[function_sig]) return get_ethers_interface_cache[function_sig]
  const interface = new ethers.Interface([function_sig])
  const fragment = interface.getFunction(function_sig);
  get_ethers_interface_cache[function_sig] = [interface, fragment]
  return get_ethers_interface_cache[function_sig]
}

// returns (type, value, error, sent_from)
async function process_inscription(inscription, btc_pkscript) {
  if (!inscription.op) return [null, null, PROCESS_ERRORS.ERROR_NO_OP, btc_pkscript];
  if (inscription.op == "call") {
    let contract_addr = null
    let data = null

    try {
      //if (Object.keys(inscription).indexOf('c') == -1) return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_NO_C, btc_pkscript];
      //if (Object.keys(inscription).indexOf('f') == -1) return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_NO_F, btc_pkscript];

      contract_addr = inscription.c
      let func_name = inscription.f

      //if (typeof contract_addr != "string") return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_C_NOT_STRING, btc_pkscript];
      //if (typeof func_name != "string") return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_F_NOT_STRING, btc_pkscript];

      let args = []
      //if (Object.keys(inscription).indexOf('a') != -1) {
      if (inscription.a) {
        args = inscription.a
        //if (!Array.isArray(args)) return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_A_NOT_ARRAY, btc_pkscript];
      }

      let arg_values = []
      for (const arg of args) {
        /* if (typeof arg != "object") return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_ARG_NOT_OBJECT, btc_pkscript];
        if (Object.keys(arg).indexOf('t') == -1) return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_ARG_NO_T, btc_pkscript];
        if (Object.keys(arg).indexOf('v') == -1) return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_ARG_NO_V, btc_pkscript];
        if (typeof arg.t != "string") return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_ARG_T_NOT_STRING, btc_pkscript];
        if (typeof arg.v != "string") return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_ARG_V_NOT_STRING, btc_pkscript];
        if (extra_arg_types.indexOf(arg.t) == -1) {
          try {
            ethers.ParamType.from(arg.t)
          } catch (e) {
            return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_ARG_T_NOT_PARAM_TYPE, btc_pkscript];
          }
        } */
        if (arg.t == "btc_address") {
          arg.v = btc_pkscript_to_eth_address(arg.v)
          arg.t = "address"
        }
        arg_values.push(arg.v)
      }

      // console.log("call: " + contract_addr + " " + func_name + " " + JSON.stringify(args))

      let function_sig = 'function ' + func_name + '('
      for (const arg of args) function_sig += arg.t + ','
      if (args.length > 0) function_sig = function_sig.slice(0, -1)
      function_sig += ')'

      let [contract, fragment] = get_ethers_interface(function_sig)
      data = contract.encodeFunctionData(fragment, arg_values)
    } catch (e) {
      console.error(e)
      return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_EXCEPTION, btc_pkscript];
    }
    let res = get_a_smart_contract_call_as_a_specific_address(contract_addr, btc_pkscript, data) // TODO: try again if throws with network error!!
    return [PROCESS_TYPES.CALL, res, null, btc_pkscript];
  } else if (inscription.op == "call2") {
    let contract_addr = null
    let data = null

    try {
      //if (Object.keys(inscription).indexOf('c') == -1) return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_NO_C, btc_pkscript];
      contract_addr = inscription.c
      //if (typeof contract_addr != "string") return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_C_NOT_STRING, btc_pkscript];

      //if (Object.keys(inscription).indexOf('d') == -1) return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_C_NOT_STRING, btc_pkscript];
      data = inscription.d
      //if (typeof data != "string") return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_C_NOT_STRING, btc_pkscript];
    } catch (e) {
      console.error(e)
      return [PROCESS_TYPES.CALL, null, CALL_ERRORS.ERROR_EXCEPTION, btc_pkscript];
    }

    let res = get_a_smart_contract_call_as_a_specific_address(contract_addr, btc_pkscript, data) // TODO: try again if throws with network error!!
    return [PROCESS_TYPES.CALL, res, null, btc_pkscript];
  } else if (inscription.op == 'deploy') {
    let contract_factory = null
    let arg_values = null

    try {
      //if (Object.keys(inscription).indexOf('sc') == -1) return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_NO_SC, btc_pkscript];
      let contract_source_code = inscription.sc
      //if (typeof contract_source_code != "string") return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_SC_NOT_STRING, btc_pkscript];

      //if (Object.keys(inscription).indexOf('cls') == -1) return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_NO_CLS, btc_pkscript];
      let deploy_class = inscription.cls
      //if (typeof deploy_class != "string") return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_CLS_NOT_STRING, btc_pkscript];

      let args = []
      //if (Object.keys(inscription).indexOf('a') != -1) {
      if (inscription.a) {
        args = inscription.a
        //if (!Array.isArray(args)) return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_A_NOT_ARRAY, btc_pkscript];
      }

      arg_values = []
      for (const arg of args) {
        /* if (typeof arg != "object") return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_ARG_NOT_OBJECT, btc_pkscript];
        if (Object.keys(arg).indexOf('t') == -1) return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_ARG_NO_T, btc_pkscript];
        if (Object.keys(arg).indexOf('v') == -1) return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_ARG_NO_V, btc_pkscript];
        if (typeof arg.t != "string") return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_ARG_T_NOT_STRING, btc_pkscript];
        if (typeof arg.v != "string") return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_ARG_V_NOT_STRING, btc_pkscript];
        if (extra_arg_types.indexOf(arg.t) == -1) {
          try {
            ethers.ParamType.from(arg.t)
          } catch (e) {
            return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_ARG_T_NOT_PARAM_TYPE, btc_pkscript];
          }
        } */
        if (arg.t == "btc_address") {
          arg.v = btc_pkscript_to_eth_address(arg.v)
          arg.t = "address"
        }
        arg_values.push(arg.v)
      }

      // console.log("deploy: " + contract_source_code + " " + deploy_class + " " + JSON.stringify(args))

      // compile contract
      let input = {
        language: 'Solidity',
        sources: {
          'main.sol': {
            content: contract_source_code
          }
        },
        settings: {
          evmVersion: "berlin",
          outputSelection: {
            '*': {
              '*': ['*']
            }
          }
        }
      };

      try {
        let compiled = JSON.parse(solc_fixed.compile(JSON.stringify(input)))
        let contract = compiled.contracts['main.sol'][deploy_class]
        let bytecode = contract.evm.bytecode.object
        let abi = contract.abi

        contract_factory = new ethers.ContractFactory(abi, bytecode, null)
      } catch (e) {
        console.error(e)
        return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_CONTRACT_COMPILE, btc_pkscript];
      }
    } catch (e) {
      console.error(e)
      return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_EXCEPTION, btc_pkscript];
    }

    let deploy_tx = await contract_factory.getDeployTransaction(...arg_values) // TODO: try again if throws with network error!!
    return [PROCESS_TYPES.DEPLOY, {
      from: btc_pkscript_to_eth_address(btc_pkscript),
      to: null,
      data: deploy_tx.data,
    }, null, btc_pkscript];
  } else if (inscription.op == 'deploy2') {
    //if (Object.keys(inscription).indexOf('bc') == -1) return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_NO_SC, btc_pkscript];
    let contract_byte_code = inscription.bc
    //if (typeof contract_byte_code != "string") return [PROCESS_TYPES.DEPLOY, null, DEPLOY_ERRORS.ERROR_SC_NOT_STRING, btc_pkscript];

    return [PROCESS_TYPES.DEPLOY, {
      from: btc_pkscript_to_eth_address(btc_pkscript),
      to: null,
      data: contract_byte_code,
    }, null, btc_pkscript];
  } else if (inscription.op == "deposit") {
    // already validated so just process
    let ticker = inscription.t
    let amount = inscription.a
    // console.log("deposit: " + ticker + " " + amount + " " + btc_pkscript)
    let arg_values = [ticker, btc_pkscript_to_eth_address(btc_pkscript), amount]
    let function_sig = 'function mint(string,address,uint256)'
    let [contract, fragment] = get_ethers_interface(function_sig)
    let data = contract.encodeFunctionData(fragment, arg_values)
    let res = get_a_smart_contract_call_as_a_specific_eth_address(brc20_controller_addr, indexer_addr, data) // TODO: try again if throws with network error!!
    return [PROCESS_TYPES.DEPOSIT, res, null, btc_pkscript];
  } else if (inscription.op == "withdraw") {
    // already validated so just process
    let ticker = inscription.t
    let amount = inscription.a
    // console.log("withdraw: " + ticker + " " + amount + " " + btc_pkscript)
    let arg_values = [ticker, btc_pkscript_to_eth_address(btc_pkscript), amount]
    let function_sig = 'function burn(string,address,uint256)'
    let [contract, fragment] = get_ethers_interface(function_sig)
    let data = contract.encodeFunctionData(fragment, arg_values)
    let res = get_a_smart_contract_call_as_a_specific_eth_address(brc20_controller_addr, indexer_addr, data) // TODO: try again if throws with network error!!
    return [PROCESS_TYPES.WITHDRAW, res, null, btc_pkscript];
  } else {
    return [null, null, PROCESS_ERRORS.ERROR_FAULTY_OP, btc_pkscript];
  }
}

async function check_balance_of_an_address(btc_address, ticker) {
  let function_sig = 'function balanceOf(string, address)'
  let [contract, fragment] = get_ethers_interface(function_sig)
  let data = contract.encodeFunctionData(fragment, [ticker, btc_pkscript_to_eth_address(btc_address)])
  console.log(data)
  let res = await call_a_function_of_a_smart_contract_as_a_specific_address(brc20_controller_addr, btc_address, data)
  return parseInt(res["callOutput"], 16)
}

async function call_view_function_as(btc_address, contract_addr, func_name, args) {
  if (typeof func_name != "string") return null
  if (!Array.isArray(args)) return null

  let arg_values = []
  for (const arg of args) {
    if (typeof arg != "object") return null
    if (Object.keys(arg).indexOf('t') == -1) return null
    if (Object.keys(arg).indexOf('v') == -1) return null
    if (typeof arg.t != "string") return null
    if (typeof arg.v != "string") return null
    if (extra_arg_types.indexOf(arg.t) == -1) {
      try {
        ethers.ParamType.from(arg.t)
      } catch (e) {
        return null
      }
    }
    if (arg.t == "btc_address") {
      arg.v = btc_pkscript_to_eth_address(arg.v)
      arg.t = "address"
    }
    arg_values.push(arg.v)
  }

  let function_sig = 'function ' + func_name + '('
  for (const arg of args) function_sig += arg.t + ','
  if (args.length > 0) function_sig = function_sig.slice(0, -1)
  function_sig += ')'

  let [contract, fragment] = get_ethers_interface(function_sig)
  let data = contract.encodeFunctionData(fragment, arg_values)

  return await call_a_function_of_a_smart_contract_as_a_specific_address(contract_addr, btc_address, data)
}



var app = express();
app.set('trust proxy', 0)

app.use(express.json({ limit: '50mb' }))

app.post('/mine_block', async (request, response) => {
  try {
    console.log(`${request.protocol}://${request.get('host')}${request.originalUrl}`)

    let ts = parseInt(request.body.ts)
    let hash = request.body.hash

    let block_txes = []
    for (const tx of request.body.txes) {
      block_txes.push({
        resp: await process_inscription(tx.inscription, tx.btc_pkscript),
      })
    }
    let resp = await mine_next_block(ts, hash, block_txes)

    response.send({
      error: null,
      result: {
        responses: resp,
      }
    })
  } catch (err) {
    console.log(err)
    response.status(500).send({ error: 'internal error', result: null })
  }
})

app.get('/check_balance', async (request, response) => {
  try {
    console.log(`${request.protocol}://${request.get('host')}${request.originalUrl}`)

    let btc_addr = request.query.btc_addr
    let ticker = request.query.ticker
    let resp = await check_balance_of_an_address(btc_addr, ticker)

    response.send({
      error: null,
      result: resp
    })
  } catch (err) {
    console.log(err)
    response.status(500).send({ error: 'internal error', result: null })
  }
})

app.post('/call_view_function_as', async (request, response) => {
  try {
    console.log(`${request.protocol}://${request.get('host')}${request.originalUrl}`)

    let btc_addr = request.body.btc_addr
    let contract_addr = request.body.contract_addr
    let func_name = request.body.func_name
    let args = request.body.args
    let resp = await call_view_function_as(btc_addr, contract_addr, func_name, args)

    response.send({
      error: null,
      result: resp
    })
  } catch (err) {
    console.log(err)
    response.status(500).send({ error: 'internal error', result: null })
  }
})

app.get('/commit_changes_to_db', async (request, response) => {
  try {
    console.log(`${request.protocol}://${request.get('host')}${request.originalUrl}`)

    let resp = await provider_send("brc20_commitToDatabase", {})

    response.send({
      error: null,
      result: resp
    })
  } catch (err) {
    console.log(err)
    response.status(500).send({ error: 'internal error', result: null })
  }
})

app.get('/current_block_height', async (request, response) => {
  try {
    console.log(`${request.protocol}://${request.get('host')}${request.originalUrl}`)

    let resp = parseInt(await provider_send("eth_blockNumber", {}))

    response.send({
      error: null,
      result: resp
    })
  } catch (err) {
    console.log(err)
    response.status(500).send({ error: 'internal error', result: null })
  }
})

app.get('/hash_of_height', async (request, response) => {
  try {
    console.log(`${request.protocol}://${request.get('host')}${request.originalUrl}`)

    let block_height = parseInt(request.query.block_height)
    let resp = (await get_info_of_block(block_height))['hash']

    response.send({
      error: null,
      result: resp
    })
  } catch (err) {
    console.log(err)
    response.status(500).send({ error: 'internal error', result: null })
  }
})

app.get('/restore_to_last_ok_height', async (request, response) => {
  try {
    console.log(`${request.protocol}://${request.get('host')}${request.originalUrl}`)

    let last_ok_height = request.query.last_ok_height
    await restore_to_last_ok_height(last_ok_height)

    response.send({
      error: null,
      result: null
    })
  } catch (err) {
    console.log(err)
    response.status(500).send({ error: 'internal error', result: null })
  }
})

app.get('/get_block_info', async (request, response) => {
  try {
    console.log(`${request.protocol}://${request.get('host')}${request.originalUrl}`)

    let block_height = parseInt(request.query.block_height)
    let res = await get_info_of_block(block_height)

    response.send({
      error: null,
      result: res
    })
  } catch (err) {
    console.log(err)
    response.status(500).send({ error: 'internal error', result: null })
  }
})

app.get('/get_contract_bytecode', async (request, response) => {
  try {
    console.log(`${request.protocol}://${request.get('host')}${request.originalUrl}`)

    let addr = request.query.addr
    let res = await provider_send("get_contract_bytecode", { "addr": addr })

    response.send({
      error: null,
      result: res
    })
  } catch (err) {
    console.log(err)
    response.status(500).send({ error: 'internal error', result: null })
  }
})

async function main() {
  let resp = parseInt(await provider_send("eth_blockNumber", {}))
  if (resp < module_activation_height - 1) {
    console.log("initialising chain")
    await initialise_chain()
    console.log("initialised chain")
  } else {
    console.log("checking chain")
    let res = await get_info_of_block(module_activation_height - 1)
    if (res["hash"] != block_hash_after_initialization) {
      console.error(`Block hash after initialization is ${res["hash"]} instead of ${block_hash_after_initialization}`);
      exit(1)
    }
    console.log("checked chain")
  }

  app.listen(8000, '0.0.0.0');
  console.log('Server running at http://0.0.0.0:8000/');
}
main()




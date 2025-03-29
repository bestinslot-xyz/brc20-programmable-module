const { exit } = require('node:process')
var express = require('express')

// Test environment (local)
//const BLOCK_HASH_AFTER_INIT = '0x0000000000000000000000000000000000000000000000000000000000000000'
//const module_activation_height = 1

// Mainnet
const BLOCK_HASH_AFTER_INIT = '0x00000000000000000000000000000000000000000000000000000000000be635'
const module_activation_height = 779830

// HELPER FUNCTIONS
async function provider_send(method, params) {
  let obj = {
    jsonrpc: "2.0",
    method: method,
    params: params,
    id: 1,
  }
  let r = await fetch('http://localhost:18545', { method: 'POST', body: JSON.stringify(obj), headers: { 'Content-Type': 'application/json' } })
  let j = await r.json()
  if (j != null && j.hasOwnProperty("jsonrpc") && j["jsonrpc"] == "2.0" && j.hasOwnProperty("result")) {
    return j["result"]
  }
  return j
}

async function brc20_deposit(ticker, pkscript, amount, timestamp, hash, tx_idx, inscription_id) {
  await provider_send("brc20_deposit", { ticker: ticker, to_pkscript: pkscript, amount: amount, timestamp: timestamp, hash: hash, tx_idx: tx_idx, inscription_id: inscription_id })
}

async function brc20_withdraw(ticker, pkscript, amount, timestamp, hash, tx_idx, inscription_id) {
  await provider_send("brc20_withdraw", { ticker: ticker, from_pkscript: pkscript, amount: amount, timestamp: timestamp, hash: hash, tx_idx: tx_idx, inscription_id: inscription_id })
}

async function brc20_balance(ticker, pkscript) {
  return parseInt(await provider_send("brc20_balance", { ticker: ticker, pkscript: pkscript }, 16))
}

async function brc20_mine(n, timestamp) {
  await provider_send("brc20_mine", { block_count: parseInt(n), timestamp: parseInt(timestamp) })
}

async function brc20_deploy(timestamp, hash, from, data, tx_idx, inscription_id) {
  return await provider_send("brc20_deploy", { timestamp: parseInt(timestamp), hash: hash, tx_idx: tx_idx, from_pkscript: from, data: data, inscription_id: inscription_id })
}

async function brc20_call(timestamp, hash, from, contract_address, data, tx_idx, inscription_id) {
  return await provider_send("brc20_call", { timestamp: parseInt(timestamp), contract_address: contract_address, hash: hash, tx_idx: tx_idx, from_pkscript: from, data: data, inscription_id: inscription_id })
}

async function brc20_finalise_block(timestamp, hash, block_tx_count) {
  return await provider_send("brc20_finaliseBlock", { timestamp: parseInt(timestamp), hash: hash, block_tx_count: block_tx_count })
}

async function eth_getBlockByNumber(block_number) {
  block_number = parseInt(block_number)
  block_number = "0x" + block_number.toString(16)
  let res = await provider_send(
    "eth_getBlockByNumber",
    { block: block_number },
  )

  return res
}

async function eth_blockNumber() {
  let res = await provider_send(
    "eth_blockNumber",
    {},
  )

  return parseInt(res)
}

async function brc20_initialise(genesis_hash, genesis_timestamp, genesis_height) {
  return await provider_send("brc20_initialise", { genesis_hash, genesis_timestamp, genesis_height })
}

// INITIALISATION FUNCTIONS
async function initialise_chain() {
  let init_st_tm = +(new Date())

  let current_block_height = await eth_blockNumber()
  if (current_block_height == 0) {
    // calling brc20_initialise to deploy genesis block
    await brc20_initialise("0x0000000000000000000000000000000000000000000000000000000000000000", 0, 0)
  }

  let st_tm = +(new Date())
  for (let i = current_block_height; i < module_activation_height - 1;) {
    console.log(`initialising block ${i} time per block ${(+(new Date()) - st_tm) / (i - current_block_height + 1)}`)
    let block_count = Math.min(100000, module_activation_height - i - 1)
    await brc20_mine(block_count, 0)
    i += block_count
  }

  let res = await eth_getBlockByNumber(module_activation_height - 1)
  if (res["hash"] != BLOCK_HASH_AFTER_INIT) {
    console.error(`Block hash after initialization is ${res["hash"]} instead of ${BLOCK_HASH_AFTER_INIT}`)

    throw new Error("Block hash after initialization does not match")
  }

  console.log(`initialisation took ${(+(new Date()) - init_st_tm) / 1000} seconds`)
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
  static ERROR_CALL_NO_C = "no_c"
  static ERROR_CALL_NO_D = "no_d"
}

var app = express()
app.set('trust proxy', 0)

app.use(express.json({ limit: '50mb' }))

app.post('/mine_block', async (request, response) => {
  try {
    console.log(`${request.protocol}://${request.get('host')}${request.originalUrl}`)

    let ts = parseInt(request.body.ts)
    let hash = request.body.hash

    let responses = []
    let tx_idx = 0
    for (var i = 0; i < request.body.txes.length; i++) {
      let btc_pkscript = request.body.txes[i].btc_pkscript
      let inscription = request.body.txes[i].inscription
      let inscription_id = inscription.inscription_id
      responses[i] = {
        sender: btc_pkscript,
      }
      if (!inscription.op) {
        responses[i].error = PROCESS_ERRORS.ERROR_NO_OP
        continue
      }
      if (inscription.op == "call") {
        if (!inscription.c) {
          responses[i].error = PROCESS_ERRORS.ERROR_CALL_NO_C
          continue
        }
        if (!inscription.d) {
          responses[i].error = PROCESS_ERRORS.ERROR_CALL_NO_D
          continue
        }
        responses[i].tx = {
          type: PROCESS_TYPES.CALL,
          to: inscription.c,
          data: inscription.d,
        }
        responses[i].receipt = await brc20_call(ts, hash, btc_pkscript, inscription.c, inscription.d, tx_idx, inscription_id)
        tx_idx += 1
      } else if (inscription.op == 'deploy') {
        responses[i].tx = {
          type: PROCESS_TYPES.DEPLOY,
          data: inscription.d,
        }
        responses[i].receipt = await brc20_deploy(ts, hash, btc_pkscript, inscription.d, tx_idx, inscription_id)
        tx_idx += 1
      } else if (inscription.op == "transfer") {
        let amount = inscription.amt
        amount = int(amount * 1e18)
        responses[i].tx = {
          type: PROCESS_TYPES.DEPOSIT,
          info: {
            to: brc20_controller_addr,
            ticker: inscription.tick,
            amount
          }
        }
        responses[i].receipt = await brc20_deposit(inscription.tick, btc_pkscript, amount, ts, hash, tx_idx, inscription_id)
        tx_idx += 1
      } else if (inscription.op == "withdraw") {
        let ticker = inscription.tick
        let amount = inscription.amt
        amount = int(amount * 1e18)
        responses[i].tx = {
          type: PROCESS_TYPES.WITHDRAW,
          info: {
            from: brc20_controller_addr,
            ticker: ticker,
            amount
          }
        }
        responses[i].receipt = await brc20_withdraw(ticker, btc_pkscript, amount, ts, hash, tx_idx, inscription_id)
        tx_idx += 1
      } else {
        responses[i].error = PROCESS_ERRORS.ERROR_FAULTY_OP
        continue
      }
    }
    await brc20_finalise_block(ts, hash, tx_idx)

    response.send({
      error: null,
      result: {
        responses: responses
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
    let resp = await brc20_balance(ticker, btc_addr)

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
    let resp = (await eth_getBlockByNumber(block_height))['hash']

    response.send({
      error: null,
      result: resp
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
    let res = await eth_getBlockByNumber(block_height)

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
  let resp = await eth_blockNumber()
  if (resp < module_activation_height) {
    console.log("initialising chain")
    await initialise_chain()
    console.log("initialised chain")
  } else {
    console.log("checking chain")
    let block_info = await eth_getBlockByNumber(module_activation_height - 1)
    if (block_info['hash'] != BLOCK_HASH_AFTER_INIT) {
      console.error(`Block hash after initialization is ${block_info['hash']} instead of ${BLOCK_HASH_AFTER_INIT}`)
      exit(1)
    }
    console.log("checked chain")
  }

  app.listen(8000, '0.0.0.0')
  console.log('Server running at http://0.0.0.0:8000/')
}
main()




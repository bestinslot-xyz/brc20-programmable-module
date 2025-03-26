# Block size of 4MB allows around 10k calls per block for BTC tx details precompile, assuming each call is around 512 bytes
# This is a very rough, potentially worst-case estimate, and the actual number of calls per block will vary.

# The performance test script is a simple script that calls a precompile contract 10k times.

# The script is run with the following command:
# python3 performance_test_single_block_precompile.py

import json
import sys
import time
from brc20_prog.brc20_prog_client import BRC20ProgClient

# Initialize the BRC20ProgClient
client = BRC20ProgClient()

print("Starting performance test...")
start_time = time.time()


# Precompile data
def load_tx_data(file):
    data = open(file).read()
    data = json.loads(data)["d"]
    return data


deploy_data = load_tx_data("contracts/brc20_prog_helper/BRC20_Prog_deploy_tx.json")

tx_data_precompiles = [
    (
        "Random Number (Contract only)",
        "",
        load_tx_data(
            "contracts/brc20_prog_helper/BRC20_Prog_get_random_number_tx.json"
        ),
        "0x000000000000000000000000000000000000000000000000000000000000002a",
    ),
    (
        "Sha256 (Built-in Precompile at 0x02)",
        "",
        load_tx_data("contracts/brc20_prog_helper/BRC20_Prog_get_sha_256_tx.json"),
        "0xa591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e",
    ),
    (
        "BIP322 (Custom Precompile at 0xFE)",
        "0x00000000000000000000000000000000000000fe",
        load_tx_data("contracts/brc20_prog_helper/BRC20_Prog_bip322_verify_tx.json"),
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    ),
    (
        "Locked PkScript (Custom Precompile at 0xFB)",
        "0x00000000000000000000000000000000000000fb",
        load_tx_data(
            "contracts/brc20_prog_helper/BRC20_Prog_btc_locked_pkscript_tx.json"
        ),
        "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003e7462317030356136663939757a63676b756167346478393677363663707337723978396b6664746b303530797939387133367864367666717975717164730000",
    ),
    (
        "BRC20 Balance (Custom Precompile at 0xFF)",
        "0x00000000000000000000000000000000000000ff",
        load_tx_data("contracts/brc20_prog_helper/BRC20_Prog_brc20_balance_tx.json"),
        "0x000000000000000000000000000000000000000000000000000000000000001a",
    ),
    (
        "BTC Last Sat Location (Custom Precompile at 0xFC)",
        "0x00000000000000000000000000000000000000fc",
        load_tx_data("contracts/brc20_prog_helper/BRC20_Prog_btc_last_sat_loc_tx.json"),
        "0x00000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000013000000000000000000000000000000000000000000000000000000000003d090000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000040386434626333616332313231313732333433366533356666626633326135386637346665393432653065613130393336353034646230376166623161663763330000000000000000000000000000000000000000000000000000000000000044353132303461363034316635346238636638623264343863366637323563623035313465353165356537653761633432396333336461363265393837363564643632663300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002c30303134663437373935326633333536316331623839613166653966323836383266363233323633653135390000000000000000000000000000000000000000",
    ),
    (
        "BTC Tx Details (Custom Precompile at 0xFD)",
        "0x00000000000000000000000000000000000000fd",
        load_tx_data("contracts/brc20_prog_helper/BRC20_Prog_btc_tx_details_tx.json"),
        "0x000000000000000000000000000000000000000000000000000000000003ad4000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001c0000000000000000000000000000000000000000000000000000000000000028000000000000000000000000000000000000000000000000000000000000002c0000000000000000000000000000000000000000000000000000000000000036000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000004038643462633361633231323131373233343336653335666662663332613538663734666539343265306561313039333635303464623037616662316166376333000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000130000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000443531323034613630343166353462386366386232643438633666373235636230353134653531653565376537616334323963333364613632653938373635646436326633000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000098968000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002c3030313466343737393532663333353631633162383961316665396632383638326636323332363365313539000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000935e90",
    ),
]

precompile_to_test = 5  # btc tx details

print("Testing calls to " + tx_data_precompiles[precompile_to_test][0])
tx_data = tx_data_precompiles[precompile_to_test][2]

# Number of calls to perform
call_cnt = 10000
btc_pkscript = "7465737420706b736372697074"

timestamp = int(time.time())
block_hash = "0x" + "0" * 64

client.clear_caches()

# deploy first
contract_address = client.add_tx_to_block(
    from_pkscript=btc_pkscript,
    contract_address=None,
    data=deploy_data,
    timestamp=timestamp,
    block_hash=block_hash,
)[0]
print("Deployed contract with address: " + contract_address)

# Temporarily call the precompile directly, there's a bug in the proxy contract that prevents calling the precompile
# contract_address="0x00000000000000000000000000000000000000fd"

for i in range(call_cnt):
    try:
        result = client.add_tx_to_block(
            from_pkscript=btc_pkscript,
            contract_address=contract_address,
            data=tx_data,
            timestamp=timestamp,
            block_hash=block_hash,
        )
        if result[1] == False or result[2] != tx_data_precompiles[precompile_to_test][3]:
            print("Call " + str(i) + " failed")
            sys.exit(1)
        print("Call " + str(i) + " with result: " + str(result[1]))
    except Exception as e:
        print("Call " + str(i) + " failed with exception: " + str(e))
        sys.exit(1)

print("Performance test complete")
print("Time taken: " + str(time.time() - start_time) + " seconds")
print("Total data sent: " + str(len(tx_data) * call_cnt) + " bytes")
print("Total data sent (MB): " + str(len(tx_data) * call_cnt / 1024 / 1024) + " MB")
print("Called " + str(call_cnt) + " times")
print(
    "Average time per call: " + str((time.time() - start_time) / call_cnt) + " seconds"
)

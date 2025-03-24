
# Block size of 4MB allows around 10k calls per block for BTC tx details, assuming each call is around 512 bytes
# This is a very rough, potentially worst-case estimate, and the actual number of calls per block will vary.

# The performance test script is a simple script that calls the btc tx details contract 10k times.

# The script is run with the following command:
# python3 performance_test_single_block_btc_tx_details.py

import json
import sys
import time
from brc20_prog.brc20_prog_client import BRC20ProgClient

# Initialize the BRC20ProgClient
client = BRC20ProgClient()

print("Starting performance test...")
start_time = time.time()

deploy_data = open("contracts/brc20_prog_helper/BRC20_Prog_deploy_tx.json").read()
deploy_data = json.loads(deploy_data)["d"]

btc_tx_data = open("contracts/brc20_prog_helper/BRC20_Prog_btc_tx_details_tx.json").read()
btc_tx_data = json.loads(btc_tx_data)["d"]

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
    block_hash=block_hash
)[0]
print("Deployed contract with address: " + contract_address)

# Temporarily call the precompile directly, there's a bug in the proxy contract that prevents calling the precompile
# contract_address="0x00000000000000000000000000000000000000fd"

for i in range(call_cnt):
    result = client.add_tx_to_block(
        from_pkscript=btc_pkscript,
        contract_address=contract_address,
        data=btc_tx_data,
        timestamp=timestamp,
        block_hash=block_hash
    )
    if result[1] == False:
        print("Call " + str(i) + " failed")
        break
    else:
        print("Call " + str(i) + " with result: " + str(result[1]))

client.finalise_block(
    block_hash=block_hash,
    timestamp=timestamp
)

print("Performance test complete")
print("Time taken: " + str(time.time() - start_time) + " seconds")
print("Total data sent: " + str(len(btc_tx_data) * call_cnt) + " bytes")
print("Total data sent (MB): " + str(len(btc_tx_data) * call_cnt / 1024 / 1024) + " MB")
print("Called " + str(call_cnt) + " times")
print("Average time per call: " + str((time.time() - start_time) / call_cnt) + " seconds")

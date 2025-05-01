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
        "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002251205b85902c60f0e252ddaedade40ba1f115d483305c00172a15dbccc58fcf8eb55000000000000000000000000000000000000000000000000000000000000",
    ),
    (
        "BRC20 Balance (Custom Precompile at 0xFF)",
        "0x00000000000000000000000000000000000000ff",
        load_tx_data("contracts/brc20_prog_helper/BRC20_Prog_brc20_balance_tx.json"),
        "", # No expected result
    ),
    (
        "BTC Last Sat Location (Custom Precompile at 0xFC)",
        "0x00000000000000000000000000000000000000fc",
        load_tx_data("contracts/brc20_prog_helper/BRC20_Prog_btc_last_sat_loc_tx.json"),
        "0x8d4bc3ac21211723436e35ffbf32a58f74fe942e0ea10936504db07afb1af7c30000000000000000000000000000000000000000000000000000000000000013000000000000000000000000000000000000000000000000000000000003d09000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000002251204a6041f54b8cf8b2d48c6f725cb0514e51e5e7e7ac429c33da62e98765dd62f300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000160014f477952f33561c1b89a1fe9f28682f623263e15900000000000000000000",
    ),
    (
        "BTC Tx Details (Custom Precompile at 0xFD)",
        "0x00000000000000000000000000000000000000fd",
        load_tx_data("contracts/brc20_prog_helper/BRC20_Prog_btc_tx_details_tx.json"),
        "0x000000000000000000000000000000000000000000000000000000000003ad4000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002c000000000000000000000000000000000000000000000000000000000000000018d4bc3ac21211723436e35ffbf32a58f74fe942e0ea10936504db07afb1af7c30000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000001300000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002251204a6041f54b8cf8b2d48c6f725cb0514e51e5e7e7ac429c33da62e98765dd62f3000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000009896800000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000160014f477952f33561c1b89a1fe9f28682f623263e1590000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000935e90",
    ),
]

btc_pkscript = "7465737420706b736372697074"  # "test pkscript"

block_hash = "0x" + "0" * 64
timestamp = int(time.time())

client.clear_caches()

# deploy first
contract_address = client.deploy(
    from_pkscript=btc_pkscript,
    data=deploy_data,
    timestamp=timestamp,
    block_hash=block_hash,
    inscription_id=None,
)

if contract_address is None:
    print("Failed to deploy contract")
    sys.exit(1)
print("Deployed contract with address: " + contract_address)

failed_tests = []

for i in range(len(tx_data_precompiles)):
    print("Testing " + tx_data_precompiles[i][0])
    if tx_data_precompiles[i][1] != "":
        print(
            "Directly accessing the precompile at address: " + tx_data_precompiles[i][1]
        )
        result = client.call(
            from_pkscript=btc_pkscript,
            contract_address=tx_data_precompiles[i][1],
            contract_inscription_id=None,
            data=tx_data_precompiles[i][2],
            timestamp=timestamp,
            block_hash=block_hash,
            inscription_id=None,
        )
        if result["status"] != "0x1" or (tx_data_precompiles[i][3] and result["output"] != tx_data_precompiles[i][3]):
            print("Direct call for " + tx_data_precompiles[i][0] + " failed")
            failed_tests.append("Direct - " + tx_data_precompiles[i][0])
            input("Press Enter to continue...")

    result = client.call(
        from_pkscript=btc_pkscript,
        contract_address=contract_address,
        contract_inscription_id=None,
        data=tx_data_precompiles[i][2],
        timestamp=timestamp,
        block_hash=block_hash,
    )
    if result["status"] != "0x1" or (tx_data_precompiles[i][3] and result["output"] != tx_data_precompiles[i][3]):
        print("Proxy call for " + tx_data_precompiles[i][0] + " failed")
        failed_tests.append("Proxy - " + tx_data_precompiles[i][0])
        input("Press Enter to continue...")

client.finalise_block(block_hash=block_hash, timestamp=timestamp)

print("Test complete\n")
for test in failed_tests:
    print("Failed test: " + test)

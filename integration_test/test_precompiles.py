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

deploy_data = open("contracts/brc20_prog_helper/BRC20_Prog_deploy_tx.json").read()
deploy_data = json.loads(deploy_data)["d"]

tx_data_precompiles = [
    ("Random Number (Contract only)", "0xdbdff2c1"),
    (
        "Sha256 (Built-in Precompile at 0x02)",
        "0x4dfb0d820000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000b48656c6c6f20576f726c64000000000000000000000000000000000000000000",
    ),
    (
        "BIP322 (Custom Precompile at 0xFE)",
        "0xdb273f73000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000002a6263317139767a61326538783537336e637a726c7a6d7330777678336773716a7837766176676b78306c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b48656c6c6f20576f726c640000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000090416b677752514968414f7a79796e6c717439336c4f4b4a722b776d6d7849656e732f2f7a507a6c397471494f75613933774f364d41694269356e35457941635053634f6a66316c4171495549517472337a4b4e656176596162487952386547686f7745684173667849414d5a5a454b5550595749344272756841516a7a46543846534653616a75467772444c3159687900000000000000000000000000000000",
    ),
    (
        "Locked PkScript (Custom Precompile at 0xFB)",
        "0x08130e9900000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000003e746231706c6e77393537376b6464786e34727933377873756c3939643034747037773373663063636c74366b307a633775336c3873776d733776667034380000",
    ),
    (
        "BRC20 Balance (Custom Precompile at 0xFF)",
        "0xed0998f4000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000004626c656800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a6263317139767a61326538783537336e637a726c7a6d7330777678336773716a7837766176676b78306c00000000000000000000000000000000000000000000",
    ),
    (
        "BTC Last Sat Location (Custom Precompile at 0xFC)",
        "0xbd1b7b1300000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000003d090000000000000000000000000000000000000000000000000000000000000004034313833666237333362393535336361386239333230386339316464613138626565336430623835313037323062313564373664393739616637666439393236",
    ),
    (
        "BTC Tx Details (Custom Precompile at 0xFD)",
        "0x96327323000000000000000000000000000000000000000000000000000000000000004034313833666237333362393535336361386239333230386339316464613138626565336430623835313037323062313564373664393739616637666439393236",
    ),
]

btc_pkscript = "7465737420706b736372697074"  # "test pkscript"

block_hash = "0x" + "0" * 64
timestamp = int(time.time())

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

for i in range(len(tx_data_precompiles)):
    try:
        result = client.add_tx_to_block(
            from_pkscript=btc_pkscript,
            contract_address=contract_address,
            data=tx_data_precompiles[i][1],
            timestamp=timestamp,
            block_hash=block_hash,
        )
        if result[1] == False:
            print("Call for " + tx_data_precompiles[i][0] + " failed")
            sys.exit(1)
    except:
        print("Call for " + tx_data_precompiles[i][0] + " failed")
        sys.exit(1)
    print("Call for " + tx_data_precompiles[i][0] + " succeeded")

client.finalise_block(block_hash=block_hash, timestamp=timestamp)

print("Test complete")

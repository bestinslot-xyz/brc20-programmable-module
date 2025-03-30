import psycopg2, requests, time
from Crypto.Hash import keccak

session = requests.Session()

conn = psycopg2.connect(
    host="127.0.0.1", 
    database="postgres", 
    user="postgres", 
    password="password"
)
conn.autocommit = True
cur = conn.cursor()


def get_evm_height():
    url = "http://localhost:8000/current_block_height"
    response = session.get(url)
    return response.json()["result"]


initial_block_height = 779830
current_block_height = get_evm_height() + 1
cur.execute("SELECT max(block_height) from brc20_block_hashes;")
max_block_height = cur.fetchone()[0]

contract_deploy_tx = {
    "inscription": {
        "inscription_id": "0000000000000000000000000000000000000000000000000000000000000000i0",
        "op": "deploy",
        "d": "0x"
        + open("contracts/brc20_deployer/BRC20_Deployer.bin")
        .read()
        .strip(),
    },
    "btc_pkscript": "512037679ea62eab55ebfd442c53c4ad46b6b75e45d8a8fa9cb31a87d0df268b029a",
}


def get_addr_data(btc_pkscript):
    k = keccak.new(digest_bits=256)
    k.update(btc_pkscript.encode('ASCII'))
    return k.hexdigest()[-40:].zfill(64)

def get_block_ts(hash):
    err_cnt = 0
    while True:
        try:
            r = requests.get('https://mempool.space/api/block/' + hash)
            j = r.json()
            return j['timestamp']
        except:
            err_cnt += 1
            if err_cnt == 5:
                print("TOO MANY ERRORS!!")
                return None
            time.sleep(1.0)

handled_block_cnt = 0
while current_block_height <= max_block_height:
    if handled_block_cnt % 1000 == 0:
        print("Committing to db...")
        st_tm = time.time()
        url = "http://localhost:8000/commit_changes_to_db"
        response = session.get(url)
        if response.status_code != 200:
            print("Error committing to db")
            exit(1)
        print("Committed to db in " + str(time.time() - st_tm) + " seconds")
    block_txes = []
    if current_block_height == initial_block_height:
        print("Deploying BRC20 Deployer contract...")
        block_txes.append(contract_deploy_tx)
    cur.execute(
        "SELECT block_hash from brc20_block_hashes where block_height = %s;",
        (current_block_height,),
    )
    btc_hash = "0x" + cur.fetchone()[0]
    #block_ts = get_block_ts(btc_hash[2:])
    #if block_ts is None:
    #    exit(5)
    btc_ts = 0 #int(block_ts)
    cur.execute(
        "SELECT event_type, event, inscription_id from brc20_events where block_height = %s order by id asc;",
        (current_block_height,),
    )
    events = cur.fetchall()
    deploy_cnt = 0
    for event in events:
        inscription_id = event[2]
        event_tx = {
            "inscription": {
                "inscription_len": 1000, ## dummy value
                "inscription_id": inscription_id,
                "op": "call",
                "c": "0x11bc79b28ab26101d4cb2cbdd4d5c2ceeea49efb",
                "d": "0x",
            }
        }
        if event[0] == 0:
            data = "0x07570dbb"
            # event_tx['inscription']['f'] = "deploy_inscribe"
            data += "0000000000000000000000000000000000000000000000000000000000000080"  ## offset of string
            data += hex(int(event[1]["limit_per_mint"]))[2:].zfill(64)
            data += hex(int(event[1]["max_supply"]))[2:].zfill(64)
            data += hex(int(event[1]["decimals"]))[2:].zfill(64)
            tick_hex = event[1]["tick"].encode("utf-8").hex()
            ln = len(tick_hex) // 2
            data += hex(ln)[2:].zfill(64)
            data += tick_hex.ljust(64, "0")

            event_tx["inscription"]["d"] = data
            event_tx["btc_pkscript"] = event[1]["deployer_pkScript"]
            deploy_cnt += 1
        elif event[0] == 1:
            data = "0x50f87a62"
            # event_tx['inscription']['f'] = "mint_inscribe"
            data += "0000000000000000000000000000000000000000000000000000000000000040"  ## offset of string
            data += hex(int(event[1]["amount"]))[2:].zfill(64)
            tick_hex = event[1]["tick"].encode("utf-8").hex()
            ln = len(tick_hex) // 2
            data += hex(ln)[2:].zfill(64)
            data += tick_hex.ljust(64, "0")

            event_tx["inscription"]["d"] = data
            event_tx["btc_pkscript"] = event[1]["minted_pkScript"]
        elif event[0] == 2:
            data = "0xf0b10bb3"
            # event_tx['inscription']['f'] = "transfer_inscribe"
            data += "0000000000000000000000000000000000000000000000000000000000000060"  ## offset of string
            data += hex(int(event[1]["amount"]))[2:].zfill(64)
            data += get_addr_data(event[1]["source_pkScript"])
            tick_hex = event[1]["tick"].encode("utf-8").hex()
            ln = len(tick_hex) // 2
            data += hex(ln)[2:].zfill(64)
            data += tick_hex.ljust(64, "0")

            event_tx["inscription"]["d"] = data
            event_tx["inscription"]["inscription_id"] += '0' ## since event type 3 also uses the same inscr_id, add another 0 to differentiate
            event_tx["btc_pkscript"] = event[1]["source_pkScript"]
        elif event[0] == 3:
            data = "0x3b63e221"
            # event_tx['inscription']['f'] = "transfer_transfer"
            data += "0000000000000000000000000000000000000000000000000000000000000060"  ## offset of string
            data += hex(int(event[1]["amount"]))[2:].zfill(64)
            to = event[1]["spent_pkScript"]
            if to is None:
                to = event[1]["source_pkScript"]
            data += get_addr_data(to)
            tick_hex = event[1]["tick"].encode("utf-8").hex()
            ln = len(tick_hex) // 2
            data += hex(ln)[2:].zfill(64)
            data += tick_hex.ljust(64, "0")

            event_tx["inscription"]["d"] = data
            event_tx["btc_pkscript"] = event[1]["source_pkScript"]
        block_txes.append(event_tx)
    st_tm = time.time()
    url = "http://localhost:8000/mine_block"
    to_send = {"ts": btc_ts, "hash": btc_hash, "txes": block_txes}
    response = session.post(url, json=to_send).json()
    if response["error"] is not None:
        print(response["error"])
        exit(1)
    for i in range(len(response["result"]["responses"])):
        resp = response["result"]["responses"][i]
        if "error" in resp:
            print(resp)
            print(block_txes[i])
            exit(1)
    print(
        "Block "
        + str(current_block_height)
        + " mined in "
        + str(int((time.time() - st_tm) * 10**3)).rjust(9)
        + " ms containing "
        + str(len(block_txes)).rjust(4)
        + " transactions and "
        + str(deploy_cnt).rjust(2)
        + " deployments."
    )
    current_block_height += 1
    handled_block_cnt += 1

print("Committing to db...")
url = "http://localhost:8000/commit_changes_to_db"
response = session.get(url)
if response.status_code != 200:
    print("Error committing to db")
    exit(1)

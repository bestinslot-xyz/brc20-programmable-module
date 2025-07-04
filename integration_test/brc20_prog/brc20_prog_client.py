import os
import requests
from typing import Dict

brc20_prog_enabled = (os.getenv("BRC20_PROG_ENABLED") or "true") == "true"
brc20_prog_rpc_url = os.getenv("BRC20_PROG_RPC_URL") or "http://localhost:18545"


def jsonrpc_call(method: str, params: Dict):
    response = requests.post(
        brc20_prog_rpc_url,
        json={
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": "brc20_index",
        },
    )
    return response.json()


class BRC20ProgClient:
    def __init__(self):
        self.current_block_hash = ""
        self.current_block_timestamp = 0
        self.current_block_tx_idx = 0

    def verify_block_hash_and_timestamp(self, block_hash: str, timestamp: int):
        if self.current_block_hash == "" and self.current_block_timestamp == 0:
            self.current_block_hash = block_hash
            self.current_block_timestamp = timestamp
            self.current_block_tx_idx = 0
        elif self.current_block_hash != block_hash:
            raise Exception("Block hash mismatch")
        elif self.current_block_timestamp != timestamp:
            raise Exception("Block timestamp mismatch")

    def deploy(
        self,
        from_pkscript: str,
        data: str,
        timestamp: int,
        block_hash: str,
        inscription_id: str,
    ) -> tuple[str, bool, str]:
        if not brc20_prog_enabled:
            return
        self.verify_block_hash_and_timestamp(block_hash, timestamp)

        tx_result = jsonrpc_call(
            "brc20_deploy",
            params={
                "from_pkscript": from_pkscript,
                "data": data,
                "timestamp": timestamp,
                "hash": block_hash,
                "tx_idx": self.current_block_tx_idx,
                "inscription_id": inscription_id,
            },
        )

        if "error" in tx_result:
            raise Exception(tx_result["error"])

        if tx_result["result"]["status"] == "0x0":
            print("Transaction failed")
            print(tx_result)

        self.current_block_tx_idx += 1
        return tx_result["result"]["contractAddress"]

    def call(
        self,
        from_pkscript: str,
        contract_address: str,
        contract_inscription_id: str,
        data: str,
        timestamp: int,
        block_hash: str,
        inscription_id: str = None,
        inscription_byte_len: int = 1024,
    ):
        if not brc20_prog_enabled:
            return
        self.verify_block_hash_and_timestamp(block_hash, timestamp)

        tx_result = jsonrpc_call(
            "brc20_call",
            params={
                "from_pkscript": from_pkscript,
                "contract_address": contract_address,
                "contract_inscription_id": contract_inscription_id,
                "data": data,
                "timestamp": timestamp,
                "hash": block_hash,
                "tx_idx": self.current_block_tx_idx,
                "inscription_id": inscription_id,
                "inscription_byte_len": inscription_byte_len,
            },
        )

        if "error" in tx_result:
            raise Exception(tx_result["error"])

        if tx_result["result"]["status"] == "0x0":
            print("Transaction failed")

        self.current_block_tx_idx += 1
        return tx_result["result"]

    def finalise_block(self, block_hash: str, timestamp: int):
        if not brc20_prog_enabled:
            return
        self.verify_block_hash_and_timestamp(block_hash, timestamp)

        result = jsonrpc_call(
            "brc20_finaliseBlock",
            params={
                "hash": block_hash,
                "timestamp": timestamp,
                "block_tx_count": self.current_block_tx_idx,
            },
        )

        if "error" in result:
            raise Exception(result["error"])

        self.reset_current_block()

    def get_block(self, block_height: int):
        if not brc20_prog_enabled:
            return ""
        result = jsonrpc_call(
            "eth_getBlockByNumber", {"block": str(block_height), "is_full": True}
        )
        if "error" in result:
            return None

        return result["result"]

    def get_block_height(self):
        if not brc20_prog_enabled:
            return 0
        return int(jsonrpc_call("eth_blockNumber", {})["result"], 0)

    def reset_current_block(self):
        self.current_block_hash = ""
        self.current_block_timestamp = 0
        self.current_block_tx_idx = 0


if __name__ == "__main__":
    if brc20_prog_enabled == "true":
        print("BRC20 Prog enabled")
    else:
        print("BRC20 Prog disabled")

    client = BRC20ProgClient()
    print("BRC20 RPC URL: " + brc20_prog_rpc_url)
    print("Block height: " + str(client.get_block_height()))

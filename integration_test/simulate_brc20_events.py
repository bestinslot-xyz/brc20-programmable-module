import psycopg2, requests, json, time

conn = psycopg2.connect(
    host="127.0.0.1",
    database="postgres",
    user="postgres",
    password="password")
conn.autocommit = True
cur = conn.cursor()

def get_evm_height():
  url = 'http://localhost:8000/current_block_height'
  response = requests.get(url)
  return response.json()['result']

def get_evm_ts(block_height):
  url = 'http://localhost:8000/get_block_info?block_height=' + str(block_height)
  response = requests.get(url)
  return response.json()['result']["timestamp"]

initial_block_height = 779832
current_block_height = get_evm_height() + 1
cur.execute('SELECT max(block_height) from brc20_block_timestamps;')
max_block_height = cur.fetchone()[0]

contract_deploy_tx = {
  "inscription": {
    "op": "deploy",
    "cls": "BRC20_Deployer",
    "sc": '// SPDX-License-Identifier: MIT\n\n// File: @openzeppelin/contracts/utils/Context.sol\n\n\n// OpenZeppelin Contracts (last updated v5.0.1) (utils/Context.sol)\n\npragma solidity ^0.8.20;\n\n/**\n * @dev Provides information about the current execution context, including the\n * sender of the transaction and its data. While these are generally available\n * via msg.sender and msg.data, they should not be accessed in such a direct\n * manner, since when dealing with meta-transactions the account sending and\n * paying for execution may not be the actual sender (as far as an application\n * is concerned).\n *\n * This contract is only required for intermediate, library-like contracts.\n */\nabstract contract Context {\n    function _msgSender() internal view virtual returns (address) {\n        return msg.sender;\n    }\n\n    function _msgData() internal view virtual returns (bytes calldata) {\n        return msg.data;\n    }\n\n    function _contextSuffixLength() internal view virtual returns (uint256) {\n        return 0;\n    }\n}\n\n// File: @openzeppelin/contracts/access/Ownable.sol\n\n\n// OpenZeppelin Contracts (last updated v5.0.0) (access/Ownable.sol)\n\npragma solidity ^0.8.20;\n\n\n/**\n * @dev Contract module which provides a basic access control mechanism, where\n * there is an account (an owner) that can be granted exclusive access to\n * specific functions.\n *\n * The initial owner is set to the address provided by the deployer. This can\n * later be changed with {transferOwnership}.\n *\n * This module is used through inheritance. It will make available the modifier\n * `onlyOwner`, which can be applied to your functions to restrict their use to\n * the owner.\n */\nabstract contract Ownable is Context {\n    address private _owner;\n\n    /**\n     * @dev The caller account is not authorized to perform an operation.\n     */\n    error OwnableUnauthorizedAccount(address account);\n\n    /**\n     * @dev The owner is not a valid owner account. (eg. `address(0)`)\n     */\n    error OwnableInvalidOwner(address owner);\n\n    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);\n\n    /**\n     * @dev Initializes the contract setting the address provided by the deployer as the initial owner.\n     */\n    constructor(address initialOwner) {\n        if (initialOwner == address(0)) {\n            revert OwnableInvalidOwner(address(0));\n        }\n        _transferOwnership(initialOwner);\n    }\n\n    /**\n     * @dev Throws if called by any account other than the owner.\n     */\n    modifier onlyOwner() {\n        _checkOwner();\n        _;\n    }\n\n    /**\n     * @dev Returns the address of the current owner.\n     */\n    function owner() public view virtual returns (address) {\n        return _owner;\n    }\n\n    /**\n     * @dev Throws if the sender is not the owner.\n     */\n    function _checkOwner() internal view virtual {\n        if (owner() != _msgSender()) {\n            revert OwnableUnauthorizedAccount(_msgSender());\n        }\n    }\n\n    /**\n     * @dev Leaves the contract without owner. It will not be possible to call\n     * `onlyOwner` functions. Can only be called by the current owner.\n     *\n     * NOTE: Renouncing ownership will leave the contract without an owner,\n     * thereby disabling any functionality that is only available to the owner.\n     */\n    function renounceOwnership() public virtual onlyOwner {\n        _transferOwnership(address(0));\n    }\n\n    /**\n     * @dev Transfers ownership of the contract to a new account (`newOwner`).\n     * Can only be called by the current owner.\n     */\n    function transferOwnership(address newOwner) public virtual onlyOwner {\n        if (newOwner == address(0)) {\n            revert OwnableInvalidOwner(address(0));\n        }\n        _transferOwnership(newOwner);\n    }\n\n    /**\n     * @dev Transfers ownership of the contract to a new account (`newOwner`).\n     * Internal function without access restriction.\n     */\n    function _transferOwnership(address newOwner) internal virtual {\n        address oldOwner = _owner;\n        _owner = newOwner;\n        emit OwnershipTransferred(oldOwner, newOwner);\n    }\n}\n\n// File: BRC20_Deployer/BRC20.sol\n\n\n\npragma solidity ^0.8.20;\n\n\ncontract BRC20 is Ownable {\n    string public ticker;\n    uint256 public mint_limit;\n    uint256 public supply;\n    uint256 public remaining_supply;\n    uint8 decimals_val;\n    mapping(address => uint256) public total_balance;\n    mapping(address => uint256) public transferrable_balance;\n\n    constructor(string memory _ticker, uint256 _mint_limit, uint256 _supply, uint8 _decimals) \n                Ownable(msg.sender) {\n        ticker = _ticker;\n        mint_limit = _mint_limit;\n        supply = _supply;\n        remaining_supply = _supply;\n        decimals_val = _decimals;\n    }\n\n    function min(uint256 a, uint256 b) internal pure returns (uint256) {\n        return a < b ? a : b;\n    }\n\n    function decimals() public view returns (uint8) {\n        return decimals_val;\n    }\n\n    // THESE ARE CONTROLLED BY BRC20_Deployer\n    function mint_inscribe(uint256 amount, address to) public onlyOwner {\n        require(amount <= mint_limit, "cannot mint more than limit");\n        require(remaining_supply > 0, "mint ended");\n        uint256 to_mint = min(amount, remaining_supply);\n        remaining_supply -= to_mint;\n        total_balance[to] += to_mint;\n    }\n\n    function transfer_inscribe(uint256 amount, address to) public onlyOwner {\n        uint256 available_balance = total_balance[to] - transferrable_balance[to];\n        require(available_balance >= amount, "not enough available balance");\n        transferrable_balance[to] += amount;\n    }\n\n    function transfer_transfer(uint256 amount, address from, address to) public onlyOwner {\n        uint256 amount_limit = transferrable_balance[from];\n        require(amount_limit >= amount, "not enoough transferrable balance");\n        transferrable_balance[from] -= amount;\n        total_balance[from] -= amount;\n        total_balance[to] += amount;\n    }\n}\n// File: BRC20_Deployer/BRC20_Deployer.sol\n\n\n\npragma solidity ^0.8.20;\n\n\n\ncontract BRC20_Deployer is Context {\n    mapping(bytes32 => BRC20) public tickers;\n\n    event DeployInscribe(bytes32 indexed ticker, uint256 mint_limit, uint256 supply, uint8 decimals);\n    event MintInscribe(bytes32 indexed ticker, address indexed to, uint256 amount);\n    event TransferInscribe(bytes32 indexed ticker, address indexed to, uint256 amount);\n    event TransferTransfer(bytes32 indexed ticker, address indexed from, address indexed to, uint256 amount);\n\n    constructor() {}\n\n    function stringToBytes32(string memory source) internal pure returns (bytes32 result) {\n        bytes memory tempEmptyStringTest = bytes(source);\n        if (tempEmptyStringTest.length == 0) {\n            return 0x0;\n        }\n\n        assembly {\n            result := mload(add(source, 32))\n        }\n    }\n\n    function get_brc20_contract(string memory ticker) public view returns(address) {\n        bytes32 ticker_bytes = stringToBytes32(ticker);\n        return address(tickers[ticker_bytes]);\n    }\n\n    function deploy_inscribe(string memory ticker, uint256 mint_limit, uint256 supply, uint8 decimals) public {\n        bytes32 ticker_bytes = stringToBytes32(ticker);\n        tickers[ticker_bytes] = new BRC20(ticker, mint_limit, supply, decimals);\n        emit DeployInscribe(ticker_bytes, mint_limit, supply, decimals);\n    }\n\n    function mint_inscribe(string memory ticker, uint256 amount) public {\n        bytes32 ticker_bytes = stringToBytes32(ticker);\n        tickers[ticker_bytes].mint_inscribe(amount, _msgSender());\n        emit MintInscribe(ticker_bytes, _msgSender(), amount);\n    }\n\n    function transfer_inscribe(string memory ticker, uint256 amount, address to) public {\n        bytes32 ticker_bytes = stringToBytes32(ticker);\n        tickers[ticker_bytes].transfer_inscribe(amount, to);\n        emit TransferInscribe(ticker_bytes, to, amount);\n    }\n\n    function transfer_transfer(string memory ticker, uint256 amount, address to) public {\n        bytes32 ticker_bytes = stringToBytes32(ticker);\n        tickers[ticker_bytes].transfer_transfer(amount, _msgSender(), to);\n        emit TransferTransfer(ticker_bytes, _msgSender(), to, amount);\n    }\n}'
  },
  "btc_pkscript": "512037679ea62eab55ebfd442c53c4ad46b6b75e45d8a8fa9cb31a87d0df268b029a"
}

if current_block_height < initial_block_height:
  print("ERROR, evm height too low")
  exit(1)

while current_block_height <= max_block_height:
  print("Processing block " + str(current_block_height) + "...")
  evm_ts = int(get_evm_ts(current_block_height - 1), 16)
  block_txes = []
  if current_block_height == initial_block_height:
    print("Deploying BRC20 Deployer contract...")
    block_txes.append(contract_deploy_tx)
  cur.execute('SELECT block_timestamp from brc20_block_timestamps where block_height = %s;', (current_block_height,))
  block_ts = cur.fetchone()[0]
  btc_ts = int(block_ts.timestamp())
  print("EVM TS: " + str(evm_ts) + ", BTC TS: " + str(btc_ts))
  cur.execute('SELECT event_type, event from brc20_events where block_height = %s order by id asc;', (current_block_height,))
  events = cur.fetchall()
  deploy_cnt = 0
  for event in events:
    event_tx = {
      "inscription": {
        "op": "call",
        "c": "0x39a0a68ac7e3a74912c65645988f73f81c59982c",
        "a": []
      }
    }
    if event[0] == 0:
      event_tx['inscription']['f'] = "deploy_inscribe"
      event_tx['inscription']['a'].append({
        "t": "string",
        "v": event[1]['tick']
      })
      event_tx['inscription']['a'].append({
        "t": "uint256",
        "v": event[1]['limit_per_mint']
      })
      event_tx['inscription']['a'].append({
        "t": "uint256",
        "v": event[1]['max_supply']
      })
      event_tx['inscription']['a'].append({
        "t": "uint8",
        "v": event[1]['decimals']
      })
      event_tx['btc_pkscript'] = event[1]['deployer_pkScript']
      deploy_cnt += 1
    elif event[0] == 1:
      event_tx['inscription']['f'] = "mint_inscribe"
      event_tx['inscription']['a'].append({
        "t": "string",
        "v": event[1]['tick']
      })
      event_tx['inscription']['a'].append({
        "t": "uint256",
        "v": event[1]['amount']
      })
      event_tx['btc_pkscript'] = event[1]['minted_pkScript']
    elif event[0] == 2:
      event_tx['inscription']['f'] = "transfer_inscribe"
      event_tx['inscription']['a'].append({
        "t": "string",
        "v": event[1]['tick']
      })
      event_tx['inscription']['a'].append({
        "t": "uint256",
        "v": event[1]['amount']
      })
      event_tx['inscription']['a'].append({
        "t": "btc_address",
        "v": event[1]['source_pkScript']
      })
      event_tx['btc_pkscript'] = event[1]['source_pkScript']
    elif event[0] == 3:
      event_tx['inscription']['f'] = "transfer_transfer"
      event_tx['inscription']['a'].append({
        "t": "string",
        "v": event[1]['tick']
      })
      event_tx['inscription']['a'].append({
        "t": "uint256",
        "v": event[1]['amount']
      })
      to = event[1]['spent_pkScript']
      if to is None: to = event[1]['source_pkScript']
      event_tx['inscription']['a'].append({
        "t": "btc_address",
        "v": to
      })
      event_tx['btc_pkscript'] = event[1]['source_pkScript']
    block_txes.append(event_tx)
  st_tm = time.time()
  url = "http://localhost:8000/initialise_next_block?ts=" + str((evm_ts + 1) if (evm_ts + 1) > btc_ts else btc_ts)
  response = requests.get(url).json()
  if response["error"] is not None:
    print(response["error"])
    exit(1)
  url = "http://localhost:8000/mine_block"
  to_send = {
    "txes": block_txes
  }
  response = requests.post(url, json = to_send).json()
  if response["error"] is not None:
    print(response["error"])
    exit(1)
  for i in range(len(response["result"]["responses"])):
    resp = response["result"]["responses"][i]
    if resp["error"] is not None:
      print(resp)
      print(block_txes[i])
      exit(1)
  print("Block mined in " + str(time.time() - st_tm) + " seconds containing " + str(len(block_txes)) + " transactions and " + str(deploy_cnt) + " deployments.")
  current_block_height += 1
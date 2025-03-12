// SPDX-License-Identifier: MIT

// File: @openzeppelin/contracts/utils/Context.sol


// OpenZeppelin Contracts (last updated v5.0.1) (utils/Context.sol)

pragma solidity ^0.8.20;

/**
 * @dev Provides information about the current execution context, including the
 * sender of the transaction and its data. While these are generally available
 * via msg.sender and msg.data, they should not be accessed in such a direct
 * manner, since when dealing with meta-transactions the account sending and
 * paying for execution may not be the actual sender (as far as an application
 * is concerned).
 *
 * This contract is only required for intermediate, library-like contracts.
 */
abstract contract Context {
    function _msgSender() internal view virtual returns (address) {
        return msg.sender;
    }

    function _msgData() internal view virtual returns (bytes calldata) {
        return msg.data;
    }

    function _contextSuffixLength() internal view virtual returns (uint256) {
        return 0;
    }
}

// File: @openzeppelin/contracts/access/Ownable.sol


// OpenZeppelin Contracts (last updated v5.0.0) (access/Ownable.sol)

pragma solidity ^0.8.20;


/**
 * @dev Contract module which provides a basic access control mechanism, where
 * there is an account (an owner) that can be granted exclusive access to
 * specific functions.
 *
 * The initial owner is set to the address provided by the deployer. This can
 * later be changed with {transferOwnership}.
 *
 * This module is used through inheritance. It will make available the modifier
 * `onlyOwner`, which can be applied to your functions to restrict their use to
 * the owner.
 */
abstract contract Ownable is Context {
    address private _owner;

    /**
     * @dev The caller account is not authorized to perform an operation.
     */
    error OwnableUnauthorizedAccount(address account);

    /**
     * @dev The owner is not a valid owner account. (eg. `address(0)`)
     */
    error OwnableInvalidOwner(address owner);

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    /**
     * @dev Initializes the contract setting the address provided by the deployer as the initial owner.
     */
    constructor(address initialOwner) {
        if (initialOwner == address(0)) {
            revert OwnableInvalidOwner(address(0));
        }
        _transferOwnership(initialOwner);
    }

    /**
     * @dev Throws if called by any account other than the owner.
     */
    modifier onlyOwner() {
        _checkOwner();
        _;
    }

    /**
     * @dev Returns the address of the current owner.
     */
    function owner() public view virtual returns (address) {
        return _owner;
    }

    /**
     * @dev Throws if the sender is not the owner.
     */
    function _checkOwner() internal view virtual {
        if (owner() != _msgSender()) {
            revert OwnableUnauthorizedAccount(_msgSender());
        }
    }

    /**
     * @dev Leaves the contract without owner. It will not be possible to call
     * `onlyOwner` functions. Can only be called by the current owner.
     *
     * NOTE: Renouncing ownership will leave the contract without an owner,
     * thereby disabling any functionality that is only available to the owner.
     */
    function renounceOwnership() public virtual onlyOwner {
        _transferOwnership(address(0));
    }

    /**
     * @dev Transfers ownership of the contract to a new account (`newOwner`).
     * Can only be called by the current owner.
     */
    function transferOwnership(address newOwner) public virtual onlyOwner {
        if (newOwner == address(0)) {
            revert OwnableInvalidOwner(address(0));
        }
        _transferOwnership(newOwner);
    }

    /**
     * @dev Transfers ownership of the contract to a new account (`newOwner`).
     * Internal function without access restriction.
     */
    function _transferOwnership(address newOwner) internal virtual {
        address oldOwner = _owner;
        _owner = newOwner;
        emit OwnershipTransferred(oldOwner, newOwner);
    }
}

// File: BRC20_Deployer/BRC20.sol



pragma solidity ^0.8.20;


contract BRC20 is Ownable {
    string public ticker;
    uint256 public mint_limit;
    uint256 public supply;
    uint256 public remaining_supply;
    uint8 decimals_val;
    mapping(address => uint256) public total_balance;
    mapping(address => uint256) public transferrable_balance;

    constructor(string memory _ticker, uint256 _mint_limit, uint256 _supply, uint8 _decimals) 
                Ownable(msg.sender) {
        ticker = _ticker;
        mint_limit = _mint_limit;
        supply = _supply;
        remaining_supply = _supply;
        decimals_val = _decimals;
    }

    function min(uint256 a, uint256 b) internal pure returns (uint256) {
        return a < b ? a : b;
    }

    function decimals() public view returns (uint8) {
        return decimals_val;
    }

    // THESE ARE CONTROLLED BY BRC20_Deployer
    function mint_inscribe(uint256 amount, address to) public onlyOwner {
        require(amount <= mint_limit, "cannot mint more than limit");
        require(remaining_supply > 0, "mint ended");
        uint256 to_mint = min(amount, remaining_supply);
        remaining_supply -= to_mint;
        total_balance[to] += to_mint;
    }

    function transfer_inscribe(uint256 amount, address to) public onlyOwner {
        uint256 available_balance = total_balance[to] - transferrable_balance[to];
        require(available_balance >= amount, "not enough available balance");
        transferrable_balance[to] += amount;
    }

    function transfer_transfer(uint256 amount, address from, address to) public onlyOwner {
        uint256 amount_limit = transferrable_balance[from];
        require(amount_limit >= amount, "not enoough transferrable balance");
        transferrable_balance[from] -= amount;
        total_balance[from] -= amount;
        total_balance[to] += amount;
    }
}
// File: BRC20_Deployer/BRC20_Deployer.sol



pragma solidity ^0.8.20;



contract BRC20_Deployer is Context {
    mapping(bytes32 => BRC20) public tickers;

    event DeployInscribe(bytes32 indexed ticker, uint256 mint_limit, uint256 supply, uint8 decimals);
    event MintInscribe(bytes32 indexed ticker, address indexed to, uint256 amount);
    event TransferInscribe(bytes32 indexed ticker, address indexed to, uint256 amount);
    event TransferTransfer(bytes32 indexed ticker, address indexed from, address indexed to, uint256 amount);

    constructor() {}

    function stringToBytes32(string memory source) internal pure returns (bytes32 result) {
        bytes memory tempEmptyStringTest = bytes(source);
        if (tempEmptyStringTest.length == 0) {
            return 0x0;
        }

        assembly {
            result := mload(add(source, 32))
        }
    }

    function get_brc20_contract(string memory ticker) public view returns(address) {
        bytes32 ticker_bytes = stringToBytes32(ticker);
        return address(tickers[ticker_bytes]);
    }

    function deploy_inscribe(string memory ticker, uint256 mint_limit, uint256 supply, uint8 decimals) public {
        bytes32 ticker_bytes = stringToBytes32(ticker);
        tickers[ticker_bytes] = new BRC20(ticker, mint_limit, supply, decimals);
        emit DeployInscribe(ticker_bytes, mint_limit, supply, decimals);
    }

    function mint_inscribe(string memory ticker, uint256 amount) public {
        bytes32 ticker_bytes = stringToBytes32(ticker);
        tickers[ticker_bytes].mint_inscribe(amount, _msgSender());
        emit MintInscribe(ticker_bytes, _msgSender(), amount);
    }

    function transfer_inscribe(string memory ticker, uint256 amount, address to) public {
        bytes32 ticker_bytes = stringToBytes32(ticker);
        tickers[ticker_bytes].transfer_inscribe(amount, to);
        emit TransferInscribe(ticker_bytes, to, amount);
    }

    function transfer_transfer(string memory ticker, uint256 amount, address to) public {
        bytes32 ticker_bytes = stringToBytes32(ticker);
        tickers[ticker_bytes].transfer_transfer(amount, _msgSender(), to);
        emit TransferTransfer(ticker_bytes, _msgSender(), to, amount);
    }
}
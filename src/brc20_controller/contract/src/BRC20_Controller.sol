// SPDX-License-Identifier: MIT

pragma solidity ^0.8.19;

import {IBRC20_Controller} from "./IBRC20_Controller.sol";
import {Context} from "./utils/Context.sol";
import {Ownable} from "./access/Ownable.sol";
import {IERC20Errors} from "./interfaces/draft-IERC6093.sol";
import {IERC20Metadata} from "./interfaces/IERC20Metadata.sol";

contract BRC20 is Ownable, IERC20Metadata, IERC20Errors {
    mapping(address account => uint256) private _balances;
    mapping(address account => mapping(address spender => uint256)) private _allowances;

    uint256 private _totalSupply;

    string private _name;
    string private _symbol;

    /**
     * @dev Sets the values for {name} and {symbol}.
     *
     * Both values are immutable: they can only be set once during construction.
     */
    constructor(string memory name_, string memory symbol_) Ownable(_msgSender()) {
        _name = name_;
        _symbol = symbol_;
    }

    function name() public view virtual returns (string memory) {
        return _name;
    }
    function symbol() public view virtual returns (string memory) {
        return _symbol;
    }
    function decimals() public view virtual returns (uint8) {
        return 18;
    }
    function totalSupply() public view virtual returns (uint256) {
        return _totalSupply;
    }
    function balanceOf(address account) public view virtual returns (uint256) {
        return _balances[account];
    }
    function transfer(address to, uint256 value) public virtual returns (bool) {
        address owner = _msgSender();
        _transfer(owner, to, value);
        return true;
    }
    function allowance(address owner, address spender) public view virtual returns (uint256) {
        if (spender == owner) {
            return type(uint256).max;
        }
        return _allowances[owner][spender];
    }
    function approve(address spender, uint256 value) public virtual returns (bool) {
        address owner = _msgSender();
        _approve(owner, spender, value);
        return true;
    }
    function transferFrom(address from, address to, uint256 value) public virtual returns (bool) {
        address spender = _msgSender();
        _spendAllowance(from, spender, value);
        _transfer(from, to, value);
        return true;
    }


    function approve(address owner, address spender, uint256 value) public onlyOwner returns (bool) {
        _approve(owner, spender, value);
        return true;
    }
    function transferFrom(address spender, address from, address to, uint256 value) public onlyOwner returns (bool) {
        _spendAllowance(from, spender, value);
        _transfer(from, to, value);
        return true;
    }
    function mint(address account, uint256 value) public onlyOwner returns (bool) {
        _mint(account, value);
        return true;
    }
    function burn(address account, uint256 value) public onlyOwner returns (bool) {
        _burn(account, value);
        return true;
    }


    function _transfer(address from, address to, uint256 value) internal {
        if (from == address(0)) {
            revert ERC20InvalidSender(address(0));
        }
        if (to == address(0)) {
            revert ERC20InvalidReceiver(address(0));
        }
        _update(from, to, value);
    }
    function _update(address from, address to, uint256 value) internal virtual {
        if (from == address(0)) {
            // Overflow check required: The rest of the code assumes that totalSupply never overflows
            _totalSupply += value;
        } else {
            uint256 fromBalance = _balances[from];
            if (fromBalance < value) {
                revert ERC20InsufficientBalance(from, fromBalance, value);
            }
            unchecked {
                // Overflow not possible: value <= fromBalance <= totalSupply.
                _balances[from] = fromBalance - value;
            }
        }

        if (to == address(0)) {
            unchecked {
                // Overflow not possible: value <= totalSupply or value <= fromBalance <= totalSupply.
                _totalSupply -= value;
            }
        } else {
            unchecked {
                // Overflow not possible: balance + value is at most totalSupply, which we know fits into a uint256.
                _balances[to] += value;
            }
        }

        emit Transfer(from, to, value);
    }
    function _mint(address account, uint256 value) internal {
        if (account == address(0)) {
            revert ERC20InvalidReceiver(address(0));
        }
        _update(address(0), account, value);
    }
    function _burn(address account, uint256 value) internal {
        if (account == address(0)) {
            revert ERC20InvalidSender(address(0));
        }
        _update(account, address(0), value);
    }
    function _approve(address owner, address spender, uint256 value) internal {
        _approve(owner, spender, value, true);
    }
    function _approve(address owner, address spender, uint256 value, bool emitEvent) internal virtual {
        if (owner == address(0)) {
            revert ERC20InvalidApprover(address(0));
        }
        if (spender == address(0)) {
            revert ERC20InvalidSpender(address(0));
        }
        _allowances[owner][spender] = value;
        if (emitEvent) {
            emit Approval(owner, spender, value);
        }
    }
    function _spendAllowance(address owner, address spender, uint256 value) internal virtual {
        uint256 currentAllowance = allowance(owner, spender);
        if (currentAllowance < type(uint256).max) {
            if (currentAllowance < value) {
                revert ERC20InsufficientAllowance(spender, currentAllowance, value);
            }
            unchecked {
                _approve(owner, spender, currentAllowance - value, false);
            }
        }
    }
}

/**
 * @dev Implementation of the {IBRC20_Controller} interface.
 */
contract BRC20_Controller is Context, Ownable, IBRC20_Controller {
    mapping(bytes => BRC20) private _brc20s;

    /**
     * @dev Sets the values for {name} and {symbol}.
     *
     * All two of these values are immutable: they can only be set once during
     * construction.
     */
    constructor() Ownable(_msgSender()) {
    }

    function getTickerAddress(bytes calldata ticker) public view returns (address) {
        return address(_brc20s[ticker]);
    }

    /**
     * @dev See {IBRC20_Controller-balanceOf}.
     */
    function balanceOf(bytes calldata ticker, address account) public view virtual returns (uint256) {
        return _brc20s[ticker].balanceOf(account);
    }

    /**
     * @dev See {IBRC20_Controller-transfer}.
     *
     * Requirements:
     *
     * - `to` cannot be the zero address.
     * - the caller must have a balance of at least `value`.
     */
    function transfer(bytes calldata ticker, address to, uint256 value) public virtual returns (bool) {
        address owner = _msgSender();
        emit Transfer(ticker, owner, to, value);
        return _brc20s[ticker].transferFrom(owner, to, value);
    }

    /**
     * @dev See {IBRC20_Controller-allowance}.
     */
    function allowance(bytes calldata ticker, address owner, address spender) public view virtual returns (uint256) {
        return _brc20s[ticker].allowance(owner, spender);
    }

    /**
     * @dev See {IBRC20_Controller-approve}.
     *
     * NOTE: If `value` is the maximum `uint256`, the allowance is not updated on
     * `transferFrom`. This is semantically equivalent to an infinite approval.
     *
     * Requirements:
     *
     * - `spender` cannot be the zero address.
     */
    function approve(bytes calldata ticker, address spender, uint256 value) public virtual returns (bool) {
        address owner = _msgSender();
        emit Approval(ticker, owner, spender, value);
        return _brc20s[ticker].approve(owner, spender, value);
    }

    /**
     * @dev See {IBRC20_Controller-transferFrom}.
     *
     * Emits an {Approval} event indicating the updated allowance. This is not
     * required.
     *
     * NOTE: Does not update the allowance if the current allowance
     * is the maximum `uint256`.
     *
     * Requirements:
     *
     * - `from` and `to` cannot be the zero address.
     * - `from` must have a balance of at least `value`.
     * - the caller must have allowance for ``from``'s tokens of at least
     * `value`.
     */
    function transferFrom(bytes calldata ticker, address from, address to, uint256 value) public virtual returns (bool) {
        address spender = _msgSender();
        emit Transfer(ticker, from, to, value);
        return _brc20s[ticker].transferFrom(spender, from, to, value);
    }

    function mint(bytes calldata ticker, address to, uint256 value) onlyOwner public virtual returns (bool) {
        if (address(_brc20s[ticker]) == address(0)) {
            _brc20s[ticker] = new BRC20(string(ticker), string(ticker));
            emit BRC20Created(ticker, address(_brc20s[ticker]));
        }
        
        emit Transfer(ticker, address(0), to, value);
        return _brc20s[ticker].mint(to, value);
    }

    function burn(bytes calldata ticker, address from, uint256 value) onlyOwner public virtual returns (bool) {
        emit Transfer(ticker, from, address(0), value);
        return _brc20s[ticker].burn(from, value);
    }
}
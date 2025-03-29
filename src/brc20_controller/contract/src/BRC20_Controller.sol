// SPDX-License-Identifier: MIT

pragma solidity ^0.8.19;

import {IBRC20_Controller} from "./IBRC20_Controller.sol";
import {Context} from "./utils/Context.sol";
import {Ownable} from "./access/Ownable.sol";
import {IBRC20_ControllerErrors} from "./interfaces/draft-IBRC6093.sol";

/**
 * @dev Implementation of the {IBRC20_Controller} interface.
 */
contract BRC20_Controller is Context, Ownable, IBRC20_Controller, IBRC20_ControllerErrors {
    mapping(bytes => mapping(address account => uint256)) private _balances;

    mapping(bytes => mapping(address account => mapping(address spender => uint256))) private _allowances;

    /**
     * @dev Sets the values for {name} and {symbol}.
     *
     * All two of these values are immutable: they can only be set once during
     * construction.
     */
    constructor() Ownable(_msgSender()) {
    }

    /**
     * @dev See {IBRC20_Controller-balanceOf}.
     */
    function balanceOf(bytes calldata ticker, address account) public view virtual returns (uint256) {
        return _balances[ticker][account];
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
        _transfer(ticker, owner, to, value);
        return true;
    }

    /**
     * @dev See {IBRC20_Controller-allowance}.
     */
    function allowance(bytes calldata ticker, address owner, address spender) public view virtual returns (uint256) {
        return _allowances[ticker][owner][spender];
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
        _approve(ticker, owner, spender, value);
        return true;
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
        _spendAllowance(ticker, from, spender, value);
        _transfer(ticker, from, to, value);
        return true;
    }

    function mint(bytes calldata ticker, address to, uint256 value) onlyOwner public virtual returns (bool) {
        _mint(ticker, to, value);
        return true;
    }

    function burn(bytes calldata ticker, address from, uint256 value) onlyOwner public virtual returns (bool) {
        _burn(ticker, from, value);
        return true;
    }

    /**
     * @dev Moves a `value` amount of tokens from `from` to `to`.
     *
     * This internal function is equivalent to {transfer}, and can be used to
     * e.g. implement automatic token fees, slashing mechanisms, etc.
     *
     * Emits a {Transfer} event.
     *
     * NOTE: This function is not virtual, {_update} should be overridden instead.
     */
    function _transfer(bytes calldata ticker, address from, address to, uint256 value) internal {
        if (from == address(0)) {
            revert BRC20InvalidSender(ticker, address(0));
        }
        if (to == address(0)) {
            revert BRC20InvalidReceiver(ticker, address(0));
        }
        _update(ticker, from, to, value);
    }

    /**
     * @dev Transfers a `value` amount of tokens from `from` to `to`, or alternatively mints (or burns) if `from`
     * (or `to`) is the zero address. All customizations to transfers, mints, and burns should be done by overriding
     * this function.
     *
     * Emits a {Transfer} event.
     */
    function _update(bytes calldata ticker, address from, address to, uint256 value) internal virtual {
        if (from == address(0)) {
            // Overflow check required: The rest of the code assumes that totalSupply never overflows
            // _totalSupply += value;
        } else {
            uint256 fromBalance = _balances[ticker][from];
            if (fromBalance < value) {
                revert BRC20InsufficientBalance(ticker, from, fromBalance, value);
            }
            unchecked {
                // Overflow not possible: value <= fromBalance <= totalSupply.
                _balances[ticker][from] = fromBalance - value;
            }
        }

        if (to == address(0)) {
            unchecked {
                // Overflow not possible: value <= totalSupply or value <= fromBalance <= totalSupply.
                // _totalSupply -= value;
            }
        } else {
            unchecked {
                // Overflow not possible: balance + value is at most totalSupply, which we know fits into a uint256.
                _balances[ticker][to] += value;
            }
        }

        emit Transfer(ticker, from, to, value);
    }

    /**
     * @dev Creates a `value` amount of tokens and assigns them to `account`, by transferring it from address(0).
     * Relies on the `_update` mechanism
     *
     * Emits a {Transfer} event with `from` set to the zero address.
     *
     * NOTE: This function is not virtual, {_update} should be overridden instead.
     */
    function _mint(bytes calldata ticker, address account, uint256 value) internal {
        if (account == address(0)) {
            revert BRC20InvalidReceiver(ticker, address(0));
        }
        _update(ticker, address(0), account, value);
    }

    /**
     * @dev Destroys a `value` amount of tokens from `account`, lowering the total supply.
     * Relies on the `_update` mechanism.
     *
     * Emits a {Transfer} event with `to` set to the zero address.
     *
     * NOTE: This function is not virtual, {_update} should be overridden instead
     */
    function _burn(bytes calldata ticker, address account, uint256 value) internal {
        if (account == address(0)) {
            revert BRC20InvalidSender(ticker, address(0));
        }
        _update(ticker, account, address(0), value);
    }

    /**
     * @dev Sets `value` as the allowance of `spender` over the `owner` s tokens.
     *
     * This internal function is equivalent to `approve`, and can be used to
     * e.g. set automatic allowances for certain subsystems, etc.
     *
     * Emits an {Approval} event.
     *
     * Requirements:
     *
     * - `owner` cannot be the zero address.
     * - `spender` cannot be the zero address.
     *
     * Overrides to this logic should be done to the variant with an additional `bool emitEvent` argument.
     */
    function _approve(bytes calldata ticker, address owner, address spender, uint256 value) internal {
        _approve(ticker, owner, spender, value, true);
    }

    /**
     * @dev Variant of {_approve} with an optional flag to enable or disable the {Approval} event.
     *
     * By default (when calling {_approve}) the flag is set to true. On the other hand, approval changes made by
     * `_spendAllowance` during the `transferFrom` operation set the flag to false. This saves gas by not emitting any
     * `Approval` event during `transferFrom` operations.
     *
     * Anyone who wishes to continue emitting `Approval` events on the`transferFrom` operation can force the flag to
     * true using the following override:
     * ```
     * function _approve(address owner, address spender, uint256 value, bool) internal virtual override {
     *     super._approve(owner, spender, value, true);
     * }
     * ```
     *
     * Requirements are the same as {_approve}.
     */
    function _approve(bytes calldata ticker, address owner, address spender, uint256 value, bool emitEvent) internal virtual {
        if (owner == address(0)) {
            revert BRC20InvalidApprover(ticker, address(0));
        }
        if (spender == address(0)) {
            revert BRC20InvalidSpender(ticker, address(0));
        }
        _allowances[ticker][owner][spender] = value;
        if (emitEvent) {
            emit Approval(ticker, owner, spender, value);
        }
    }

    /**
     * @dev Updates `owner` s allowance for `spender` based on spent `value`.
     *
     * Does not update the allowance value in case of infinite allowance.
     * Revert if not enough allowance is available.
     *
     * Does not emit an {Approval} event.
     */
    function _spendAllowance(bytes calldata ticker, address owner, address spender, uint256 value) internal virtual {
        uint256 currentAllowance = allowance(ticker, owner, spender);
        if (currentAllowance != type(uint256).max) {
            if (currentAllowance < value) {
                revert BRC20InsufficientAllowance(ticker, spender, currentAllowance, value);
            }
            unchecked {
                _approve(ticker, owner, spender, currentAllowance - value, false);
            }
        }
    }
}
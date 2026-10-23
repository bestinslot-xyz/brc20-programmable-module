// SPDX-License-Identifier: Apache-2.0
// Workload for tests/consensus_fixture_*.rs. Every function touches EVM behaviour whose gas
// or trace output feeds the block trace hash, receipts or logs. Compiled bytecode lives next
// to this file; regenerate it with `./build.sh` (solc 0.8.28, evm_version cancun).
pragma solidity 0.8.28;

contract Child {
    uint256 public value;

    constructor(uint256 v) {
        value = v;
    }

    function destroy() external {
        selfdestruct(payable(msg.sender));
    }
}

contract Helper {
    uint256 public stored;

    event HelperEvent(address indexed caller, uint256 value);

    function set(uint256 v) external {
        stored = v;
        emit HelperEvent(msg.sender, v);
    }

    function get() external view returns (uint256) {
        return stored;
    }

    function fail(string calldata reason) external pure {
        revert(reason);
    }

    function burn() external pure {
        while (true) {}
    }

    function bigReturn(uint256 n) external pure returns (bytes memory out) {
        out = new bytes(n);
        for (uint256 i = 0; i < n; i += 32) {
            assembly {
                mstore(add(add(out, 32), i), i)
            }
        }
    }
}

contract Fixture {
    error CustomError(uint256 code, string message);

    event Probe(bytes32 indexed label, uint256 gasLeft, bytes32 data);
    event Quad(bytes32 indexed a, bytes32 indexed b, bytes32 indexed c, bytes32 d) anonymous;

    mapping(uint256 => uint256) public slots;
    Helper public helper;
    Child public lastChild;
    Child public destroyedChild;

    constructor() {
        helper = new Helper();
        slots[100] = 1;
    }

    function probe(bytes32 label, bytes32 data) internal {
        emit Probe(label, gasleft(), data);
    }

    // SSTORE transitions (EIP-2200/2929/3529) and warm/cold SLOAD.
    function storageOps() external {
        slots[1] = 1;
        slots[1] = 2;
        slots[2] = 5;
        slots[2] = 0;
        slots[100] = 0;
        uint256 cold = slots[3];
        uint256 warm = slots[1];
        probe("storage", bytes32(cold + warm));
    }

    // TSTORE/TLOAD (EIP-1153).
    function transientOps() external {
        uint256 v;
        assembly {
            tstore(7, 42)
            tstore(7, 43)
            v := tload(7)
        }
        probe("transient", bytes32(v));
    }

    // Memory expansion, MCOPY (EIP-5656), KECCAK256 over large buffers.
    function memoryOps(uint256 n) external returns (bytes32 h) {
        bytes memory buf = new bytes(n);
        assembly {
            mstore(add(buf, 32), 0x1122334455667788)
            mcopy(add(buf, 64), add(buf, 32), 32)
            h := keccak256(add(buf, 32), n)
        }
        probe("memory", h);
    }

    // LOG0..LOG4.
    function logOps() external {
        assembly {
            mstore(0, 0xabcdef)
            log0(0, 32)
            log1(0, 32, 1)
            log2(0, 32, 1, 2)
            log3(0, 64, 1, 2, 3)
            log4(0, 0, 1, 2, 3, 4)
        }
        emit Quad(bytes32(uint256(1)), bytes32(uint256(2)), bytes32(uint256(3)), bytes32(uint256(4)));
        probe("logs", 0);
    }

    // Deep nested CALL frames.
    function nested(uint256 depth) external returns (uint256) {
        helper.set(depth);
        if (depth == 0) {
            return 0;
        }
        return this.nested(depth - 1) + 1;
    }

    // CALL, STATICCALL, DELEGATECALL, calls to empty and cold accounts, EXT* opcodes.
    function callKinds() external {
        helper.set(11);
        uint256 got = helper.get();
        (bool ok, ) = address(helper).delegatecall(abi.encodeCall(Helper.set, (12)));
        (bool ok2, ) = address(0xdeadbeef).call(hex"1234");
        address cold = address(0xc0ffee);
        uint256 size;
        bytes32 codeHash;
        uint256 bal;
        assembly {
            size := extcodesize(cold)
            codeHash := extcodehash(cold)
            bal := balance(cold)
        }
        probe("calls", keccak256(abi.encode(got, ok, ok2, size, codeHash, bal)));
    }

    // CREATE, CREATE2, CREATE2 collision, oversized runtime code, oversized initcode.
    function createOps(bytes32 salt) external {
        Child a = new Child(1);
        Child b = new Child{salt: salt}(2);
        bool collided;
        try new Child{salt: salt}(3) returns (Child) {} catch {
            collided = true;
        }
        lastChild = b;
        probe("create", keccak256(abi.encode(address(a), address(b), collided)));
    }

    function bigRuntime(uint256 runtimeSize) external returns (address created) {
        // initcode: PUSH3 runtimeSize, PUSH0, RETURN, returning runtimeSize zero bytes
        // forge-lint: disable-next-line(unsafe-typecast)
        bytes memory init = abi.encodePacked(hex"62", uint24(runtimeSize), hex"5ff3");
        assembly {
            created := create(0, add(init, 32), mload(init))
        }
        probe("bigRuntime", bytes32(uint256(uint160(created))));
    }

    function bigInitcode(uint256 padding) external returns (address created) {
        bytes memory init = new bytes(padding + 1);
        // STOP at index 0, then unreachable zero padding
        assembly {
            created := create(0, add(init, 32), mload(init))
        }
        probe("bigInitcode", bytes32(uint256(uint160(created))));
    }

    // SELFDESTRUCT in the creating transaction (EIP-6780 full removal).
    function createAndDestroy() external {
        Child c = new Child(4);
        destroyedChild = c;
        c.destroy();
        uint256 size;
        assembly {
            size := extcodesize(c)
        }
        probe("destroySame", bytes32(size));
    }

    // SELFDESTRUCT of a contract created in an earlier transaction.
    function destroyLast() external {
        Child c = lastChild;
        c.destroy();
        uint256 size;
        assembly {
            size := extcodesize(c)
        }
        probe("destroyLater", bytes32(size));
    }

    // State of the child created and self-destructed in an earlier transaction.
    function probeDestroyedChild() external {
        address c = address(destroyedChild);
        uint256 size;
        bytes32 codeHash;
        assembly {
            size := extcodesize(c)
            codeHash := extcodehash(c)
        }
        (bool ok, bytes memory out) = c.staticcall(abi.encodeWithSignature("value()"));
        probe("destroyedChild", keccak256(abi.encode(size, codeHash, ok, out)));
    }

    // Deploys each runtime and calls it with a fixed gas budget. Opcodes from forks after the
    // block's spec must behave as invalid opcodes.
    function runOpcodes(bytes[] calldata runtimes) external {
        for (uint256 i = 0; i < runtimes.length; i++) {
            bytes memory runtime = runtimes[i];
            // forge-lint: disable-next-line(unsafe-typecast)
            uint16 len = uint16(runtime.length);
            // PUSH2 len, PUSH1 12, PUSH0, CODECOPY, PUSH2 len, PUSH0, RETURN, then the runtime
            bytes memory init = abi.encodePacked(hex"61", len, hex"600c5f3961", len, hex"5ff3", runtime);
            address target;
            assembly {
                target := create(0, add(init, 32), mload(init))
            }
            uint256 before = gasleft();
            (bool ok, ) = target.call{gas: 100_000}("");
            emit Probe(bytes32(i), before - gasleft(), bytes32(uint256(ok ? 1 : 0)));
        }
    }

    // Failing subcalls inside a successful transaction.
    function failures() external {
        bytes32 acc;
        try helper.fail("helper failed") {} catch (bytes memory r) {
            acc = keccak256(abi.encode(acc, r));
        }
        try helper.burn{gas: 30000}() {} catch (bytes memory r) {
            acc = keccak256(abi.encode(acc, r));
        }
        try this.panicAssert() {} catch (bytes memory r) {
            acc = keccak256(abi.encode(acc, r));
        }
        try this.panicDivZero(0) {} catch (bytes memory r) {
            acc = keccak256(abi.encode(acc, r));
        }
        try this.customRevert() {} catch (bytes memory r) {
            acc = keccak256(abi.encode(acc, r));
        }
        try this.invalidOpcode{gas: 50000}() {} catch (bytes memory r) {
            acc = keccak256(abi.encode(acc, r));
        }
        try this.returnDataOutOfBounds() {} catch (bytes memory r) {
            acc = keccak256(abi.encode(acc, r));
        }
        // Recursion until out of gas; the gas cap keeps the trace shallow enough for JSON clients.
        try this.stackDepth{gas: 20_000}(0) {} catch (bytes memory r) {
            acc = keccak256(abi.encode(acc, r));
        }
        bytes memory big = helper.bigReturn(4096);
        acc = keccak256(abi.encode(acc, big));
        probe("failures", acc);
    }

    function panicAssert() external pure {
        assert(false);
    }

    function panicDivZero(uint256 d) external pure returns (uint256) {
        return 1 / d;
    }

    function customRevert() public pure {
        revert CustomError(7, "custom");
    }

    function invalidOpcode() external pure {
        assembly {
            invalid()
        }
    }

    function returnDataOutOfBounds() external view {
        address h = address(helper);
        bytes memory data = abi.encodeCall(Helper.get, ());
        assembly {
            pop(staticcall(gas(), h, add(data, 32), mload(data), 0, 0))
            returndatacopy(0, 0, 64)
        }
    }

    function stackDepth(uint256 n) external returns (uint256) {
        return this.stackDepth(n + 1);
    }

    // Top-level failures.
    function revertTop() external pure {
        revert CustomError(1, "top level revert");
    }

    function revertTopString() external pure {
        revert("top level string revert");
    }

    function invalidTop() external pure {
        assembly {
            invalid()
        }
    }

    function burnTop() external pure {
        while (true) {}
    }

    // Logs emitted before a failure are discarded with the failing frame.
    function logThenRevert() public {
        probe("beforeRevert", 0);
        revert CustomError(2, "after log");
    }

    function logThenInvalid() external {
        probe("beforeInvalid", 0);
        assembly {
            invalid()
        }
    }

    function logThenBurn() external {
        probe("beforeBurn", 0);
        while (true) {}
    }

    function nestedLogRevert() external {
        try this.logThenRevert() {} catch {}
        probe("afterNestedRevert", 0);
    }

    // Arithmetic and bitwise opcodes with dynamic gas (EXP) and edge operands.
    function mathOps(uint256 x) external returns (uint256 r) {
        assembly {
            r := exp(x, 0xffffffffffffffffffffffffffffffff)
            r := xor(r, mulmod(x, not(0), sub(0, 3)))
            r := xor(r, addmod(not(0), not(0), 7))
            r := xor(r, signextend(0, 0xff))
            r := xor(r, sar(4, sub(0, 256)))
            r := xor(r, shl(255, 1))
            r := xor(r, shr(1, not(0)))
            r := xor(r, byte(31, x))
            r := xor(r, sdiv(sub(0, 10), 3))
            r := xor(r, smod(sub(0, 10), 3))
        }
        probe("math", bytes32(r));
    }

    // Block and transaction environment opcodes.
    function envOps() external {
        bytes32 h = keccak256(
            abi.encode(
                block.number,
                block.timestamp,
                block.basefee,
                block.blobbasefee,
                block.chainid,
                block.gaslimit,
                block.coinbase,
                block.prevrandao,
                blockhash(block.number - 1),
                blobhash(0),
                tx.gasprice,
                tx.origin,
                msg.sender,
                address(this).balance
            )
        );
        probe("env", h);
    }

    // Large calldata exercises intrinsic gas and the EIP-7623 calldata floor under Prague.
    function calldataSink(bytes calldata data) external pure returns (uint256) {
        return data.length;
    }

    // Precompiles: forward a fixed amount of gas, record success, output hash and gas used.
    function callPrecompile(address target, bytes calldata input, uint256 gasLimit) public returns (bool ok) {
        uint256 before = gasleft();
        bytes memory out;
        (ok, out) = target.staticcall{gas: gasLimit}(input);
        uint256 used = before - gasleft();
        emit Probe(bytes32(uint256(uint160(target))), used, keccak256(abi.encode(ok, out)));
    }

    // Precompiles reached through CALL, DELEGATECALL and CALLCODE. The engine resolves
    // precompiles by target address, so the delegated forms run the (empty) code at the
    // precompile address instead of the precompile.
    function precompileCallKinds() external {
        bytes memory input = hex"616263";
        bytes memory bip322Selector = hex"de8f50a1";
        bytes32 acc;
        (bool ok, bytes memory out) = address(2).call(input);
        acc = keccak256(abi.encode(acc, ok, out));
        (ok, out) = address(2).delegatecall(input);
        acc = keccak256(abi.encode(acc, ok, out));
        (ok, out) = address(0xfe).delegatecall(bip322Selector);
        acc = keccak256(abi.encode(acc, ok, out));
        (ok, out) = address(0xfa).delegatecall("");
        acc = keccak256(abi.encode(acc, ok, out));
        bool callcodeOk;
        bytes32 callcodeOut;
        assembly {
            callcodeOk := callcode(gas(), 2, 0, add(input, 32), mload(input), 0, 32)
            callcodeOut := mload(0)
        }
        acc = keccak256(abi.encode(acc, callcodeOk, callcodeOut));
        probe("precompileCallKinds", acc);
    }

    function precompileBatch(address[] calldata targets, bytes[] calldata inputs, uint256 gasLimit) external {
        for (uint256 i = 0; i < targets.length; i++) {
            callPrecompile(targets[i], inputs[i], gasLimit);
        }
    }
}

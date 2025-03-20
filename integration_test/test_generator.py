stack_limit = 1023


class Mode:
    min_stack = 0
    full_stack = 1


class InstructionCategory:
    nop = "n"  # < No-op instruction.
    nullop = "a"  # < Nullary operator - produces a result without any stack input.
    unop = "u"  # < Unary operator.
    binop = "b"  # < Binary operator.
    push = "p"  # < PUSH instruction.
    dup = "d"  # < DUP instruction.
    swap = "s"  # < SWAP instruction.
    other = "X"  # < Not any of the categories above.


OP_STOP = 0x00
OP_ADD = 0x01
OP_MUL = 0x02
OP_SUB = 0x03
OP_DIV = 0x04
OP_SDIV = 0x05
OP_MOD = 0x06
OP_SMOD = 0x07
OP_ADDMOD = 0x08
OP_MULMOD = 0x09
OP_EXP = 0x0A
OP_SIGNEXTEND = 0x0B

OP_LT = 0x10
OP_GT = 0x11
OP_SLT = 0x12
OP_SGT = 0x13
OP_EQ = 0x14
OP_ISZERO = 0x15
OP_AND = 0x16
OP_OR = 0x17
OP_XOR = 0x18
OP_NOT = 0x19
OP_BYTE = 0x1A
OP_SHL = 0x1B
OP_SHR = 0x1C
OP_SAR = 0x1D

OP_KECCAK256 = 0x20

OP_ADDRESS = 0x30
OP_BALANCE = 0x31
OP_ORIGIN = 0x32
OP_CALLER = 0x33
OP_CALLVALUE = 0x34
OP_CALLDATALOAD = 0x35
OP_CALLDATASIZE = 0x36
OP_CALLDATACOPY = 0x37
OP_CODESIZE = 0x38
OP_CODECOPY = 0x39
OP_GASPRICE = 0x3A
OP_EXTCODESIZE = 0x3B
OP_EXTCODECOPY = 0x3C
OP_RETURNDATASIZE = 0x3D
OP_RETURNDATACOPY = 0x3E
OP_EXTCODEHASH = 0x3F

OP_BLOCKHASH = 0x40
OP_COINBASE = 0x41
OP_TIMESTAMP = 0x42
OP_NUMBER = 0x43
OP_PREVRANDAO = 0x44
OP_GASLIMIT = 0x45
OP_CHAINID = 0x46
OP_SELFBALANCE = 0x47
OP_BASEFEE = 0x48
OP_BLOBHASH = 0x49
OP_BLOBBASEFEE = 0x4A

OP_POP = 0x50
OP_MLOAD = 0x51
OP_MSTORE = 0x52
OP_MSTORE8 = 0x53
OP_SLOAD = 0x54
OP_SSTORE = 0x55
OP_JUMP = 0x56
OP_JUMPI = 0x57
OP_PC = 0x58
OP_MSIZE = 0x59
OP_GAS = 0x5A
OP_JUMPDEST = 0x5B
OP_TLOAD = 0x5C
OP_TSTORE = 0x5D
OP_MCOPY = 0x5E
OP_PUSH0 = 0x5F
OP_PUSH1 = 0x60
OP_PUSH2 = 0x61
OP_PUSH3 = 0x62
OP_PUSH4 = 0x63
OP_PUSH5 = 0x64
OP_PUSH6 = 0x65
OP_PUSH7 = 0x66
OP_PUSH8 = 0x67
OP_PUSH9 = 0x68
OP_PUSH10 = 0x69
OP_PUSH11 = 0x6A
OP_PUSH12 = 0x6B
OP_PUSH13 = 0x6C
OP_PUSH14 = 0x6D
OP_PUSH15 = 0x6E
OP_PUSH16 = 0x6F
OP_PUSH17 = 0x70
OP_PUSH18 = 0x71
OP_PUSH19 = 0x72
OP_PUSH20 = 0x73
OP_PUSH21 = 0x74
OP_PUSH22 = 0x75
OP_PUSH23 = 0x76
OP_PUSH24 = 0x77
OP_PUSH25 = 0x78
OP_PUSH26 = 0x79
OP_PUSH27 = 0x7A
OP_PUSH28 = 0x7B
OP_PUSH29 = 0x7C
OP_PUSH30 = 0x7D
OP_PUSH31 = 0x7E
OP_PUSH32 = 0x7F
OP_DUP1 = 0x80
OP_DUP2 = 0x81
OP_DUP3 = 0x82
OP_DUP4 = 0x83
OP_DUP5 = 0x84
OP_DUP6 = 0x85
OP_DUP7 = 0x86
OP_DUP8 = 0x87
OP_DUP9 = 0x88
OP_DUP10 = 0x89
OP_DUP11 = 0x8A
OP_DUP12 = 0x8B
OP_DUP13 = 0x8C
OP_DUP14 = 0x8D
OP_DUP15 = 0x8E
OP_DUP16 = 0x8F
OP_SWAP1 = 0x90
OP_SWAP2 = 0x91
OP_SWAP3 = 0x92
OP_SWAP4 = 0x93
OP_SWAP5 = 0x94
OP_SWAP6 = 0x95
OP_SWAP7 = 0x96
OP_SWAP8 = 0x97
OP_SWAP9 = 0x98
OP_SWAP10 = 0x99
OP_SWAP11 = 0x9A
OP_SWAP12 = 0x9B
OP_SWAP13 = 0x9C
OP_SWAP14 = 0x9D
OP_SWAP15 = 0x9E
OP_SWAP16 = 0x9F
OP_LOG0 = 0xA0
OP_LOG1 = 0xA1
OP_LOG2 = 0xA2
OP_LOG3 = 0xA3
OP_LOG4 = 0xA4

OP_DATALOAD = 0xD0
OP_DATALOADN = 0xD1
OP_DATASIZE = 0xD2
OP_DATACOPY = 0xD3

OP_RJUMP = 0xE0
OP_RJUMPI = 0xE1
OP_RJUMPV = 0xE2
OP_CALLF = 0xE3
OP_RETF = 0xE4
OP_JUMPF = 0xE5

OP_DUPN = 0xE6
OP_SWAPN = 0xE7

OP_CREATE = 0xF0
OP_CALL = 0xF1
OP_CALLCODE = 0xF2
OP_RETURN = 0xF3
OP_DELEGATECALL = 0xF4
OP_CREATE2 = 0xF5
OP_RETURNDATALOAD = 0xF7

OP_STATICCALL = 0xFA

OP_REVERT = 0xFD
OP_INVALID = 0xFE
OP_SELFDESTRUCT = 0xFF

traits = {}

traits[OP_STOP] = [0, 0]
traits[OP_ADD] = [2, -1]
traits[OP_MUL] = [2, -1]
traits[OP_SUB] = [2, -1]
traits[OP_DIV] = [2, -1]
traits[OP_SDIV] = [2, -1]
traits[OP_MOD] = [2, -1]
traits[OP_SMOD] = [2, -1]
traits[OP_ADDMOD] = [3, -2]
traits[OP_MULMOD] = [3, -2]
traits[OP_EXP] = [2, -1]
traits[OP_SIGNEXTEND] = [2, -1]

traits[OP_LT] = [2, -1]
traits[OP_GT] = [2, -1]
traits[OP_SLT] = [2, -1]
traits[OP_SGT] = [2, -1]
traits[OP_EQ] = [2, -1]
traits[OP_ISZERO] = [1, 0]
traits[OP_AND] = [2, -1]
traits[OP_OR] = [2, -1]
traits[OP_XOR] = [2, -1]
traits[OP_NOT] = [1, 0]
traits[OP_BYTE] = [2, -1]
traits[OP_SHL] = [2, -1]
traits[OP_SHR] = [2, -1]
traits[OP_SAR] = [2, -1]

traits[OP_KECCAK256] = [2, -1]

traits[OP_ADDRESS] = [0, 1]
traits[OP_BALANCE] = [1, 0]
traits[OP_ORIGIN] = [0, 1]
traits[OP_CALLER] = [0, 1]
traits[OP_CALLVALUE] = [0, 1]
traits[OP_CALLDATALOAD] = [1, 0]
traits[OP_CALLDATASIZE] = [0, 1]
traits[OP_CALLDATACOPY] = [3, -3]
traits[OP_CODESIZE] = [0, 1]
traits[OP_CODECOPY] = [3, -3]
traits[OP_GASPRICE] = [0, 1]
traits[OP_EXTCODESIZE] = [1, 0]
traits[OP_EXTCODECOPY] = [4, -4]
traits[OP_RETURNDATASIZE] = [0, 1]
traits[OP_RETURNDATACOPY] = [3, -3]
traits[OP_EXTCODEHASH] = [1, 0]

traits[OP_BLOCKHASH] = [1, 0]
traits[OP_COINBASE] = [0, 1]
traits[OP_TIMESTAMP] = [0, 1]
traits[OP_NUMBER] = [0, 1]
traits[OP_PREVRANDAO] = [0, 1]
traits[OP_GASLIMIT] = [0, 1]
traits[OP_CHAINID] = [0, 1]
traits[OP_SELFBALANCE] = [0, 1]
traits[OP_BASEFEE] = [0, 1]
traits[OP_BLOBHASH] = [1, 0]
traits[OP_BLOBBASEFEE] = [0, 1]

traits[OP_POP] = [1, -1]
traits[OP_MLOAD] = [1, 0]
traits[OP_MSTORE] = [2, -2]
traits[OP_MSTORE8] = [2, -2]
traits[OP_SLOAD] = [1, 0]
traits[OP_SSTORE] = [2, -2]
traits[OP_JUMP] = [1, -1]
traits[OP_JUMPI] = [2, -2]
traits[OP_PC] = [0, 1]
traits[OP_MSIZE] = [0, 1]
traits[OP_GAS] = [0, 1]
traits[OP_JUMPDEST] = [0, 0]
traits[OP_RJUMP] = [0, 0]
traits[OP_RJUMPI] = [1, -1]
traits[OP_RJUMPV] = [1, -1]

traits[OP_TLOAD] = [1, 0]
traits[OP_TSTORE] = [2, -2]
traits[OP_PUSH0] = [0, 1]

traits[OP_PUSH1] = [0, 1]
traits[OP_PUSH2] = [0, 1]
traits[OP_PUSH3] = [0, 1]
traits[OP_PUSH4] = [0, 1]
traits[OP_PUSH5] = [0, 1]
traits[OP_PUSH6] = [0, 1]
traits[OP_PUSH7] = [0, 1]
traits[OP_PUSH8] = [0, 1]
traits[OP_PUSH9] = [0, 1]
traits[OP_PUSH10] = [0, 1]
traits[OP_PUSH11] = [0, 1]
traits[OP_PUSH12] = [0, 1]
traits[OP_PUSH13] = [0, 1]
traits[OP_PUSH14] = [0, 1]
traits[OP_PUSH15] = [0, 1]
traits[OP_PUSH16] = [0, 1]
traits[OP_PUSH17] = [0, 1]
traits[OP_PUSH18] = [0, 1]
traits[OP_PUSH19] = [0, 1]
traits[OP_PUSH20] = [0, 1]
traits[OP_PUSH21] = [0, 1]
traits[OP_PUSH22] = [0, 1]
traits[OP_PUSH23] = [0, 1]
traits[OP_PUSH24] = [0, 1]
traits[OP_PUSH25] = [0, 1]
traits[OP_PUSH26] = [0, 1]
traits[OP_PUSH27] = [0, 1]
traits[OP_PUSH28] = [0, 1]
traits[OP_PUSH29] = [0, 1]
traits[OP_PUSH30] = [0, 1]
traits[OP_PUSH31] = [0, 1]
traits[OP_PUSH32] = [0, 1]

traits[OP_DUP1] = [1, 1]
traits[OP_DUP2] = [2, 1]
traits[OP_DUP3] = [3, 1]
traits[OP_DUP4] = [4, 1]
traits[OP_DUP5] = [5, 1]
traits[OP_DUP6] = [6, 1]
traits[OP_DUP7] = [7, 1]
traits[OP_DUP8] = [8, 1]
traits[OP_DUP9] = [9, 1]
traits[OP_DUP10] = [10, 1]
traits[OP_DUP11] = [11, 1]
traits[OP_DUP12] = [12, 1]
traits[OP_DUP13] = [13, 1]
traits[OP_DUP14] = [14, 1]
traits[OP_DUP15] = [15, 1]
traits[OP_DUP16] = [16, 1]

traits[OP_SWAP1] = [2, 0]
traits[OP_SWAP2] = [3, 0]
traits[OP_SWAP3] = [4, 0]
traits[OP_SWAP4] = [5, 0]
traits[OP_SWAP5] = [6, 0]
traits[OP_SWAP6] = [7, 0]
traits[OP_SWAP7] = [8, 0]
traits[OP_SWAP8] = [9, 0]
traits[OP_SWAP9] = [10, 0]
traits[OP_SWAP10] = [11, 0]
traits[OP_SWAP11] = [12, 0]
traits[OP_SWAP12] = [13, 0]
traits[OP_SWAP13] = [14, 0]
traits[OP_SWAP14] = [15, 0]
traits[OP_SWAP15] = [16, 0]
traits[OP_SWAP16] = [17, 0]

traits[OP_LOG0] = [2, -2]
traits[OP_LOG1] = [3, -3]
traits[OP_LOG2] = [4, -4]
traits[OP_LOG3] = [5, -5]
traits[OP_LOG4] = [6, -6]

traits[OP_DUPN] = [0, 1]
traits[OP_SWAPN] = [0, 0]
traits[OP_MCOPY] = [3, -3]
traits[OP_DATALOAD] = [1, 0]
traits[OP_DATALOADN] = [0, 1]
traits[OP_DATASIZE] = [0, 1]
traits[OP_DATACOPY] = [3, -3]

traits[OP_CREATE] = [3, -2]
traits[OP_CALL] = [7, -6]
traits[OP_CALLCODE] = [7, -6]
traits[OP_RETURN] = [2, -2]
traits[OP_DELEGATECALL] = [6, -5]
traits[OP_CREATE2] = [4, -3]
traits[OP_RETURNDATALOAD] = [1, 0]
traits[OP_STATICCALL] = [6, -5]
traits[OP_CALLF] = [0, 0]
traits[OP_RETF] = [0, 0]
traits[OP_JUMPF] = [0, 0]
traits[OP_REVERT] = [2, -2]
traits[OP_INVALID] = [0, 0]
traits[OP_SELFDESTRUCT] = [1, -1]


def get_instruction_category(opcode):
    trait = traits[opcode]
    stack_height_required = trait[0]
    stack_height_change = trait[1]
    if opcode >= OP_PUSH1 and opcode <= OP_PUSH32:
        return InstructionCategory.push
    elif opcode >= OP_SWAP1 and opcode <= OP_SWAP16:
        return InstructionCategory.swap
    elif opcode >= OP_DUP1 and opcode <= OP_DUP16:
        return InstructionCategory.dup
    elif stack_height_required == 0 and stack_height_change == 0:
        return InstructionCategory.nop
    elif stack_height_required == 0 and stack_height_change == 1:
        return InstructionCategory.nullop
    elif stack_height_required == 1 and stack_height_change == 0:
        return InstructionCategory.unop
    elif stack_height_required == 2 and stack_height_change == -1:
        return InstructionCategory.binop
    else:
        return InstructionCategory.other


def get(opcode):
    h = hex(opcode)[2:]
    if len(h) == 1:
        h = "0" + h
    return h


def push(opcode):
    if opcode < OP_PUSH1 or opcode > OP_PUSH32:
        print("Invalid opcode - PUSH")
        exit(1)
    num_instr_bytes = opcode - OP_PUSH1 + 1
    instr_bytes = "00" * num_instr_bytes
    return get(opcode) + instr_bytes


def push_hex(hex_str):
    num_instr_bytes = len(hex_str) // 2
    if num_instr_bytes < 1 or num_instr_bytes > 32:
        print("Invalid hex string - PUSH")
        exit(1)
    return get(OP_PUSH1 + num_instr_bytes - 1) + hex_str


def push_int(num):
    hex_str = hex(num)[2:]
    if len(hex_str) % 2 == 1:
        hex_str = "0" + hex_str
    return push_hex(hex_str)


def generate_loop_inner_code(opcode, mode):
    category = get_instruction_category(opcode)
    if mode == Mode.min_stack:
        if category == InstructionCategory.nop:
            return stack_limit * 2 * get(opcode)
        elif category == InstructionCategory.nullop:
            return stack_limit * (get(opcode) + get(OP_POP))
        elif category == InstructionCategory.unop:
            return get(OP_DUP1) + stack_limit * 2 * get(opcode) + get(OP_POP)
        elif category == InstructionCategory.binop:
            return (
                get(OP_DUP1)
                + (stack_limit - 1) * (get(OP_DUP1) + get(opcode))
                + get(OP_POP)
            )
        elif category == InstructionCategory.push:
            return stack_limit * (push(opcode) + get(OP_POP))
        elif category == InstructionCategory.dup:
            n = opcode - OP_DUP1 + 1
            return (
                (n - 1) * get(OP_DUP1)
                + (stack_limit - (n - 1)) * (get(opcode) + get(OP_POP))
                + (n - 1) * get(OP_POP)
            )
        elif category == InstructionCategory.swap:
            n = opcode - OP_SWAP1 + 1
            return n * get(OP_DUP1) + stack_limit * 2 * get(opcode) + n * get(OP_POP)
        else:
            print("Invalid opcode - LOOP")
            exit(1)
    elif mode == Mode.full_stack:
        if category == InstructionCategory.nullop:
            return stack_limit * get(opcode) + stack_limit * get(OP_POP)
        elif category == InstructionCategory.binop:
            return (
                stack_limit * get(OP_DUP1)
                + (stack_limit - 1) * get(opcode)
                + get(OP_POP)
            )
        elif category == InstructionCategory.push:
            return stack_limit * push(opcode) + stack_limit * get(OP_POP)
        elif category == InstructionCategory.dup:
            n = opcode - OP_DUP1 + 1
            return (
                (n - 1) * get(OP_DUP1)
                + (stack_limit - (n - 1)) * get(opcode)
                + stack_limit * get(OP_POP)
            )
        else:
            print("Invalid opcode - LOOP")
            exit(1)
    else:
        print("Invalid mode - LOOP")
        exit(1)


def generate_loop_v2(inner_code):
    counter = push_hex(
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffb001"
    )
    jumpdest_offset = len(counter) // 2
    return (
        counter
        + get(OP_JUMPDEST)
        + inner_code
        + push_int(1)
        + get(OP_ADD)
        + get(OP_DUP1)
        + push_int(jumpdest_offset)
        + get(OP_JUMPI)
    )


def generate_code(opcode, mode):
    return generate_loop_v2(generate_loop_inner_code(opcode, mode))


def wrap_deploy_code(code):
    codelen = len(code) // 2
    codelen_hex = hex(codelen)[2:]
    codelen_hex = "0" * (16 - len(codelen_hex)) + codelen_hex
    res = "0x"
    res += get(OP_PUSH8) + codelen_hex  ## code size
    res += get(OP_PUSH1) + "1b"  ## code offset
    res += get(OP_PUSH1) + "00"  ## target offset, fixed
    res += get(OP_CODECOPY)
    res += get(OP_PUSH8) + codelen_hex  ## code size
    res += get(OP_PUSH1) + "00"  ## memory offset, fixed
    res += get(OP_RETURN)
    return res + "d5" + code


def get_all_bench_codes():
    codes = []
    params_list = []
    for opcode in [OP_JUMPDEST, OP_ISZERO, OP_NOT]:
        params_list.append([opcode, Mode.min_stack])
    for opcode in [
        OP_ADD,
        OP_MUL,
        OP_SUB,
        OP_SIGNEXTEND,
        OP_LT,
        OP_GT,
        OP_SLT,
        OP_SGT,
        OP_EQ,
        OP_AND,
        OP_OR,
        OP_XOR,
        OP_BYTE,
        OP_SHL,
        OP_SHR,
        OP_SAR,
    ]:
        params_list.append([opcode, Mode.min_stack])
        params_list.append([opcode, Mode.full_stack])
    for opcode in [
        OP_ADDRESS,
        OP_CALLER,
        OP_CALLVALUE,
        OP_CALLDATASIZE,
        OP_CODESIZE,
        OP_RETURNDATASIZE,
        OP_PC,
        OP_MSIZE,
        OP_GAS,
    ]:
        params_list.append([opcode, Mode.min_stack])
        params_list.append([opcode, Mode.full_stack])
    for opcode in range(OP_PUSH1, OP_PUSH32 + 1):
        params_list.append([opcode, Mode.min_stack])
        params_list.append([opcode, Mode.full_stack])
    for opcode in range(OP_SWAP1, OP_SWAP16 + 1):
        params_list.append([opcode, Mode.min_stack])
    for opcode in range(OP_DUP1, OP_DUP16 + 1):
        params_list.append([opcode, Mode.min_stack])
        params_list.append([opcode, Mode.full_stack])
    for params in params_list:
        codes.append(wrap_deploy_code(generate_code(*params)))
    return [codes, params_list]


codes, params_list = get_all_bench_codes()

import requests

url_mine = "http://localhost:8000/mine_block"
data_mine = {
    "ts": 5,
    "hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "txes": [
        {
            "inscription": {"op": "deploy", "d": ""},
            "btc_pkscript": "512037679ea62eab55ebfd442c53c4ad46b6b75e45d8a8fa9cb31a87d0df268b029a",
        }
    ],
}
data_call = {
    "ts": 5,
    "hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "txes": [
        {
            "inscription": {"op": "call", "c": "", "d": "0x"},
            "btc_pkscript": "512037679ea62eab55ebfd442c53c4ad46b6b75e45d8a8fa9cb31a87d0df268b029a",
        }
    ],
}

url_current_block_height = "http://localhost:8000/current_block_height"
url_get_block_info = "http://localhost:8000/get_block_info?block_height="


def convert_hex_or_decimal_to_float(s):
    if s.startswith("0x"):
        return float(int(s, 16))
    else:
        return float(s)


ress = []
for i in range(len(codes)):
    code = codes[i]
    params = params_list[i]
    print("deploying: ", get(params[0]), params[1])
    data_mine["txes"][0]["inscription"]["d"] = code
    response = requests.post(
        url_mine, json=data_mine, headers={"Content-Type": "application/json"}
    )
    js = response.json()
    contractAddress = js["result"]["responses"][0]["receipt"]["contractAddress"]
    print("contractAddress: ", contractAddress)
    data_call["txes"][0]["inscription"]["c"] = contractAddress
    response = requests.post(
        url_mine, json=data_call, headers={"Content-Type": "application/json"}
    )
    js = response.json()
    if js["result"]["responses"][0]["receipt"]["txResult"] != "Success":
        print("failed: ", params)
        exit(1)
    response = requests.get(url_current_block_height)
    js = response.json()
    blockHeight = js["result"]
    response = requests.get(url_get_block_info + str(blockHeight))
    js = response.json()
    gasUsed = convert_hex_or_decimal_to_float(js["result"]["gasUsed"])
    mineTm = convert_hex_or_decimal_to_float(js["result"]["mineTimestamp"])
    ratio = mineTm / gasUsed
    print("ratio: ", ratio, " gasUsed: ", gasUsed, " mineTimestamp: ", mineTm)
    ress.append([params, ratio, gasUsed, mineTm])

ress.sort(key=lambda x: x[1], reverse=True)
for res in ress:
    params, ratio, gasUsed, mineTm = res
    print(
        "params:",
        get(params[0]),
        params[1],
        "ratio:",
        ratio,
        "gasUsed:",
        gasUsed,
        "mineTm:",
        mineTm,
    )

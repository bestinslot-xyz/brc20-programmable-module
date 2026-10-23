#!/usr/bin/env bash
# Regenerates the consensus fixture inputs from Fixture.sol. Requires Foundry (forge, cast).
#   fixture_deploy_tx_data  creation bytecode of Fixture
#   fixture_calls           one "label calldata" pair per line, consumed by tests/consensus_fixture_*.rs
# Changing either file changes every consensus fixture snapshot, so regenerate only together
# with an intentional snapshot update.
set -euo pipefail
cd "$(dirname "$0")"
out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT
forge build --contracts Fixture.sol --use 0.8.28 --evm-version cancun --optimize --optimizer-runs 200 \
    --out "$out/out" --cache-path "$out/cache" --root . >/dev/null
python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["bytecode"]["object"], end="")' \
    "$out/out/Fixture.sol/Fixture.json" > fixture_deploy_tx_data

zeros() { printf '%0*d' "$(($1 * 2))" 0; }
word() { printf '%064x' "$1"; }

# ecrecover: keccak("brc20 consensus fixture") signed by private key 0x01
ecrecover="1a8a9c6f81d3c217ef9c92054353df2e7ecd997bfa42a4343ae92194285eb19e$(word 27)"
ecrecover+="9e472ba7c681ebc6157bbef8dee8548e81e3cd5a0e1ebea58e50e48f72054149"
ecrecover+="3580129da3e8d141a37eb7cc0e7de27b7b3c1a7b8d4fe8d901eef5594c8a3a01"
# modexp: 3^(2^256-1) mod (2^255-19), then a 512-byte modulus for the EIP-2565 length term
modexp_small="$(word 1)$(word 32)$(word 32)03$(printf 'f%.0s' {1..64})7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffed"
modexp_big="$(word 64)$(word 64)$(word 512)$(printf '07%.0s' {1..64})$(printf 'ab%.0s' {1..64})$(printf 'ff%.0s' {1..512})"
# alt_bn128: G1 = (1, 2)
bn_add="$(word 1)$(word 2)$(word 1)$(word 2)"
bn_mul="$(word 1)$(word 2)$(word 3)"
bn_pairing_bad="$(word 1)$(word 2)$(zeros 128)"
# blake2f: EIP-152 test vector 5 ("abc", 12 rounds, final block)
blake2f="0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b"
blake2f+="616263$(zeros 125)03000000000000000000000000000000""01"
# BLS12-381 (EIP-2537): G1 generator, padded to 64-byte field elements
g1x="17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb"
g1y="08b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1"
g1="$(zeros 16)${g1x}$(zeros 16)${g1y}"

# Inputs the precompiles accept. A rejected input aborts the whole transaction in this engine,
# so these share one transaction and each rejected input below gets a transaction of its own.
eth_targets=() eth_inputs=()
add() { eth_targets+=("$1"); eth_inputs+=("0x$2"); }
add 0x0000000000000000000000000000000000000001 "$ecrecover"
add 0x0000000000000000000000000000000000000001 "$(zeros 128)"
add 0x0000000000000000000000000000000000000002 "616263"
add 0x0000000000000000000000000000000000000003 "616263"
add 0x0000000000000000000000000000000000000004 "$(printf '5a%.0s' {1..100})"
add 0x0000000000000000000000000000000000000005 "$modexp_small"
add 0x0000000000000000000000000000000000000005 "$modexp_big"
add 0x0000000000000000000000000000000000000006 "$bn_add"
add 0x0000000000000000000000000000000000000007 "$bn_mul"
add 0x0000000000000000000000000000000000000008 ""
add 0x0000000000000000000000000000000000000008 "$bn_pairing_bad"
add 0x0000000000000000000000000000000000000009 "$blake2f"
add 0x000000000000000000000000000000000000000b "${g1}${g1}"
add 0x000000000000000000000000000000000000000c "${g1}$(word 2)"
add 0x0000000000000000000000000000000000000010 "$(zeros 64)"
add 0x0000000000000000000000000000000000000011 "$(zeros 128)"
add 0x0000000000000000000000000000000000000100 "$(zeros 160)"

# Rejected inputs, out-of-gas calls and fork-gated limits, one transaction each:
# label, target, input, gas forwarded to the precompile.
modexp_over_7823="$(word 1)$(word 1)$(word 1025)03"01"$(printf 'ff%.0s' {1..1025})"
edge_cases=(
    "pointEvaluationZeros 0x000000000000000000000000000000000000000a 0x$(zeros 192) 1000000"
    "blake2fShortInput 0x0000000000000000000000000000000000000009 0x00 1000000"
    "bnAddPointNotOnCurve 0x0000000000000000000000000000000000000006 0x$(word 1)$(word 3)$(word 1)$(word 2) 1000000"
    "bnPairingBadLength 0x0000000000000000000000000000000000000008 0x00 1000000"
    "blsG1AddEmpty 0x000000000000000000000000000000000000000b 0x 1000000"
    "blsG1MsmEmpty 0x000000000000000000000000000000000000000c 0x 1000000"
    "blsG2AddEmpty 0x000000000000000000000000000000000000000d 0x 1000000"
    "blsG2MsmEmpty 0x000000000000000000000000000000000000000e 0x 1000000"
    "blsPairingEmpty 0x000000000000000000000000000000000000000f 0x 1000000"
    "blsMapFpToG1BadLength 0x0000000000000000000000000000000000000010 0x00 1000000"
    "ecrecoverOutOfGas 0x0000000000000000000000000000000000000001 0x$ecrecover 2999"
    "sha256OutOfGas 0x0000000000000000000000000000000000000002 0x616263 10"
    "modexpAbove7823Limit 0x0000000000000000000000000000000000000005 0x$modexp_over_7823 1000000"
)

brc20_targets=() brc20_inputs=()
brc20_targets+=(0x00000000000000000000000000000000000000fa) brc20_inputs+=(0x)
brc20_targets+=(0x00000000000000000000000000000000000000fb) brc20_inputs+=("$(cat ../btc_get_locked_pkscript_call_tx_data)")
brc20_targets+=(0x00000000000000000000000000000000000000fe) brc20_inputs+=("$(cat ../bip322_verify_call_tx_data)")
brc20_targets+=(0x00000000000000000000000000000000000000fe) brc20_inputs+=(0xde8f50a1)

join() { local IFS=,; echo "[$*]"; }
calldata_sink="0x$(zeros 2048)$(printf '01%.0s' {1..2048})"

{
    echo "storageOps $(cast calldata 'storageOps()')"
    echo "transientOps $(cast calldata 'transientOps()')"
    echo "memoryOps $(cast calldata 'memoryOps(uint256)' 20000)"
    echo "logOps $(cast calldata 'logOps()')"
    echo "nested $(cast calldata 'nested(uint256)' 8)"
    echo "callKinds $(cast calldata 'callKinds()')"
    echo "createOps $(cast calldata 'createOps(bytes32)' 0x0000000000000000000000000000000000000000000000000000000000000001)"
    echo "createOpsSecondSalt $(cast calldata 'createOps(bytes32)' 0x0000000000000000000000000000000000000000000000000000000000000002)"
    echo "bigRuntime $(cast calldata 'bigRuntime(uint256)' 30000)"
    echo "bigInitcode $(cast calldata 'bigInitcode(uint256)' 60000)"
    echo "createAndDestroy $(cast calldata 'createAndDestroy()')"
    echo "destroyLast $(cast calldata 'destroyLast()')"
    echo "failures $(cast calldata 'failures()')"
    echo "revertTop $(cast calldata 'revertTop()')"
    echo "revertTopString $(cast calldata 'revertTopString()')"
    echo "invalidTop $(cast calldata 'invalidTop()')"
    echo "burnTop $(cast calldata 'burnTop()')"
    echo "logThenRevert $(cast calldata 'logThenRevert()')"
    echo "logThenInvalid $(cast calldata 'logThenInvalid()')"
    echo "logThenBurn $(cast calldata 'logThenBurn()')"
    echo "nestedLogRevert $(cast calldata 'nestedLogRevert()')"
    echo "mathOps $(cast calldata 'mathOps(uint256)' 0x1234567890abcdef1234567890abcdef)"
    echo "envOps $(cast calldata 'envOps()')"
    echo "calldataSink $(cast calldata 'calldataSink(bytes)' "$calldata_sink")"
    echo "ethPrecompiles $(cast calldata 'precompileBatch(address[],bytes[],uint256)' "$(join "${eth_targets[@]}")" "$(join "${eth_inputs[@]}")" 1000000)"
    echo "brc20Precompiles $(cast calldata 'precompileBatch(address[],bytes[],uint256)' "$(join "${brc20_targets[@]}")" "$(join "${brc20_inputs[@]}")" 1000000)"
    echo "unknownSelector 0xdeadbeef"
    echo "precompileCallKinds $(cast calldata 'precompileCallKinds()')"
    echo "probeDestroyedChild $(cast calldata 'probeDestroyedChild()')"
    # Five PUSH1 1 for stack arguments, the opcode under test, then STOP. CLZ (Osaka),
    # DUPN/SWAPN/EXCHANGE (Amsterdam) and EOF opcodes must stay invalid; the rest are active.
    runtimes=()
    for op in 1e e6 e7 e8 d0 d1 e0 e1 ec ee f8 f9 fb 5f 49 4a 5e; do
        runtimes+=("0x60016001600160016001${op}00")
    done
    echo "opcodeProbe $(cast calldata 'runOpcodes(bytes[])' "$(join "${runtimes[@]}")")"
    for entry in "${edge_cases[@]}"; do
        read -r label target input gas <<<"$entry"
        echo "precompileEdge:$label $(cast calldata 'callPrecompile(address,bytes,uint256)' "$target" "$input" "$gas")"
    done
} > fixture_calls

# Signed legacy transactions for brc20_transact, one file per chain id.
# fixture_txs_mainnet uses chain id "BRC20", fixture_txs_testnets uses "BRC20s".
key=0x0000000000000000000000000000000000000000000000000000000000000002
sender="$(cast wallet address --private-key "$key")"
contract="$(cast compute-address "$sender" --nonce 0 | awk '{print $NF}')"
call() { awk -v l="$1" '$1 == l {print $2}' fixture_calls; }
mktx() { cast mktx --legacy --private-key "$key" --gas-price 0 --gas-limit 0 --chain "$@"; }
for network in mainnet:$((16#4252433230)):$((16#425243323073)) testnets:$((16#425243323073)):$((16#4252433230)); do
    IFS=: read -r name chain other_chain <<<"$network"
    {
        echo "deploy $(mktx "$chain" --nonce 0 --create "$(cat fixture_deploy_tx_data)")"
        echo "storageOps $(mktx "$chain" --nonce 1 "$contract" "$(call storageOps)")"
        echo "futureNonceLogOps $(mktx "$chain" --nonce 3 "$contract" "$(call logOps)")"
        echo "fillGapNested $(mktx "$chain" --nonce 2 "$contract" "$(call nested)")"
        echo "highSCallKinds $(python3 -I malleate.py "$(mktx "$chain" --nonce 4 "$contract" "$(call callKinds)")")"
        echo "wrongChainId $(mktx "$other_chain" --nonce 5 "$contract" "$(call envOps)")"
        echo "failures $(mktx "$chain" --nonce 5 "$contract" "$(call failures)")"
        echo "revertTop $(mktx "$chain" --nonce 6 "$contract" "$(call revertTop)")"
        echo "malformed 0xdeadbeef"
    } > "fixture_txs_$name"
done

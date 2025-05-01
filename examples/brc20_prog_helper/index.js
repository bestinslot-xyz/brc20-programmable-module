var solc_fixed = require('solc_0_8_24');
var ethers = require('ethers');
var fs = require('fs');

function findImports(path) {
    if (!fs.existsSync(path)) {
        path = 'node_modules/' + path
    }
    if (!fs.existsSync(path)) {
        return { error: 'File not found' };
    }
    return {
        contents:
            fs.readFileSync(path, 'utf8')
    };
}

async function main() {
    let BRC20Prog_sol = fs.readFileSync("./BRC20_Prog.sol", "utf8");

    let input = {
        language: 'Solidity',
        sources: {
            'BRC20_Prog.sol': { content: BRC20Prog_sol },
        },
        settings: {
            metadata: {
              useLiteralContent: true
            },
            evmVersion: "cancun",
            outputSelection: {
                '*': {
                    '*': ['*']
                }
            }
        }
    };

    let compiled = JSON.parse(solc_fixed.compile(JSON.stringify(input), { import: findImports }))
    let contract = compiled.contracts['BRC20_Prog.sol']["BRC20_Prog"]
    let bytecode = contract.evm.bytecode.object
    let abi = contract.abi

    let contract_factory = new ethers.ContractFactory(abi, bytecode, null)
    let deploy_tx = await contract_factory.getDeployTransaction()

    fs.mkdirSync('output', { recursive: true });

    let metadata = JSON.parse(contract["metadata"])
    delete metadata.output

    fs.writeFileSync('output/BRC20_Prog.json', JSON.stringify(metadata, null, 4), function (_) { });
    fs.writeFileSync('output/BRC20_Prog.abi', JSON.stringify(abi, null, 4), function (_) { });
    fs.writeFileSync('output/BRC20_Prog.bytecode', bytecode, function (_) { });

    fs.writeFileSync('output/BRC20_Prog_deploy_tx.json', JSON.stringify(
        {
            p: "brc20-prog",
            op: "deploy",
            d: deploy_tx.data
        }
    ), function (_) { });

    let pkscript = "0x00142b05d564e6a7a33c087f16e0f730d1440123799d"
    let message = "0x48656c6c6f20576f726c64" // "Hello World"
    let signature = "0x02483045022100ecf2ca796ab7dde538a26bfb09a6c487a7b3fff33f397db6a20eb9af77c0ee8c022062e67e44c8070f49c3a37f5940a8850842daf7cca35e6af61a6c7c91f1e1a1a3012102c7f12003196442943d8588e01aee840423cc54fc1521526a3b85c2b0cbd58872"

    fs.writeFileSync('output/BRC20_Prog_bip322_verify_tx.json', JSON.stringify(
        {
            p: "brc20-prog",
            op: "call",
            c: "REPLACE_THIS_WITH_CONTRACT_ADDRESS",
            d: contract_factory.interface.encodeFunctionData("verify", [pkscript, message, signature]),
        }
    ), function (_) { });

    let btc_pkscript = "0x5120e0e224cd541454519b62047aa0891ea7b81a16598556aeb83a412a0b06a20aab"
    let ticker = "0x20202020"

    fs.writeFileSync('output/BRC20_Prog_brc20_balance_tx.json', JSON.stringify(
        {
            p: "brc20-prog",
            op: "call",
            c: "REPLACE_THIS_WITH_CONTRACT_ADDRESS",
            d: contract_factory.interface.encodeFunctionData("balanceOf", [ticker, btc_pkscript]),
        }
    ), function (_) { });

    // https://mempool.space/signet/tx/d09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd
    let btc_tx_id = "0xd09d26752d0a33d1bdb0213cf36819635d1258a7e4fcbe669e12bc7dab8cecdd"

    fs.writeFileSync('output/BRC20_Prog_btc_tx_details_tx.json', JSON.stringify(
        {
            p: "brc20-prog",
            op: "call",
            c: "REPLACE_THIS_WITH_CONTRACT_ADDRESS",
            d: contract_factory.interface.encodeFunctionData("getTxDetails", [btc_tx_id]),
        }
    ), function (_) { });

    let btc_vout = 0
    let btc_sat = 250000

    fs.writeFileSync('output/BRC20_Prog_btc_last_sat_loc_tx.json', JSON.stringify(
        {
            p: "brc20-prog",
            op: "call",
            c: "REPLACE_THIS_WITH_CONTRACT_ADDRESS",
            d: contract_factory.interface.encodeFunctionData("getLastSatLocation", [btc_tx_id, btc_vout, btc_sat]),
        }
    ), function (_) { });

    let lock_block_count = 100

    fs.writeFileSync('output/BRC20_Prog_btc_locked_pkscript_tx.json', JSON.stringify(
        {
            p: "brc20-prog",
            op: "call",
            c: "REPLACE_THIS_WITH_CONTRACT_ADDRESS",
            d: contract_factory.interface.encodeFunctionData("getLockedPkscript", [btc_pkscript, lock_block_count]),
        }
    ), function (_) { });

    fs.writeFileSync('output/BRC20_Prog_get_sha_256_tx.json', JSON.stringify(
        {
            p: "brc20-prog",
            op: "call",
            c: "REPLACE_THIS_WITH_CONTRACT_ADDRESS",
            d: contract_factory.interface.encodeFunctionData("getSha256", [message]),
        }
    ), function (_) { });

    fs.writeFileSync('output/BRC20_Prog_get_random_number_tx.json', JSON.stringify(
        {
            p: "brc20-prog",
            op: "call",
            c: "REPLACE_THIS_WITH_CONTRACT_ADDRESS",
            d: contract_factory.interface.encodeFunctionData("getRandomNumber", []),
        }
    ), function (_) { });

    // copy everything from output to integration_test folder
    fs.cpSync("output", "../../integration_test/contracts/brc20_prog_helper", { recursive: true });
}

main()
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
    let BobCoin_sol = fs.readFileSync("./BobCoin.sol", "utf8");

    let input = {
        language: 'Solidity',
        sources: {
            'BobCoin.sol': { content: BobCoin_sol },
        },
        settings: {
            evmVersion: "cancun",
            outputSelection: {
                '*': {
                    '*': ['*']
                }
            }
        }
    };

    let compiled = JSON.parse(solc_fixed.compile(JSON.stringify(input), { import: findImports }))
    let contract = compiled.contracts['BobCoin.sol']["BobCoin"]
    let bytecode = contract.evm.bytecode.object
    let abi = contract.abi

    let contract_factory = new ethers.ContractFactory(abi, bytecode, null)
    let deploy_tx = await contract_factory.getDeployTransaction("BobCoin", "BC")

    fs.mkdirSync('output', { recursive: true });

    fs.writeFile('output/BobCoin.abi', JSON.stringify(abi, null, 4), function (_) { });
    fs.writeFile('output/BobCoin.bytecode', bytecode, function (_) { });

    fs.writeFile('output/BobCoin_deploy_tx.json', JSON.stringify(
        {
            p: "brc20-prog",
            op: "deploy",
            d: deploy_tx.data
        }
    ), function (_) { });
}

main()
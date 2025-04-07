var solc_fixed = require('solc_0_8_28');
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
    let BRC20Controller_sol = fs.readFileSync("./BRC20_Controller.sol", "utf8");

    let input = {
        language: 'Solidity',
        sources: {
            'BRC20_Controller.sol': { content: BRC20Controller_sol },
        },
        settings: {
            optimizer: {
                enabled: true,
                runs: 10000,
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
    let contract = compiled.contracts['BRC20_Controller.sol']["BRC20_Controller"]
    let bytecode = contract.evm.bytecode.object
    let abi = contract.abi

    let contract_factory = new ethers.ContractFactory(abi, bytecode, null)
    let deploy_tx = await contract_factory.getDeployTransaction()

    fs.mkdirSync('output', { recursive: true });

    fs.writeFileSync('../BRC20_Controller.json', JSON.stringify(JSON.parse(contract["metadata"]), null, 4), function (_) { });
    fs.writeFileSync('../BRC20_Controller.abi', JSON.stringify(abi, null, 4), function (_) { });
    fs.writeFileSync('../BRC20_Controller.bin', bytecode, function (_) { });
    fs.writeFileSync('../BRC20_Controller_deploy.bytecode', deploy_tx.data, function (_) { });
}

main()
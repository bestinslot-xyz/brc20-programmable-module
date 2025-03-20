var solc_fixed = require('solc_0_8_24');
var ethers = require('ethers');
var fs = require('fs');

async function main() {
    let Simple_sol = fs.readFileSync("./Simple.sol", "utf8");

    let input = {
        language: 'Solidity',
        sources: {
            'Simple.sol': { content: Simple_sol },
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

    let compiled = JSON.parse(solc_fixed.compile(JSON.stringify(input)))
    let contract = compiled.contracts['Simple.sol']["Simple"]
    let bytecode = contract.evm.bytecode.object
    let abi = contract.abi

    let contract_factory = new ethers.ContractFactory(abi, bytecode, null)
    let deploy_tx = await contract_factory.getDeployTransaction([])
    let interface = contract_factory.interface

    fs.mkdirSync('output', { recursive: true });

    fs.writeFile('output/Simple.abi', JSON.stringify(abi, null, 4), function (_) { });
    fs.writeFile('output/Simple.bytecode', bytecode, function (_) { });

    fs.writeFile('output/Simple_deploy_tx.json', JSON.stringify(
        {
            p: "brc20-prog",
            op: "deploy",
            d: deploy_tx.data
        }
    ), function (_) { });

    fs.writeFile('output/Simple_setValue_tx.json', JSON.stringify(
        {
            p: "brc20-prog",
            op: "call",
            c: "REPLACE_THIS_WITH_CONTRACT_ADDRESS",
            d: interface.encodeFunctionData("setValue", [42]),
        }
    ), function (_) { });

    fs.writeFile('output/Simple_getValue_tx.json', JSON.stringify(
        {
            p: "brc20-prog",
            op: "call",
            c: "REPLACE_THIS_WITH_CONTRACT_ADDRESS",
            d: interface.encodeFunctionData("getValue", []),
        }
    ), function (_) { });
}

main()
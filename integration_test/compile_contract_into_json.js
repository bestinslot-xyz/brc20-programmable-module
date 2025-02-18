import solc_fixed from 'solc_0_8_24';
import fs from 'fs';
import { ethers } from 'ethers';

let indexer_addr = "0x0000000000000000000000000000000000003Ca6";

let BRC20_Controller_sol = fs.readFileSync("./contracts/BRC20_Controller.sol", "utf8");
let IBRC20_Controller_sol = fs.readFileSync("./contracts/IBRC20_Controller.sol", "utf8");
let Context_sol = fs.readFileSync("./contracts/utils/Context.sol", "utf8");
let draft_IBRC6093_sol = fs.readFileSync("./contracts/interfaces/draft-IBRC6093.sol", "utf8");
let Ownable_sol = fs.readFileSync("./contracts/access/Ownable.sol", "utf8");
let input = {
    language: 'Solidity',
    sources: {
        'BRC20_Controller.sol': { content: BRC20_Controller_sol },
        'IBRC20_Controller.sol': { content: IBRC20_Controller_sol },
        'utils/Context.sol': { content: Context_sol },
        'interfaces/draft-IBRC6093.sol': { content: draft_IBRC6093_sol },
        'access/Ownable.sol': { content: Ownable_sol },
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
let contract = compiled.contracts['BRC20_Controller.sol']["BRC20_Controller"]
let bytecode = contract.evm.bytecode.object
let abi = contract.abi

let contract_factory = new ethers.ContractFactory(abi, bytecode, null)
let deploy_tx = await contract_factory.getDeployTransaction([])

fs.writeFile("./contracts/BRC20_Controller.json", JSON.stringify({
    from: indexer_addr,
    to: null,
    data: deploy_tx.data,
}), (err) => {
    if (err) {
        console.error(err)
    }
});

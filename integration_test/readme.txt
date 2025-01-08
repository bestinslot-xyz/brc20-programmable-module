order:
- process deposit & withdraw in sync with other brc20 operations and save successful operations to be sent to evm
  - while checking for withdraw, check evm balance + deposits in this block before withdraw
- after all regular brc20 operations are processed, send all evm operations in this order
  - send deposits and withdrawals in btc order
  - send remaining finance module operations in btc order
- if remaining operations change brc20 controller balances, change modified balances in main-db (optional)

inscrs

<type>: eth standard types and btc_address

arg:
{
    "t": <type>,
    "v": <value>
}

functionCall:
{
    "p": "brc20-prog",
    "op": "call",
    "c": <contract_addr>,
    "f": <function_name>,
    "a": [<arg>, <arg>, ...]
}

deployContract:
{
    "p": "brc20-prog",
    "op": "deploy",
    "sc": <flattened_contract_source_code>,
    "cls": <deploy_class>,
    "a": [<arg>, <arg>, ...]
}
{
    "p": "brc20-prog",
    "op": "deploy",
    "d": <bytecode + constructor_args>
}
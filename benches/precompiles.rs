use brc20_prog::types::{AddressED, EthCall};
use brc20_prog::Brc20ProgApiClient;
use criterion::{criterion_group, criterion_main, Criterion};
use test_utils::{
    load_file_as_eth_bytes, print_gas_per_call, spawn_balance_server, spawn_test_server,
};
use tokio::runtime::Runtime;

fn bip322_fn(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (server, client) = rt.block_on(async { spawn_test_server(Default::default()).await });

    let from_address: Option<AddressED> = Some([1u8; 20].into());

    let mut bip322_precompile_address = [0u8; 20];
    bip322_precompile_address[19] = 0xfe; // BIP322 precompile address
    let to_address: Option<AddressED> = Some(bip322_precompile_address.into());

    let call_tx_data = load_file_as_eth_bytes("bip322_verify_call_tx_data").unwrap();

    let eth_call = EthCall::new(
        from_address.clone(),
        to_address.clone(),
        call_tx_data.clone(),
    );

    c.bench_function("Call bip322_verify", |b| {
        b.iter(|| {
            rt.block_on(async {
                client.eth_call(eth_call.clone(), None).await.unwrap();
            });
        })
    });

    print_gas_per_call(&rt, &client, eth_call.clone());

    server.stop().unwrap();
}

fn get_locked_pkscript_fn(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (server, client) = rt.block_on(async { spawn_test_server(Default::default()).await });

    let from_address: Option<AddressED> = Some([1u8; 20].into());

    let mut btc_locked_pkscript_precompile = [0; 20];
    btc_locked_pkscript_precompile[19] = 0xfb;

    let to_address: Option<AddressED> = Some(btc_locked_pkscript_precompile.into());

    let call_tx_data = load_file_as_eth_bytes("btc_get_locked_pkscript_call_tx_data").unwrap();

    let eth_call = EthCall::new(
        from_address.clone(),
        to_address.clone(),
        call_tx_data.clone(),
    );

    c.bench_function("Call btc_get_locked_pkscript", |b| {
        b.iter(|| {
            rt.block_on(async {
                client.eth_call(eth_call.clone(), None).await.unwrap();
            });
        })
    });

    print_gas_per_call(&rt, &client, eth_call.clone());

    server.stop().unwrap();
}

fn get_brc20_balance_fn(c: &mut Criterion) {
    spawn_balance_server(); // Start the balance server in a separate thread

    let rt = Runtime::new().unwrap();
    let (server, client) = rt.block_on(async { spawn_test_server(Default::default()).await });

    let from_address: Option<AddressED> = Some([1u8; 20].into());

    let mut brc20_balance_precompile = [0; 20];
    brc20_balance_precompile[19] = 0xff;

    let to_address: Option<AddressED> = Some(brc20_balance_precompile.into());

    let call_tx_data = load_file_as_eth_bytes("get_brc20_balance_call_tx_data").unwrap();

    let eth_call = EthCall::new(
        from_address.clone(),
        to_address.clone(),
        call_tx_data.clone(),
    );

    c.bench_function("Call get_brc20_balance", |b| {
        b.iter(|| {
            rt.block_on(async {
                client.eth_call(eth_call.clone(), None).await.unwrap();
            });
        })
    });

    print_gas_per_call(&rt, &client, eth_call.clone());

    server.stop().unwrap();
}

criterion_group!(
    precompiles,
    bip322_fn,
    get_brc20_balance_fn,
    get_locked_pkscript_fn
);
criterion_main!(precompiles);

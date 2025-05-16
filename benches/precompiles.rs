use brc20_prog::types::{AddressED, EthCall};
use brc20_prog::Brc20ProgApiClient;
use criterion::{criterion_group, criterion_main, Criterion};
use jsonrpsee::http_client::HttpClient;
use test_utils::{load_file_as_bytes, spawn_balance_server, spawn_test_server};
use tokio::runtime::Runtime;

fn print_gas_per_call(rt: &Runtime, client: &HttpClient, eth_call: EthCall) -> u64 {
    let gas_per_call = u64::from_str_radix(
        rt.block_on(async {
            client
                .eth_estimate_gas(eth_call.clone(), None)
                .await
                .unwrap()
        })
        .trim_start_matches("0x"),
        16,
    )
    .unwrap();

    println!("Gas per call: {}\n", gas_per_call);
    gas_per_call
}

fn bip322_fn(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (server, client) = rt.block_on(async { spawn_test_server(Default::default()).await });

    let from_address: Option<AddressED> = Some([1u8; 20].into());

    let mut bip322_precompile_address = [0u8; 20];
    bip322_precompile_address[19] = 0xfe; // BIP322 precompile address
    let to_address: Option<AddressED> = Some(bip322_precompile_address.into());

    let call_tx_data = load_file_as_bytes("bip322_verify_call_tx_data").unwrap();

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

fn last_sat_loc_signet_precompile_fn(c: &mut Criterion) {
    dotenvy::from_filename_override("env.signet.sample").ok();
    let rt = Runtime::new().unwrap();
    let (server, client) = rt.block_on(async { spawn_test_server(Default::default()).await });
    let from_address: Option<AddressED> = Some([1u8; 20].into());

    let mut last_sat_loc_precompile_address = [0u8; 20];
    last_sat_loc_precompile_address[19] = 0xfc; // Last sat loc precompile address
    let to_address: Option<AddressED> = Some(last_sat_loc_precompile_address.into());

    let call_tx_data = load_file_as_bytes("btc_last_sat_loc_signet_call_tx_data").unwrap();

    let eth_call = EthCall::new(
        from_address.clone(),
        to_address.clone(),
        call_tx_data.clone(),
    );

    rt.block_on(async {
        client.brc20_mine(300000, 42).await.unwrap();
    });

    c.bench_function("Call last_sat_location on signet", |b| {
        b.iter(|| {
            rt.block_on(async {
                client.eth_call(eth_call.clone(), None).await.unwrap();
            });
        })
    });

    print_gas_per_call(&rt, &client, eth_call.clone());

    server.stop().unwrap();
}

fn get_tx_details_mainnet_fn(c: &mut Criterion) {
    dotenvy::dotenv_override().ok();
    let rt = Runtime::new().unwrap();
    let (server, client) = rt.block_on(async { spawn_test_server(Default::default()).await });
    let from_address: Option<AddressED> = Some([1u8; 20].into());

    let mut get_tx_details_precompile_address = [0u8; 20];
    get_tx_details_precompile_address[19] = 0xfd; // Get tx details precompile address
    let to_address: Option<AddressED> = Some(get_tx_details_precompile_address.into());

    let call_tx_data = load_file_as_bytes("btc_get_tx_details_mainnet_call_tx_data").unwrap();

    let eth_call = EthCall::new(
        from_address.clone(),
        to_address.clone(),
        call_tx_data.clone(),
    );

    rt.block_on(async {
        client.brc20_mine(300000, 42).await.unwrap();
    });

    c.bench_function("Call btc_get_tx_details on mainnet", |b| {
        b.iter(|| {
            rt.block_on(async {
                client.eth_call(eth_call.clone(), None).await.unwrap();
            });
        })
    });

    print_gas_per_call(&rt, &client, eth_call.clone());

    server.stop().unwrap();
}

fn get_tx_details_signet_fn(c: &mut Criterion) {
    dotenvy::from_filename_override("env.signet.sample").ok();
    let rt = Runtime::new().unwrap();
    let (server, client) = rt.block_on(async { spawn_test_server(Default::default()).await });
    let from_address: Option<AddressED> = Some([1u8; 20].into());

    let mut get_tx_details_precompile_address = [0u8; 20];
    get_tx_details_precompile_address[19] = 0xfd; // Get tx details precompile address
    let to_address: Option<AddressED> = Some(get_tx_details_precompile_address.into());

    let call_tx_data = load_file_as_bytes("btc_get_tx_details_signet_call_tx_data").unwrap();

    let eth_call = EthCall::new(
        from_address.clone(),
        to_address.clone(),
        call_tx_data.clone(),
    );

    rt.block_on(async {
        client.brc20_mine(300000, 42).await.unwrap();
    });

    c.bench_function("Call btc_get_tx_details on signet", |b| {
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

    let call_tx_data = load_file_as_bytes("btc_get_locked_pkscript_call_tx_data").unwrap();

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

    let call_tx_data = load_file_as_bytes("get_brc20_balance_call_tx_data").unwrap();

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
    last_sat_loc_signet_precompile_fn,
    get_tx_details_mainnet_fn,
    get_tx_details_signet_fn,
    get_brc20_balance_fn,
    get_locked_pkscript_fn
);
criterion_main!(precompiles);

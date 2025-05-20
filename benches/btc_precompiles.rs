use brc20_prog::types::{AddressED, EthCall};
use brc20_prog::Brc20ProgApiClient;
use criterion::{criterion_group, criterion_main, Criterion};
use test_utils::{load_file_as_eth_bytes, print_gas_per_call, spawn_test_server};
use tokio::runtime::Runtime;

fn get_tx_details_mainnet_case_1_fn(c: &mut Criterion) {
    dotenvy::from_filename_override(".env.mainnet").ok();
    let rt = Runtime::new().unwrap();
    let (server, client) = rt.block_on(async { spawn_test_server(Default::default()).await });
    let from_address: Option<AddressED> = Some([1u8; 20].into());

    let mut get_tx_details_precompile_address = [0u8; 20];
    get_tx_details_precompile_address[19] = 0xfd; // Get tx details precompile address
    let to_address: Option<AddressED> = Some(get_tx_details_precompile_address.into());

    let call_tx_data = load_file_as_eth_bytes("btc_get_tx_details_mainnet_call_1_tx_data").unwrap();

    let eth_call = EthCall::new(
        from_address.clone(),
        to_address.clone(),
        call_tx_data.clone(),
    );

    rt.block_on(async {
        client.brc20_mine(300000, 42).await.unwrap();
    });

    c.bench_function("Call btc_get_tx_details on mainnet (1 input)", |b| {
        b.iter(|| {
            rt.block_on(async {
                client.eth_call(eth_call.clone(), None).await.unwrap();
            });
        })
    });

    print_gas_per_call(&rt, &client, eth_call.clone());

    server.stop().unwrap();
}

fn get_tx_details_mainnet_case_2_fn(c: &mut Criterion) {
    dotenvy::from_filename_override(".env.mainnet").ok();
    let rt = Runtime::new().unwrap();
    let (server, client) = rt.block_on(async { spawn_test_server(Default::default()).await });
    let from_address: Option<AddressED> = Some([1u8; 20].into());

    let mut get_tx_details_precompile_address = [0u8; 20];
    get_tx_details_precompile_address[19] = 0xfd; // Get tx details precompile address
    let to_address: Option<AddressED> = Some(get_tx_details_precompile_address.into());

    let call_tx_data = load_file_as_eth_bytes("btc_get_tx_details_mainnet_call_2_tx_data").unwrap();

    let eth_call = EthCall::new(
        from_address.clone(),
        to_address.clone(),
        call_tx_data.clone(),
    );

    rt.block_on(async {
        client.brc20_mine(300000, 42).await.unwrap();
    });

    c.bench_function("Call btc_get_tx_details on mainnet (3 inputs)", |b| {
        b.iter(|| {
            rt.block_on(async {
                client.eth_call(eth_call.clone(), None).await.unwrap();
            });
        })
    });

    print_gas_per_call(&rt, &client, eth_call.clone());

    server.stop().unwrap();
}

fn get_tx_details_mainnet_case_3_fn(c: &mut Criterion) {
    dotenvy::from_filename_override(".env.mainnet").ok();
    let rt = Runtime::new().unwrap();
    let (server, client) = rt.block_on(async { spawn_test_server(Default::default()).await });
    let from_address: Option<AddressED> = Some([1u8; 20].into());

    let mut get_tx_details_precompile_address = [0u8; 20];
    get_tx_details_precompile_address[19] = 0xfd; // Get tx details precompile address
    let to_address: Option<AddressED> = Some(get_tx_details_precompile_address.into());

    let call_tx_data = load_file_as_eth_bytes("btc_get_tx_details_mainnet_call_3_tx_data").unwrap();

    let eth_call = EthCall::new(
        from_address.clone(),
        to_address.clone(),
        call_tx_data.clone(),
    );

    rt.block_on(async {
        client.brc20_mine(300000, 42).await.unwrap();
    });

    c.bench_function("Call btc_get_tx_details on mainnet (5 inputs)", |b| {
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
    dotenvy::from_filename_override(".env.signet").ok();
    let rt = Runtime::new().unwrap();
    let (server, client) = rt.block_on(async { spawn_test_server(Default::default()).await });
    let from_address: Option<AddressED> = Some([1u8; 20].into());

    let mut get_tx_details_precompile_address = [0u8; 20];
    get_tx_details_precompile_address[19] = 0xfd; // Get tx details precompile address
    let to_address: Option<AddressED> = Some(get_tx_details_precompile_address.into());

    let call_tx_data = load_file_as_eth_bytes("btc_get_tx_details_signet_call_tx_data").unwrap();

    let eth_call = EthCall::new(
        from_address.clone(),
        to_address.clone(),
        call_tx_data.clone(),
    );

    rt.block_on(async {
        client.brc20_mine(300000, 42).await.unwrap();
    });

    c.bench_function("Call btc_get_tx_details on signet (1 input)", |b| {
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
    dotenvy::from_filename_override(".env.signet").ok();
    let rt = Runtime::new().unwrap();
    let (server, client) = rt.block_on(async { spawn_test_server(Default::default()).await });
    let from_address: Option<AddressED> = Some([1u8; 20].into());

    let mut last_sat_loc_precompile_address = [0u8; 20];
    last_sat_loc_precompile_address[19] = 0xfc; // Last sat loc precompile address
    let to_address: Option<AddressED> = Some(last_sat_loc_precompile_address.into());

    let call_tx_data = load_file_as_eth_bytes("btc_last_sat_loc_signet_call_tx_data").unwrap();

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

criterion_group!(
    btc_precompiles,
    get_tx_details_mainnet_case_1_fn,
    get_tx_details_mainnet_case_2_fn,
    get_tx_details_mainnet_case_3_fn,
    get_tx_details_signet_fn,
    last_sat_loc_signet_precompile_fn,
);
criterion_main!(btc_precompiles);

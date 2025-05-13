use brc20_prog::types::{AddressED, EthCall};
use brc20_prog::Brc20ProgApiClient;
use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

fn bip322_fn(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (server, client) =
        rt.block_on(async { test_utils::spawn_test_server(Default::default()).await });

    let from_address: Option<AddressED> = Some([1u8; 20].into());

    let mut bip322_precompile_address = [0u8; 20];
    bip322_precompile_address[19] = 0xfe; // BIP322 precompile address
    let to_address: Option<AddressED> = Some(bip322_precompile_address.into());

    let call_tx_data = test_utils::load_file_as_bytes("bip322_verify_call_tx_data").unwrap();

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

    server.stop().unwrap();
}

criterion_group!(precompiles, bip322_fn);
criterion_main!(precompiles);

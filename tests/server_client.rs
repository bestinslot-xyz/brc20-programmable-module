use brc20_prog::Brc20ProgApiClient;

mod common;
use common::spawn_test_server;

#[tokio::test]
async fn test_version() {
    let (server, client) = spawn_test_server(Default::default()).await;

    assert_eq!(
        env!("CARGO_PKG_VERSION"),
        client.brc20_version().await.unwrap()
    );

    server.stop().unwrap();
}

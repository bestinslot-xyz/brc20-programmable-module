use brc20_prog::{Brc20Prog, Brc20ProgApiClient, Brc20ProgApiServer, Brc20ProgConfig};
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use tempfile::TempDir;
use test_utils::get_free_port;

struct Embedded {
    prog: Brc20Prog,
    public_address: String,
    _db_path: TempDir,
}

fn open_embedded() -> Embedded {
    let db_path = TempDir::new().unwrap();
    let public_address = format!("127.0.0.1:{}", get_free_port());
    let prog = Brc20Prog::open(Brc20ProgConfig {
        db_path: db_path.path().to_str().unwrap().to_string(),
        brc20_prog_rpc_server_url: public_address.clone(),
        fail_on_bitcoin_rpc_error: false,
        ..Default::default()
    })
    .expect("Failed to open brc20-prog");

    Embedded {
        prog,
        public_address,
        _db_path: db_path,
    }
}

fn client(address: &str) -> HttpClient {
    HttpClientBuilder::default()
        .build(format!("http://{}", address))
        .expect("Failed to create client")
}

#[tokio::test]
async fn test_in_process_calls_drive_the_engine() {
    let embedded = open_embedded();

    embedded.prog.brc20_mine(2, 0).await.unwrap();
    assert_eq!(embedded.prog.eth_block_number().await.unwrap(), "0x1");
}

#[tokio::test]
async fn test_public_server_reads_in_process_state() {
    let embedded = open_embedded();
    let handle = embedded.prog.start_public_rpc_server().await.unwrap();
    let public = client(&embedded.public_address);

    embedded.prog.brc20_mine(1, 0).await.unwrap();
    assert_eq!(public.eth_block_number().await.unwrap(), "0x0");

    // A block mined in process after the server started is visible to it
    embedded.prog.brc20_mine(1, 0).await.unwrap();
    assert_eq!(public.eth_block_number().await.unwrap(), "0x1");

    handle.stop().unwrap();
}

#[tokio::test]
async fn test_public_server_rejects_indexer_methods() {
    let embedded = open_embedded();
    let handle = embedded.prog.start_public_rpc_server().await.unwrap();
    let public = client(&embedded.public_address);

    assert!(public.brc20_mine(1, 0).await.is_err());
    assert!(public.brc20_commit_to_database().await.is_err());
    assert!(public.eth_block_number().await.is_ok());

    handle.stop().unwrap();
}

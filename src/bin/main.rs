use std::error::Error;
use std::process::exit;

use brc20_prog::Brc20ProgConfig;
use tracing::error;

pub struct Args {
    pub log_level: tracing::Level, // passed with -l or --log-level
    pub log_file: Option<String>,  // passed with -f or --log-file
}

/// Parses the command line arguments and returns an Args struct
/// containing the log level and log file.
/// -f and -l are used to set the log file and log level respectively.
fn parse_args() -> Args {
    let args = std::env::args().collect::<Vec<_>>();
    let mut log_level = tracing::Level::WARN;
    let mut log_file = None;

    for i in 1..args.len() {
        match args[i].as_str() {
            "-l" | "--log-level" => {
                if i + 1 < args.len() {
                    log_level = args[i + 1].parse().unwrap_or(tracing::Level::WARN);
                }
            }
            "-f" | "--log-file" => {
                if i + 1 < args.len() {
                    log_file = Some(args[i + 1].clone());
                }
            }
            "-h" | "--help" => {
                println!("Usage: brc20_prog [OPTIONS]");
                println!("Options:");
                println!("  -l, --log-level <level>   Set the log level (default: WARN)");
                println!("  -f, --log-file <file>     Set the log file");
                println!("  -h, --help                Show this help message");
                std::process::exit(0);
            }
            _ => {}
        }
    }
    Args {
        log_level,
        log_file,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let args = parse_args();
    if let Some(log_file) = args.log_file {
        tracing_subscriber::fmt()
            .with_ansi(false)
            .with_max_level(args.log_level)
            .with_writer(std::fs::File::create(log_file)?)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(args.log_level)
            .init();
    }

    println!("BRC20 Prog v{}", env!("CARGO_PKG_VERSION"));

    let server = brc20_prog::start(Brc20ProgConfig::from_env().into()).await;
    let Ok(server_handle) = server else {
        error!("Error starting server: {}", server.unwrap_err());
        exit(1);
    };

    println!(
        "Started JSON-RPC server on {}",
        std::env::var("BRC20_PROG_RPC_SERVER_URL").unwrap_or("None".to_string())
    );
    server_handle.stopped().await;
    Ok(())
}

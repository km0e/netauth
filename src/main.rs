mod config;
mod unicom;
use anyhow::Result;
use clap::{Parser, Subcommand};
use config::Config;
use tracing::{info, warn};
use xcfg::XCfg;

fn default_config() -> String {
    home::home_dir()
        .expect("can't find home directory")
        .join(".config/netauth/config")
        .to_str()
        .expect("can't convert path to string")
        .to_string()
}

#[derive(Parser, Debug)]
#[command(version = "0.1")]
struct Cli {
    #[command(subcommand)]
    command: Service,
    #[arg(short, long, default_value_t = default_config())]
    config: String,
}

#[derive(Subcommand, Debug)]
enum Service {
    #[command(subcommand)]
    Unicom(unicom::Unicom),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Cli::parse();
    let config = Config::load(&args.config)
        .inspect_err(|e| {
            warn!("failed to load config: {}", e);
        })
        .ok()
        .map(|config| config.into_inner());
    match args.command {
        Service::Unicom(arg) => {
            unicom::dispatch(arg, config.and_then(|config| config.unicom)).await?;
        }
    }
    info!("operation completed");
    Ok(())
}

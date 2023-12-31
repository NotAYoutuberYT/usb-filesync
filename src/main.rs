use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use anyhow::Ok;
use blake3::Hasher;
use clap::Parser;
use sysinfo::System;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::config::Config;

mod config;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    /// User-provided salt added to system information pre-hash
    #[arg(short, long)]
    salt: Option<String>,

    /// Auto-confirm new configuration profile
    #[arg(short = 'y', long)]
    confirm_new: bool,
}

fn main() -> anyhow::Result<()> {
    // send debug info only in debug mode
    if cfg!(debug_assertions) {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .finish();

        tracing::subscriber::set_global_default(subscriber)?;
    }

    let args = Args::parse();

    let mut system = System::new();
    system.refresh_cpu();
    system.refresh_memory();

    let mut hasher = Hasher::new();
    hasher.update(args.salt.unwrap_or_default().as_bytes());
    hasher.update(System::name().unwrap_or_default().as_bytes());
    hasher.update(System::host_name().unwrap_or_default().as_bytes());
    hasher.update(&system.total_memory().to_be_bytes());
    hasher.update(
        &system
            .physical_core_count()
            .unwrap_or_default()
            .to_be_bytes(),
    );
    let system_id = hasher.finalize().to_string();
    info!("constructed system id {system_id}");

    let config_path_string = format!("system-configs/{system_id}.json");
    let config_path = Path::new(config_path_string.as_str());

    let mut config = Config::default();
    if !config_path.exists() && !args.confirm_new {
        loop {
            print!("Would you like to create a new system with the default configuration? [y/n] ");
            io::stdout().flush()?;

            let mut input = String::default();
            io::stdin().read_line(&mut input)?;

            match input.trim() {
                "y" => break,
                "n" => return Ok(()),
                _ => (),
            }
        }

        fs::write(config_path, serde_json::to_string(&Config::default())?)?;
        info!("created new config file at {config_path_string}")
    } else if !config_path.exists() && args.confirm_new {
        fs::write(config_path, serde_json::to_string(&Config::default())?)?;
        info!("created new config file at {config_path_string}")
    } else {
        let config_contents = fs::read_to_string(config_path)?;
        config = serde_json::from_str(&config_contents)?;
    }

    Ok(())
}

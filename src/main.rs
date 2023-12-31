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

use crate::{
    config::Config,
    system_id::{SystemAndSalt, SystemId},
};

mod config;
mod system_id;

#[derive(Parser, Debug, Clone)]
#[command(author, version)]
struct Args {
    /// User-provided salt added to system information pre-hash
    #[arg(short, long)]
    salt: Option<String>,

    /// Auto-confirm new configuration profile
    #[arg(short = 'y', long)]
    confirm_new: bool,
}

/// Repeatedly asks for confirmation using the given msg until the user enters a valid y/n input
fn get_user_confirmation(msg: &str) -> anyhow::Result<bool> {
    loop {
        print!("{msg}");
        io::stdout().flush()?;

        let mut input = String::default();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "y" => return Ok(true),
            "n" => return Ok(false),
            _ => (),
        }
    }
}

fn get_system_id(args: &Args) -> SystemId {
    let system = System::new();
    let mut system_and_salt = SystemAndSalt::new(system, (*args).clone().salt.unwrap_or_default());
    let mut hasher = Hasher::new();

    SystemId::new_by_hashing(&mut system_and_salt, &mut hasher)
}

/// Attempts to read a config file, and seeks confirmation to create one if none exists for
/// the current system id. Returns any i/o errors, and a None is returned when the user
/// doesn't give confirmation to create a new config file.
fn manage_config_file(system_id: &SystemId, args: &Args) -> anyhow::Result<Option<Config>> {
    let config_path_string = format!("system-configs/{}.json", system_id.id);
    let config_path = Path::new(config_path_string.as_str());
    let needs_to_make_file = !config_path.exists();

    let mut can_make_file = args.confirm_new;
    if needs_to_make_file && !can_make_file {
        can_make_file = get_user_confirmation(
            "Would you like to create a new system with the default configuration? [y/n] ",
        )?;
    }

    if needs_to_make_file && !can_make_file {
        return Ok(None);
    }

    if needs_to_make_file {
        fs::write(config_path, serde_json::to_string(&Config::default())?)?;
        info!("created new config file at {config_path_string}");

        Ok(Some(Config::default()))
    } else {
        let config_contents = fs::read_to_string(config_path)?;
        Ok(serde_json::from_str(&config_contents)?)
    }
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

    let system_id = get_system_id(&args);
    info!("constructed system id {}", system_id.id);

    let config = match manage_config_file(&system_id, &args)? {
        Some(config) => config,
        None => return Ok(()),
    };

    println!("{:?}", config);

    Ok(())
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    #[ctor::ctor]
    fn init() {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("failed to set global default tracing subscriber");
    }
}

// SPDX-License-Identifier: GPL-3.0

#[cfg(any(feature = "parachain", feature = "contract"))]
mod commands;
#[cfg(any(feature = "parachain", feature = "contract"))]
mod style;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use commands::*;
use pop_telemetry::{config_file_path, record_cli_command, record_cli_used, Telemetry};
use serde_json::{json, Value};
use std::{env::args, fs::create_dir_all, path::PathBuf};
use tokio::spawn;

#[derive(Parser)]
#[command(author, version, about, styles=style::get_styles())]
pub struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
#[command(subcommand_required = true)]
enum Commands {
	/// Generate a new parachain, pallet or smart contract.
	#[clap(alias = "n")]
	New(commands::new::NewArgs),
	/// Build a parachain or smart contract.
	#[clap(alias = "b")]
	Build(commands::build::BuildArgs),
	/// Call a smart contract.
	#[clap(alias = "c")]
	#[cfg(feature = "contract")]
	Call(commands::call::CallArgs),
	/// Deploy a parachain or smart contract.
	#[clap(alias = "u")]
	Up(commands::up::UpArgs),
	/// Test a smart contract.
	#[clap(alias = "t")]
	#[cfg(feature = "contract")]
	Test(commands::test::TestArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
	#[cfg(feature = "no_telemetry")]
	let maybe_tel = None;
	#[cfg(not(feature = "no_telemetry"))]
	let maybe_tel = init().unwrap_or(None);

	let args: Vec<_> = args().collect();

	// Handle for await not used here as telemetry should complete before any of the commands do.
	// Sends a generic ping saying the CLI was used.
	if let Some(tel) = maybe_tel.clone() {
		spawn(record_cli_used(tel));
	}

	let cli = Cli::parse();
	let res = match cli.command {
		Commands::New(args) => match args.command {
			#[cfg(feature = "parachain")]
			new::NewCommands::Parachain(cmd) => match cmd.execute().await {
				Ok(template) => {
					// telemetry should never cause a panic or early exit
					Ok(json!({template.provider().unwrap_or("provider-missing"): template.name()}))
				},
				Err(e) => Err(e),
			},
			#[cfg(feature = "parachain")]
			new::NewCommands::Pallet(cmd) => {
				// When more contract selections are added the tel data will likely need to go deeper in the stack
				cmd.execute().await.map(|_| json!("template"))
			},
			#[cfg(feature = "contract")]
			new::NewCommands::Contract(cmd) => {
				// When more contract selections are added, the tel data will likely need to go deeper in the stack
				cmd.execute().await.map(|_| json!("default"))
			},
		},
		Commands::Build(args) => match &args.command {
			#[cfg(feature = "parachain")]
			build::BuildCommands::Parachain(cmd) => cmd.execute().map(|_| Value::Null),
			#[cfg(feature = "contract")]
			build::BuildCommands::Contract(cmd) => cmd.execute().map(|_| Value::Null),
		},
		#[cfg(feature = "contract")]
		Commands::Call(args) => match &args.command {
			call::CallCommands::Contract(cmd) => cmd.execute().await.map(|_| Value::Null),
		},
		Commands::Up(args) => match &args.command {
			#[cfg(feature = "parachain")]
			up::UpCommands::Parachain(cmd) => cmd.execute().await.map(|_| Value::Null),
			#[cfg(feature = "contract")]
			up::UpCommands::Contract(cmd) => cmd.execute().await.map(|_| Value::Null),
		},
		#[cfg(feature = "contract")]
		Commands::Test(args) => match &args.command {
			test::TestCommands::Contract(cmd) => match cmd.execute() {
				Ok(feature) => Ok(json!(feature)),
				Err(e) => Err(e),
			},
		},
	};

	if let Some(tel) = maybe_tel.clone() {
		if let Ok(sub_data) = &res {
			// Best effort to send on first try, no action if failure.
			// `args` is guaranteed to have at least 3 elements as clap will display help message if not set.
			let _ =
				record_cli_command(tel.clone(), &args[1], json!({&args[2]: sub_data.to_string()}))
					.await;
		} else {
			// `args` is guaranteed to have at least 3 elements as clap will display help message if not set.
			let _ = record_cli_command(tel, "error", json!({&args[1]: &args[2]})).await;
		}
	}

	// map result from Result<Value> to Result<()>
	res.map(|_| ())
}
#[cfg(feature = "parachain")]
fn cache() -> Result<PathBuf> {
	let cache_path = dirs::cache_dir()
		.ok_or(anyhow!("the cache directory could not be determined"))?
		.join("pop");
	// Creates pop dir if needed
	create_dir_all(cache_path.as_path())?;
	Ok(cache_path)
}

fn init_config() -> Result<PathBuf> {
	let path = config_file_path()?;
	match pop_telemetry::write_default_config(&path) {
		Ok(written) => {
			if written {
				cliclack::log::info(format!(
					"Initialized config file at {}",
					&path.to_str().unwrap()
				))?;
			}
		},
		Err(err) => {
			cliclack::log::warning(format!(
				"Unable to initialize config file, continuing... {}",
				err.to_string()
			))?;
		},
	}
	Ok(path)
}

fn init() -> Result<Option<Telemetry>> {
	env_logger::init();
	let maybe_config_path = init_config();
	// environment variable `POP_TELEMETRY_ENDPOINT` is evaluated at compile time
	let endpoint =
		option_env!("POP_TELEMETRY_ENDPOINT").unwrap_or("http://127.0.0.1:3000/api/send");

	// if config file errors set telemetry to None, otherwise Some(tel)
	Ok(maybe_config_path.ok().map(|path| Telemetry::new(endpoint.to_string(), path)))
}

#[test]
fn verify_cli() {
	// https://docs.rs/clap/latest/clap/_derive/_tutorial/chapter_4/index.html
	use clap::CommandFactory;
	Cli::command().debug_assert()
}

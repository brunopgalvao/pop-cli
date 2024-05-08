// SPDX-License-Identifier: GPL-3.0

use anyhow::Context;
use clap::Args;
use cliclack::{confirm, log, outro, outro_cancel};
use duct::cmd;
use os_info::Type;
use tokio::{fs, process::Command};

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true)]
/// Setup user environment for substrate development
pub(crate) struct InstallArgs;

impl InstallArgs {
	pub(crate) async fn execute(self) -> anyhow::Result<()> {
		if cfg!(target_os = "windows") {
			return not_supported_message();
		} else if cfg!(target_os = "macos") {
			log::info("ℹ️ Mac OS (Darwin) detected.")?;
			return install_mac().await;
		} else if cfg!(target_os = "linux") {
			let info = os_info::get();
			match info.os_type() {
				Type::Arch => {
					log::info("ℹ️ Arch Linux detected.")?;
					return install_arch().await;
				},
				Type::Debian => {
					log::info("ℹ️ Ubuntu/Debian Linux detected.")?;
					return install_debian().await;
				},
				Type::Redhat => {
					log::info("ℹ️ Redhat Linux detected.")?;
					return install_redhat().await;
				},
				_ => return not_supported_message(),
			}
		} else {
			return not_supported_message();
		}
	}
}

async fn install_mac() -> anyhow::Result<()> {
	log::info("More information about the packages to be installed here: https://docs.substrate.io/install/macos/")?;
	if !confirm("📦 Do you want to proceed with the installation of the following packages: Homebrew, protobuf, openssl, rustup, and cmake?")
		.initial_value(true)
		.interact()?
	{
		outro_cancel("🚫 You have cancelled the installation process.")?;
		return Ok(());
	}
	match cmd("which", vec!["brew"]).read() {
		Ok(output) => log::info(format!("ℹ️ Homebrew installed already at {}.", output))?,
		Err(_) => install_homebrew().await?,
	}
	cmd("brew", vec!["update"]).run()?;
	cmd("brew", vec!["install", "protobuf", "openssl", "cmake"]).run()?;
	install_rustup().await?;

	log::success("✅ Installation complete.");
	Ok(())
}

async fn install_arch() -> anyhow::Result<()> {
	log::info("More information about the packages to be installed here: https://docs.substrate.io/install/linux/")?;
	if !confirm("📦 Do you want to proceed with the installation of the following packages:cmake gcc openssl-1.0 pkgconf git clang?")
		.initial_value(true)
		.interact()?
	{
		outro_cancel("🚫 You have cancelled the installation process.")?;
		return Ok(());
	}
	cmd(
		"sudo",
		vec![
			"pacman",
			"-Syu",
			"--needed",
			"--noconfirm",
			"cmake",
			"gcc",
			"openssl-1.0",
			"pkgconf",
			"git",
			"clang",
		],
	)
	.run()?;
	cmd("export", vec!["OPENSSL_LIB_DIR='/usr/lib/openssl-1.0'"]).run()?;
	cmd("export", vec!["OPENSSL_INCLUDE_DIR='/usr/include/openssl-1.0'"]).run()?;
	install_rustup().await?;

	log::success("✅ Installation complete.");
	Ok(())
}
async fn install_debian() -> anyhow::Result<()> {
	log::info("More information about the packages to be installed here: https://docs.substrate.io/install/linux/")?;
	if !confirm("📦 Do you want to proceed with the installation of the following packages:cmake pkg-config libssl-dev git gcc build-essential git protobuf-compiler clang libclang-dev?")
		.initial_value(true)
		.interact()?
	{
		outro_cancel("🚫 You have cancelled the installation process.")?;
		return Ok(());
	}
	cmd("sudo", vec!["apt", "update"]).run()?;
	cmd(
		"sudo",
		vec![
			"apt",
			"install",
			"-y",
			"cmake",
			"pkg-config",
			"libssl-dev",
			"git",
			"gcc",
			"build-essential",
			"git",
			"protobuf-compiler",
			"clang",
			"libclang-dev",
		],
	)
	.run()?;
	install_rustup().await?;

	log::success("✅ Installation complete.");
	Ok(())
}

async fn install_redhat() -> anyhow::Result<()> {
	log::info("More information about the packages to be installed here: https://docs.substrate.io/install/linux/")?;
	if !confirm("📦 Do you want to proceed with the installation of the following packages:cmake, openssl-devel, git, protobuf, protobuf-compiler, clang, clang-devel and srustup?")
		.initial_value(true)
		.interact()?
	{
		outro_cancel("🚫 You have cancelled the installation process.")?;
		return Ok(());
	}
	cmd("sudo", vec!["yum", "update", "-y"]).run()?;
	cmd("sudo", vec!["yum", "groupinstall", "-y", "'Development Tool"]).run()?;
	cmd(
		"sudo",
		vec![
			"yum",
			"install",
			"-y",
			"cmake",
			"openssl-devel",
			"git",
			"protobuf",
			"protobuf-compiler",
			"clang",
			"clang-devel",
		],
	)
	.run()?;
	install_rustup().await?;

	log::success("✅ Installation complete.");
	Ok(())
}

fn not_supported_message() -> anyhow::Result<()> {
	log::error("This OS is not supported at present")?;
	log::warning("⚠️ Please refer to https://docs.substrate.io/install/ for setup information.")?;
	Ok(())
}

async fn install_rustup() -> anyhow::Result<()> {
	match cmd("which", vec!["rustup"]).read() {
		Ok(output) => {
			log::info(format!("ℹ️ rustup installed already at {}.", output))?;
			cmd("rustup", vec!["update"]).run()?;
			cmd("rustup", vec!["default", "stable"]).run()?;
		},
		Err(_) => {
			let mut spinner = cliclack::spinner();
			spinner.start("Installing rustup ...");
			let temp = tempfile::tempdir()?;
			let scripts_path = temp.path().join("rustup.sh");
			let client = reqwest::Client::new();
			let script = client
				.get("https://sh.rustup.rs")
				.send()
				.await
				.context("Network Error: Failed to fetch script from Github")?
				.text()
				.await?;
			fs::write(scripts_path.as_path(), script).await?;
			Command::new("sh").arg(scripts_path).status().await?;
			temp.close()?;
			outro("rustup installed!");
			cmd(
				"source",
				vec![
					"~/.cargo/env
",
				],
			)
			.run()?;
			cmd("rustup", vec!["default", "stable"]).run()?;
		},
	}
	cmd("rustup", vec!["update", "nightly"]).run()?;
	cmd("rustup", vec!["target", "add", "wasm32-unknown-unknown", "--toolchain", "nightly"])
		.run()?;

	Ok(())
}

async fn install_homebrew() -> anyhow::Result<()> {
	let temp = tempfile::tempdir()?;
	let scripts_path = temp.path().join("install.sh");
	let client = reqwest::Client::new();
	let script = client
		.get("https://raw.githubusercontent.com/Homebrew/install/master/install.sh")
		.send()
		.await
		.context("Network Error: Failed to fetch script from Github")?
		.text()
		.await?;
	fs::write(scripts_path.as_path(), script).await?;
	Command::new("bash").arg(scripts_path).status().await?;
	temp.close()?;
	Ok(())
}

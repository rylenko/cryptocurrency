#![deny(clippy::correctness)]
#![feature(error_iter)]
#![warn(
	clippy::complexity,
	clippy::pedantic,
	clippy::perf,
	clippy::style,
	clippy::suspicious
)]
#![allow(
	clippy::as_conversions,
	clippy::implicit_return,
	clippy::missing_docs_in_private_items,
	clippy::missing_errors_doc
)]

mod cli;
mod request;

use anyhow::{Context as _, Result};

/// An entrypoint that requires `args` for easy program control with
/// `crate::cli`.
pub fn launch(args: Vec<String>) -> Result<()> {
	use clap::Clap as _;

	// Basic dependencies for work
	let opts = cli::Opts::parse_from(args);
	let config = common::config::Config::load(None)
		.context("Failed to load the config.")?;
	let user = blockchain::user::User::load_or_create()
		.context("Failed to load or create a user.")?;

	let _tracing_guard =
		common::tracing::set_subscriber(config.tracing().client())
			.context("Failed to set tracing subscriber.")?;

	match opts.subcommand {
		cli::SubCommand::User(c) => match c {
			cli::UserSubCommand::Address => println!("{}", user.address()),
			cli::UserSubCommand::Balance => {
				request::balance(&config, user.address());
			}
		},
		cli::SubCommand::Blockchain(c) => match c {
			cli::BlockchainSubCommand::Len => request::blockchain_len(&config),
			cli::BlockchainSubCommand::Balance(c) => {
				request::balance(&config, &c.address);
			}
			cli::BlockchainSubCommand::Transaction(c) => {
				request::transaction(&config, &user, &c.address, c.amount)
					.context("Failed to request transaction.")?;
			}
		},
	}
	Ok(())
}

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

mod block_add_info;
mod handle;
mod helpers;

use anyhow::{Context as _, Result};

/// An entrypoint that starts a new node at the specified `address`.
pub fn launch(address: common::nodes::Node) -> Result<()> {
	// Load the config, user and a blockchain
	let config = common::config::Config::load(Some(address))
		.context("Failed to load the config.")?;
	let user = blockchain::user::User::load_or_create()
		.context("Failed to load or create a user.")?;
	let mut blockchain = blockchain::Blockchain::load_or_create(user)
		.context("Failed to load or create the blockchain.")?;
	if blockchain.is_empty().context("Failed to get blockchain len.")? {
		blockchain
			.mine_genesis_block()
			.context("Failed to mine genesis block.")?;
	}

	let _tracing_guard =
		common::tracing::set_subscriber(config.tracing().node())
			.context("Failed to set tracing subscriber.")?;

	// Leak a blockchain and the config
	let blockchain_leaked: &'static std::sync::RwLock<blockchain::Blockchain> =
		Box::leak(Box::new(std::sync::RwLock::new(blockchain)));
	let config_leaked: &'static common::config::Config =
		Box::leak(Box::new(config));

	let node = std::net::TcpListener::bind(address)
		.context("Failed to bind listener.")?;
	println!("Listening at {address}...");
	loop {
		let Ok((stream, from_address)) = node.accept() else {
			tracing::debug!("Failed to accept connection.");
			continue;
		};
		tracing::debug!("New connection from {from_address}.");
		std::thread::spawn(move || {
			if let Err(e) = handle::stream(
				stream,
				from_address,
				config_leaked,
				blockchain_leaked,
			)
			.context("Failed to handle stream.")
			{
				tracing::warn!("\n{:?}\n", e);
			}
		});
	}
}

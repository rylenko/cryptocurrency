use anyhow::{Context as _, Result};

/// The main entry processing point.
#[tracing::instrument(level = tracing::Level::DEBUG, skip(stream, blockchain))]
pub(crate) fn stream(
	mut stream: std::net::TcpStream,
	sender: common::nodes::Node,
	config: &common::config::Config,
	blockchain: &std::sync::RwLock<blockchain::Blockchain>,
) -> Result<()> {
	use common::package::{Action, Package};

	// Receive package
	let package = Package::receive(
		config,
		&mut stream,
		Some(common::set![
			Action::AddBlock,
			Action::AddTransaction,
			Action::GetBalance,
			Action::GetBlockchainLen,
			Action::GetBlocks,
			Action::GetLastBlockHash
		]),
	)
	.context("Failed to receive a package.")?;
	tracing::debug!("Received a packaeg with action {:?}.", package.action());
	match package.action() {
		Action::AddBlock => {
			add_block(blockchain, sender, &package, config)
				.context("Failed to handle block addition.")?;
		}
		Action::AddTransaction => {
			add_transaction(stream, blockchain, &package, config)
				.context("Failed to handle transaction addition.")?;
		}
		Action::GetBalance => {
			get_balance(stream, blockchain, &package, config)
				.context("Failed to handle balance getting.")?;
		}
		Action::GetBlockchainLen => {
			get_len(stream, blockchain, config)
				.context("Failed to handle len getting.")?;
		}
		Action::GetBlocks => {
			get_blocks(stream, blockchain, config)
				.context("Failed to handle blocks getting.")?;
		}
		Action::GetLastBlockHash => {
			get_last_block_hash(stream, blockchain, config)
				.context("Failed to handle last block hash getting.")?;
		}
		_ => unreachable!(),
	};
	Ok(())
}

/// Processes the user's request to get the balance of the user whose address
/// is specified in the `package.data()`.
fn get_balance(
	mut stream: std::net::TcpStream,
	blockchain: &std::sync::RwLock<blockchain::Blockchain>,
	package: &common::package::Package,
	config: &common::config::Config,
) -> Result<()> {
	let balance = blockchain
		.read()
		.unwrap()
		.get_balance(package.data())
		.context("Failed to get balance.")?;
	common::package::Package::new(
		common::package::Action::GetBalanceSuccess,
		balance.to_string(),
	)
	.send(config, &mut stream)
	.context("Failed to send package.")?;
	Ok(())
}

/// Processes a request to add a new block to the blockchain. Such a request is
/// accepted only from other nodes if mining is successful.
///
/// If the block does not fit the blockchain and the passed `blockchain_len`
/// in `crate::block_add_info::BlockAddInfo` is greater than the current
/// length, this function will call `create::helperstransfer_blockchain_from`.
fn add_block(
	blockchain: &std::sync::RwLock<blockchain::Blockchain>,
	sender: common::nodes::Node,
	package: &common::package::Package,
	config: &common::config::Config,
) -> Result<()> {
	anyhow::ensure!(config.nodes().contains(&sender), "Invalid sender.");

	let info: crate::block_add_info::BlockAddInfo =
		serde_json::from_str(package.data())
			.context("Failed to convert JSON to add info.")?;
	let mut lock = blockchain.write().unwrap();
	// Add a block and, if our blockchain is lagging, move it from another node
	if let Err(e) = lock.add_block(info.block(), false) {
		let blockchain_len =
			lock.len().context("Failed to get blockchain len.")?;
		if info.blockchain_len() > blockchain_len {
			tracing::warn!(
				"info.blockchain_len() > current blockchain length."
			);
			drop(lock);
			return crate::helpers::transfer_blockchain_from(
				sender, blockchain, config,
			)
			.with_context(|| {
				format!("Failed to transfer blockchain from {sender}.")
			});
		}
		return Err(e).context("Failed to add block.");
	}

	tracing::info!(
		"New block added: {} {} ({})",
		info.block().miner(),
		info.block()
			.compute_hash()
			.context("Failed to compute block hash.")?,
		info.block().nonce()
	);
	Ok(())
}

/// Processes user request to check and add transaction, JSON dump of which is
/// specified in `package.data()`.
fn add_transaction(
	mut stream: std::net::TcpStream,
	blockchain: &std::sync::RwLock<blockchain::Blockchain>,
	package: &common::package::Package,
	config: &common::config::Config,
) -> Result<()> {
	// Convert the dump in a package to the `Transaction`
	let transaction: blockchain::transaction::Transaction =
		match serde_json::from_str(package.data()) {
			Ok(t) => t,
			Err(e) => {
				common::package::Package::new(
					common::package::Action::AddTransactionFail,
					"invalid JSON.",
				)
				.send(config, &mut stream)
				.context(
					"Failed to send on-fail package when `serde_json` failed.",
				)?;
				return Err(e)
					.context("Failed to convert JSON to transaction");
			}
		};
	// Attempting to add a transaction to the blockchain
	if let Err(e) =
		blockchain.write().unwrap().add_transaction(transaction.clone())
	{
		common::package::Package::new(
			common::package::Action::AddTransactionFail,
			"error. Maybe invalid balance?.",
		)
		.send(config, &mut stream)
		.context("Failed to send on-fail package when addition failed.")?;
		return Err(e).context("Failed to add transaction.");
	}
	tracing::info!(
		"New transaction added: {} -> {} ({})",
		transaction.sender(),
		transaction.recipient(),
		transaction.amount()
	);

	// Sending a response about the successful addition
	common::package::Package::new(
		common::package::Action::AddTransactionSuccess,
		"",
	)
	.send(config, &mut stream)
	.context("Failed to send successful package.")?;

	// Mining a new block if there are enough transactions
	if blockchain.read().unwrap().minable() {
		crate::helpers::mine_block(blockchain, config)
			.context("Failed to mine block.")?;
	}
	Ok(())
}

/// Sends blockchain blocks in response to a user request. This only happens
/// when requested by another node, in
/// `crate::helpers::transfer_blockchain_from`.
fn get_blocks(
	mut stream: std::net::TcpStream,
	blockchain: &std::sync::RwLock<blockchain::Blockchain>,
	config: &common::config::Config,
) -> Result<()> {
	let data = blockchain
		.read()
		.unwrap()
		.to_string()
		.context("Failed to convert blockchain to string.")?;
	common::package::Package::new(
		common::package::Action::GetBlocksSuccess,
		data,
	)
	.send(config, &mut stream)
	.context("Failed to send package.")?;
	Ok(())
}

/// Processes the user's request for blockchain length.
fn get_len(
	mut stream: std::net::TcpStream,
	blockchain: &std::sync::RwLock<blockchain::Blockchain>,
	config: &common::config::Config,
) -> Result<()> {
	let len = blockchain
		.read()
		.unwrap()
		.len()
		.context("Failed to get blockchain len")?;
	common::package::Package::new(
		common::package::Action::GetBlockchainLenSuccess,
		len.to_string(),
	)
	.send(config, &mut stream)
	.context("Failed to send package.")?;
	Ok(())
}

/// Processes the user's request for blockchain last block hash.
fn get_last_block_hash(
	mut stream: std::net::TcpStream,
	blockchain: &std::sync::RwLock<blockchain::Blockchain>,
	config: &common::config::Config,
) -> Result<()> {
	let hash = blockchain
		.read()
		.unwrap()
		.get_last_block_hash()
		.context("Failed to get last block hash.")?;
	common::package::Package::new(
		common::package::Action::GetLastBlockHashSuccess,
		hash,
	)
	.send(config, &mut stream)
	.context("Failed to send package.")?;
	Ok(())
}

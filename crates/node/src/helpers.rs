use anyhow::{Context as _, Result};

/// Starts mining a new block.
#[tracing::instrument(skip(blockchain))]
pub(crate) fn mine_block(
	blockchain: &std::sync::RwLock<blockchain::Blockchain>,
	config: &common::config::Config,
) -> Result<()> {
	// In order not to interfere with other requests to `RwLock<Blockchain>`
	let mut blockchain_clone = (*blockchain.read().unwrap()).clone();

	// Mine block
	let new_block =
		blockchain_clone.mine_block().context("Failed to mine block.")?;
	// Make add info
	let len = blockchain
		.read()
		.unwrap()
		.len()
		.context("Failed to get blockchain len.")?;
	let info = crate::block_add_info::BlockAddInfo::new(&new_block, len);
	let info_json = serde_json::to_string(&info)
		.context("Failed to convert add info to JSON.")?;

	// Replace blockchain and drop some values because of move out of
	// blockchain
	drop(info);
	drop(new_block);
	*blockchain.write().unwrap() = blockchain_clone;

	// Send a new block to nodes
	tracing::info!(
		"Sending a new block to the nodes ({})...",
		config.nodes().len()
	);
	let package = common::package::Package::new(
		common::package::Action::AddBlock,
		info_json,
	);
	for node in config.nodes() {
		let mut stream = common::connect_or_continue!(node);
		common::send_package_or_continue!(config, package, &mut stream, node);
	}
	Ok(())
}

/// Needed to move the valid blockchain from a specified `node`.
#[tracing::instrument(skip(blockchain))]
pub(crate) fn transfer_blockchain_from(
	node: common::nodes::Node,
	blockchain: &std::sync::RwLock<blockchain::Blockchain>,
	config: &common::config::Config,
) -> Result<()> {
	use std::sync::atomic::Ordering;

	let mut stream =
		std::net::TcpStream::connect(node).context("Failed to connect.")?;
	// Sending request for blocks
	common::package::Package::new(common::package::Action::GetBlocks, "")
		.send(config, &mut stream)
		.context("Failed to send request.")?;
	// Receiving node blocks
	let response = common::package::Package::receive(
		config,
		&mut stream,
		Some(common::set![common::package::Action::GetBlocksSuccess]),
	)
	.context("Failed to receive a response.")?;

	// Attempting to rebuild the blockchain
	let mut lock = blockchain.write().unwrap();
	let miner = lock.miner().clone();
	let new_blockchain =
		blockchain::Blockchain::from_str(miner, response.data())
			.context("Failed to build blockchain from str.")?;
	// Last steps and blockchain replacement
	if blockchain::IS_MINING.load(Ordering::SeqCst) {
		blockchain::IS_MINING.store(false, Ordering::SeqCst);
	}
	*lock = new_blockchain;

	tracing::info!("The blockchain has been replaced from the {node}.");
	Ok(())
}

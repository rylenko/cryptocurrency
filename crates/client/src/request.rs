use anyhow::{Context as _, Result};

/// Used to request the user balance at the specified `address` for all
/// `nodes`.
#[tracing::instrument]
pub(crate) fn balance(config: &common::config::Config, address: &str) {
	let package = common::package::Package::new(
		common::package::Action::GetBalance,
		address.to_owned(),
	);
	for node in config.nodes() {
		let mut stream = common::connect_or_continue!(node);
		common::send_package_or_continue!(config, package, &mut stream, node);
		let response = common::receive_package_or_continue!(
			config,
			&mut stream,
			Some(common::set![common::package::Action::GetBalanceSuccess]),
			node,
		);
		common::nprintln!(node, "Balance: {}", response.data());
	}
}

/// Used to request the blockchain length for all `nodes`.
#[tracing::instrument]
pub(crate) fn blockchain_len(config: &common::config::Config) {
	let package = common::package::Package::new(
		common::package::Action::GetBlockchainLen,
		"",
	);
	for node in config.nodes() {
		let mut stream = common::connect_or_continue!(node);
		common::send_package_or_continue!(config, package, &mut stream, node);
		let response = common::receive_package_or_continue!(
			config,
			&mut stream,
			Some(common::set![
				common::package::Action::GetBlockchainLenSuccess
			]),
			node,
		);
		common::nprintln!(node, "Blockchain length: {}", response.data());
	}
}

/// Used to request all `nodes` to validate and add a transaction with these
/// parameters.
#[tracing::instrument]
pub(crate) fn transaction(
	config: &common::config::Config,
	user: &blockchain::user::User,
	recipient: &str,
	amount: std::num::NonZeroU64,
) -> Result<()> {
	let hash_package = common::package::Package::new(
		common::package::Action::GetLastBlockHash,
		"",
	);
	// This package will be created after we get the hash of the last block
	// from one of the nodes
	let mut transaction_package: Option<common::package::Package> = None;

	for node in config.nodes() {
		if transaction_package.is_none() {
			// Trying to connect and get the hash of the last block
			let mut stream = common::connect_or_continue!(node);
			common::send_package_or_continue!(
				config,
				hash_package,
				&mut stream,
				node
			);
			transaction_package = {
				// Getting a response with the hash of the last block
				let response = common::receive_package_or_continue!(
					config,
					&mut stream,
					Some(common::set![
						common::package::Action::GetLastBlockHashSuccess
					]),
					node,
				);
				// Creating and signing a transaction
				let mut transaction =
					blockchain::transaction::Transaction::new(
						user.address(),
						recipient,
						amount,
						response.data().to_owned(),
					);
				transaction
					.sign(user)
					.context("Failed to sign transaction.")?;
				// Creating a package with a transaction
				let data = serde_json::to_string(&transaction)
					.context("Failed to convert transaction to JSON.")?;
				Some(common::package::Package::new(
					common::package::Action::AddTransaction,
					data,
				))
			};
			tracing::debug!("Transaction package was made with {node} help.");
		}

		// Send transaction request
		let mut stream = common::connect_or_continue!(node);
		common::send_package_or_continue!(
			config,
			transaction_package.as_ref().unwrap(),
			&mut stream,
			node,
		);
		// Getting a response about the status of adding a transaction
		let response = common::receive_package_or_continue!(
			config,
			&mut stream,
			Some(common::set![
				common::package::Action::AddTransactionSuccess,
				common::package::Action::AddTransactionFail
			]),
			node,
		);
		// Display messages about the status of addition
		if response.action() == common::package::Action::AddTransactionSuccess
		{
			common::nprintln!(node, "The transaction was successfully made.");
		} else {
			common::nprintln!(
				node,
				"Failed to add transaction: {}",
				response.data()
			);
		}
	}

	Ok(())
}

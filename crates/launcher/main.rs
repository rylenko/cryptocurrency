#![deny(clippy::correctness)]
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
	clippy::missing_docs_in_private_items
)]

use anyhow::{Context as _, Result};

const EXECUTABLE_NAME_POSITION: u8 = 0;
const NODE_ADDRESS_POSITION: u8 = 1;

/// Used to pull an argument or, if it does not exist, to ask the user to
/// specify it.
#[must_use]
pub fn parse_arg(position: u8, name: &str) -> String {
	let mut iter = std::env::args().skip(1); // Skip filename
	iter.nth(position as usize).unwrap_or_else(|| {
		eprintln!("Enter the {name}.");
		std::process::exit(1);
	})
}

/// Parses the executable component name from [`std::env::args`], which is
/// either `node` or `client`.
#[inline]
#[must_use]
fn extract_executable_name_from_args() -> String {
	let name = parse_arg(EXECUTABLE_NAME_POSITION, "executable name");
	if name != "node" && name != "client" {
		eprintln!("Invalid executable name.");
		std::process::exit(1);
	}
	name
}

/// Parses node binding address from [`std::env::args`].
#[inline]
#[must_use]
fn extract_node_address_from_args() -> common::nodes::Node {
	let address_string = parse_arg(NODE_ADDRESS_POSITION, "node address");
	if let Ok(n) = address_string.parse() {
		n
	} else {
		eprintln!("Invalid address.");
		std::process::exit(1);
	}
}

/// Parses arguments for the `client` (`client::launch`) from
/// [`std::env::args`].
#[inline]
#[must_use]
fn extract_client_args_from_args() -> Vec<String> {
	let mut args: Vec<String> = std::env::args().collect();
	args.remove(1); // Remove executable name
	args
}

fn main() -> Result<()> {
	if extract_executable_name_from_args() == "client" {
		let client_args = extract_client_args_from_args();
		client::launch(client_args).context("Failed to launch the client.")?;
	} else {
		let node_address = extract_node_address_from_args();
		node::launch(node_address).context("Failed to launch the node.")?;
	}
	Ok(())
}

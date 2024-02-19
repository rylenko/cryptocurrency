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
	clippy::missing_docs_in_private_items,
	clippy::missing_errors_doc
)]

pub mod block;
pub mod blockchain;
pub mod consts;
pub mod error;
mod helpers;
mod preparing_block_state;
#[cfg(test)]
mod test_helpers;
pub mod transaction;
pub mod user;

pub use blockchain::{Blockchain, IS_MINING};

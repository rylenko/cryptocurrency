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

pub mod config;
pub mod consts;
pub mod error;
mod macros;
pub mod nodes;
pub mod package;
pub mod tracing;

use crate::error::{
	ReceivePackageBytesError, ReceivePackageError, SendPackageError,
};

/// `Package` action.
#[derive(
	Clone,
	Copy,
	Debug,
	Eq,
	Hash,
	PartialEq,
	serde::Deserialize,
	serde::Serialize,
)]
pub enum Action {
	AddBlock,
	AddTransaction,
	AddTransactionFail,
	AddTransactionSuccess,
	GetBalance,
	GetBalanceSuccess,
	GetBlockchainLen,
	GetBlockchainLenSuccess,
	GetBlocks,
	GetBlocksSuccess,
	GetLastBlockHash,
	GetLastBlockHashSuccess,
}

/// The structure that is required for each shipment. It makes it easy to
/// determine the purpose (`action`) for which some `data` are sent.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Package<'a> {
	action: Action,
	data: std::borrow::Cow<'a, str>,
}

impl<'a> Package<'a> {
	crate::accessor!(copy action -> Action);

	crate::accessor!(& data -> &str);

	#[inline]
	#[must_use = "Send a package via `self.send`."]
	pub fn new<D>(action: Action, data: D) -> Self
	where
		D: Into<std::borrow::Cow<'a, str>>,
	{
		Self { action, data: data.into() }
	}

	/// Receiving `Self` with `config.package_limits().receive_timeout()` and
	/// validates action with `accepted_actions`.
	///
	/// See also: [`send`](Package::send).
	#[tracing::instrument(
		fields(max_size = config.package_limits().max_size()),
		level = tracing::Level::DEBUG,
		skip(config, stream),
	)]
	pub fn receive(
		config: &crate::config::Config,
		stream: &mut std::net::TcpStream,
		accepted_actions: Option<std::collections::HashSet<Action>>,
	) -> Result<Self, ReceivePackageError> {
		let bytes = Self::receive_bytes(config, stream)?;
		let package: Self = serde_json::from_slice(&bytes)?;
		if let Some(aa) = accepted_actions {
			if !aa.contains(&package.action) {
				return Err(ReceivePackageError::InvalidAction);
			}
		}
		Ok(package)
	}

	/// Receiving `Self` bytes with
	/// `config.package_limits().receive_timeout_secs()` timeout.
	///
	/// See also: [`send`](Package::send).
	fn receive_bytes(
		config: &crate::config::Config,
		stream: &mut std::net::TcpStream,
	) -> Result<Box<[u8]>, ReceivePackageBytesError> {
		use std::io::Read as _;

		// Get an old and set a new timeout
		let old_timeout = stream
			.read_timeout()
			.map_err(ReceivePackageBytesError::Timeout)?;
		stream
			.set_read_timeout(Some(std::time::Duration::from_secs(
				config.package_limits().receive_timeout_secs(),
			)))
			.map_err(ReceivePackageBytesError::Timeout)?;

		// Receive a size
		let size = {
			let mut be_bytes_buffer = [0; 8];
			stream
				.read_exact(&mut be_bytes_buffer)
				.map_err(ReceivePackageBytesError::ReadLen)?;
			usize::from_be_bytes(be_bytes_buffer)
		};
		if size > config.package_limits().max_size() {
			return Err(ReceivePackageBytesError::TooBig);
		}

		// Receive a bytes
		let mut bytes_buffer = vec![0; size].into_boxed_slice();
		stream
			.read_exact(&mut bytes_buffer)
			.map_err(ReceivePackageBytesError::ReadBytes)?;

		// Set the old timeout
		stream
			.set_read_timeout(old_timeout)
			.map_err(ReceivePackageBytesError::Timeout)?;

		tracing::debug!("{size} bytes received...");
		Ok(bytes_buffer)
	}

	/// Sends `self` to [`stream`](std::net::TcpStream).
	///
	/// First it sends a data with a length of 8 bytes, which contains the
	/// length of the `self`. Then it sends the `self`'s bytes.
	#[tracing::instrument(
		fields(max_size = config.package_limits().max_size()),
		level = tracing::Level::DEBUG,
		skip(self, stream),
	)]
	pub fn send(
		&self,
		config: &crate::config::Config,
		stream: &mut std::net::TcpStream,
	) -> Result<(), SendPackageError> {
		use std::io::Write as _;

		let bytes = serde_json::to_vec(self)?;
		if bytes.len() > config.package_limits().max_size() {
			return Err(SendPackageError::TooBig);
		}
		let size_u64_be_bytes = (bytes.len() as u64).to_be_bytes();
		stream
			.write_all(&size_u64_be_bytes)
			.map_err(SendPackageError::WriteLen)?;
		stream.write_all(&bytes).map_err(SendPackageError::WriteBytes)?;

		tracing::debug!("{} bytes sent.", bytes.len());
		Ok(())
	}
}

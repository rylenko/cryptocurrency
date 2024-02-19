#![allow(clippy::module_name_repetitions)]

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoadConfigError {
	#[error("Failed to convert JSON to config.")]
	FromJson(#[from] serde_json::Error),
	#[error("Failed to validate the config.")]
	ValidateConfig(#[from] ValidateConfigError),
	#[error("Failed to read a file.")]
	Read(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ReceivePackageError {
	#[error("Failed to convert JSON to package.")]
	FromJson(#[from] serde_json::Error),
	#[error("Invalid action.")]
	InvalidAction,
	#[error("Failed to receive bytes.")]
	ReceiveBytes(#[from] ReceivePackageBytesError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ReceivePackageBytesError {
	#[error("Package is too big.")]
	TooBig,
	#[error("Failed to read a bytes.")]
	ReadBytes(#[source] std::io::Error),
	#[error("Failed to read a len.")]
	ReadLen(#[source] std::io::Error),
	#[error("Timeout error.")]
	Timeout(#[source] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SendPackageError {
	#[error("Failed to convert a package to JSON.")]
	ToJson(#[from] serde_json::Error),
	#[error("Package is too big.")]
	TooBig,
	#[error("Failed to write a bytes.")]
	WriteBytes(#[source] std::io::Error),
	#[error("Failed to write a len.")]
	WriteLen(#[source] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SetTracingSubscriberError {
	#[error("Failed to set the global default.")]
	SetGlobalDefault(#[from] tracing::subscriber::SetGlobalDefaultError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateConfigError {
	#[error("Invalid tracing level.")]
	InvalidTracingLevel,
	#[error("Package's max size limit is greater than isize::MAX.")]
	InvalidPackageMaxSizeLimit,
	#[error("The list of nodes is empty.")]
	NoNodes,
}

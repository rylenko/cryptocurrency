use crate::error::{LoadConfigError, ValidateConfigError};

trait Validate {
	fn validate(&self) -> Result<(), ValidateConfigError>;
}

#[derive(Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct Config {
	nodes: crate::nodes::Nodes,
	package_limits: PackageLimits,
	tracing: Tracing,
}

impl Config {
	crate::accessor!(& nodes -> &crate::nodes::Nodes);

	crate::accessor!(& package_limits -> &PackageLimits);

	crate::accessor!(& tracing -> &Tracing);

	/// # Params
	///
	/// `exclude_node` for nodes to exclude themselves from their list.
	#[tracing::instrument(ret)]
	pub fn load(
		exclude_node: Option<crate::nodes::Node>,
	) -> Result<Self, LoadConfigError> {
		// Read and deserialize the config file
		let content = std::fs::read(&*crate::consts::CONFIG_PATH)?;
		let mut rv: Self = serde_json::from_slice(&content)?;

		// Remove `exclude_node` node
		if let Some(exclude_node) = exclude_node {
			rv.nodes.remove(&exclude_node);
		}

		rv.validate()?;
		Ok(rv)
	}
}

impl Validate for Config {
	fn validate(&self) -> Result<(), ValidateConfigError> {
		if self.nodes.is_empty() {
			return Err(ValidateConfigError::NoNodes)?;
		}
		self.package_limits.validate()?;
		self.tracing.validate()?;
		Ok(())
	}
}

#[derive(Debug, serde::Deserialize)]
#[non_exhaustive]
pub struct PackageLimits {
	max_size: usize,
	receive_timeout_secs: u64,
}

impl PackageLimits {
	crate::accessor!(copy max_size -> usize);

	crate::accessor!(copy receive_timeout_secs -> u64);
}

impl Validate for PackageLimits {
	fn validate(&self) -> Result<(), ValidateConfigError> {
		if self.max_size > isize::MAX as usize {
			return Err(ValidateConfigError::InvalidPackageMaxSizeLimit);
		}
		Ok(())
	}
}

#[derive(Debug, serde::Deserialize)]
pub struct Tracing {
	client: TracingTarget,
	node: TracingTarget,
}

impl Tracing {
	crate::accessor!(& client -> &TracingTarget);

	crate::accessor!(& node -> &TracingTarget);
}

impl Validate for Tracing {
	fn validate(&self) -> Result<(), ValidateConfigError> {
		self.client.validate()?;
		self.node.validate()?;
		Ok(())
	}
}

#[derive(Debug, serde::Deserialize)]
pub struct TracingTarget {
	level: String,
	path: String,
}

impl TracingTarget {
	crate::accessor!(& level -> &str);

	crate::accessor!(& path -> &str);
}

impl Validate for TracingTarget {
	fn validate(&self) -> Result<(), ValidateConfigError> {
		if self.level.parse::<tracing::Level>().is_err() {
			return Err(ValidateConfigError::InvalidTracingLevel);
		}
		Ok(())
	}
}

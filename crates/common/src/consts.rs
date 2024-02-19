lazy_static::lazy_static! {
	static ref BASE_DIR: std::path::PathBuf = std::env::current_dir().unwrap();
	static ref RESOURCES_DIR: std::path::PathBuf = BASE_DIR.join("resources");
	pub(crate) static ref CONFIG_PATH: std::path::PathBuf =
		RESOURCES_DIR.join("config.json");
}

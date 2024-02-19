lazy_static::lazy_static! {
	static ref BASE_DIR: std::path::PathBuf = std::env::current_dir().unwrap();
	pub static ref RESOURCES_DIR: std::path::PathBuf = BASE_DIR.join("resources");
	pub(crate) static ref DB_PATH: std::path::PathBuf
		= RESOURCES_DIR.join("sqlite.db");
	pub(crate) static ref TEMP_DB_PATH: std::path::PathBuf =
		RESOURCES_DIR.join("temp-sqlite.db");
	pub(crate) static ref PRIVATE_KEY_PATH: std::path::PathBuf =
		RESOURCES_DIR.join("private-key");
	pub(crate) static ref PROOF_OF_WORK_DIFFICULTY_STRING: String =
		"0".repeat(PROOF_OF_WORK_DIFFICULTY as usize);
}

pub const USER_TRANSACTIONS_PER_BLOCK: u8 = 2;
pub const TRANSACTIONS_PER_BLOCK: u8 = USER_TRANSACTIONS_PER_BLOCK + 1;
pub(crate) const GENESIS_BLOCK_REWARD: u64 = 100;
#[cfg(not(test))]
pub(crate) const PROOF_OF_WORK_DIFFICULTY: u8 = 4;
#[cfg(test)]
pub(crate) const PROOF_OF_WORK_DIFFICULTY: u8 = 2;
pub(crate) const MINING_REWARD: std::num::NonZeroU64 =
	unsafe { std::num::NonZeroU64::new_unchecked(1) };

pub(crate) const STORAGE_ADDRESS: &str = "STORAGE";
pub(crate) const STORAGE_START_BALANCE: u64 = 100;
pub(crate) const STORAGE_REWARD: u64 = 1;
pub(crate) const STORAGE_REWARD_STARTING_FROM: u64 = 10;

pub(crate) const DB_CREATE_TABLE_IF_NOT_EXISTS_QUERY: &str = "
CREATE TABLE IF NOT EXISTS block (
	id INTEGER PRIMARY KEY,
	json TEXT
)
";
pub(crate) const DB_GET_COUNT_QUERY: &str = "SELECT COUNT(*) FROM block";
pub(crate) const DB_GET_ALL_QUERY: &str = "SELECT json FROM block ORDER BY id";
pub(crate) const DB_GET_LAST_QUERY: &str =
	"SELECT json FROM block ORDER BY id DESC LIMIT 1";
pub(crate) const DB_INSERT_QUERY_TEMPLATE: &str =
	"INSERT INTO block (json) VALUES (?)";

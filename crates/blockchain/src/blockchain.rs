use crate::error::{
	AddBlockError, AddBlockToDatabaseError, AddToBalanceError,
	AddTransactionError, BlockchainFromStrError, BlockchainToStringError,
	GenerateBlockProofOfWorkError, GetBalanceError,
	GetBalanceFromDatabaseError, GetBlockBeforeBlockError,
	GetBlocksCountError, GetBlocksError, GetLastBlockHashError,
	LoadOrCreateBlockchainError, MakeStorageTransactionError, MineBlockError,
	MineGenesisBlockError, NewBlockchainError, RemoveFromBalanceError,
};

pub static IS_MINING: std::sync::atomic::AtomicBool =
	std::sync::atomic::AtomicBool::new(false);
static DB_IO_LOCKED: std::sync::atomic::AtomicBool =
	std::sync::atomic::AtomicBool::new(false);

type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

/// Stores the state of the blockchain.
///
/// To load or create a `Blockchain` object, use `Self::load_or_create`.
#[derive(Clone)]
pub struct Blockchain<'a> {
	preparing_block_state:
		crate::preparing_block_state::PreparingBlockState<'a>,
	miner: crate::user::User,
	db_pool: DbPool,
}

impl<'a> Blockchain<'a> {
	common::accessor!(& miner -> &crate::user::User);

	/// Loads or creates a blockchain depending on the state of the database
	/// file in the `consts::DB_PATH` path.
	#[tracing::instrument]
	pub fn load_or_create(
		miner: crate::user::User,
	) -> Result<Self, LoadOrCreateBlockchainError> {
		let path = crate::consts::DB_PATH.as_path();
		if path.exists() {
			tracing::info!("Loading an existing database...");
		} else {
			tracing::info!("Initializing a new database...");
		}

		let pool =
			r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::file(path))?;
		Ok(Self::new(miner, pool)?)
	}

	/// Accepts a string that contains block JSONs, from which it reconstructs
	/// the blockchain.
	///
	/// Loads blocks from `s` into the temporary database at
	/// `consts::TEMP_DB_PATH`. If the integrity of the received data is
	/// confirmed, moves the temporary database to `consts::DB_PATH` and
	/// returns a new `Blockchain` object.
	#[tracing::instrument]
	pub fn from_str(
		miner: crate::user::User,
		s: &str,
	) -> Result<Self, BlockchainFromStrError> {
		use std::sync::atomic::Ordering;

		// Convert block JSONs into objects
		let blocks: Vec<crate::block::Block> = serde_json::from_str(s)?;
		// Take IO lock
		while DB_IO_LOCKED
			.compare_exchange_weak(
				false,
				true,
				Ordering::Acquire,
				Ordering::Relaxed,
			)
			.is_err()
		{
			std::hint::spin_loop();
		}
		// Creating an empty blockchain with a temporary database
		let path = crate::consts::TEMP_DB_PATH.as_path();
		let pool =
			r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::file(path))?;
		let mut rv = Self::new(miner.clone(), pool)?;
		// Transferring all blocks to the new blockchain
		for (i, block) in blocks.iter().enumerate() {
			if let Err(e) = rv.add_block(block, i == 0) {
				std::fs::remove_file(path)
					.map_err(BlockchainFromStrError::RemoveTempDb)?;
				return Err(e)?;
			}
		}
		// Replacing the temporary database with the current one
		std::fs::rename(path, crate::consts::DB_PATH.as_path())
			.map_err(BlockchainFromStrError::ReplaceDb)?;
		// Drop IO lock
		DB_IO_LOCKED.store(false, Ordering::Release);

		let rv = Self::load_or_create(miner)?;
		Ok(rv)
	}

	#[tracing::instrument(skip(db_pool))]
	fn new(
		miner: crate::user::User,
		db_pool: DbPool,
	) -> Result<Self, NewBlockchainError> {
		db_pool
			.get()?
			.execute(crate::consts::DB_CREATE_TABLE_IF_NOT_EXISTS_QUERY, [])?;
		let preparing_block_state =
			crate::preparing_block_state::PreparingBlockState::new();
		Ok(Self { preparing_block_state, miner, db_pool })
	}

	#[tracing::instrument(level = tracing::Level::DEBUG, ret, skip(self))]
	pub fn get_block_before_block(
		&self,
		before_block: &crate::block::Block,
	) -> Result<Option<crate::block::Block>, GetBlockBeforeBlockError> {
		// Getting connection and results with jsons
		let connection = self.db_pool.get()?;
		let mut statement = connection
			.prepare(crate::consts::DB_GET_ALL_QUERY)
			.map_err(GetBlockBeforeBlockError::PrepareDbStatement)?;
		let json_results = statement
			.query_map([], |row| row.get(0))
			.map_err(GetBlockBeforeBlockError::QueryDb)?;
		// Unwrap results and find previous block
		for json_result in json_results {
			let json: String = json_result
				.map_err(GetBlockBeforeBlockError::UnwrapDbResult)?;
			let block: crate::block::Block = serde_json::from_str(&json)?;
			let block_hash = block.compute_hash()?;
			if before_block
				.previous_hash()
				.map_or(false, |ph| ph == block_hash)
			{
				return Ok(Some(block));
			}
		}
		Ok(None)
	}

	/// Gets a list of all blocks and dumps them into JSONs string format.
	pub fn to_string(&self) -> Result<String, BlockchainToStringError> {
		let blocks = self.get_blocks(None)?;
		let rv = serde_json::to_string(&blocks)?;
		Ok(rv)
	}

	///  Gets the hash of the last block from the database.
	#[tracing::instrument(level = tracing::Level::DEBUG, skip(self))]
	pub fn get_last_block_hash(
		&self,
	) -> Result<String, GetLastBlockHashError> {
		let json: String = self.db_pool.get()?.query_row(
			crate::consts::DB_GET_LAST_QUERY,
			[],
			|row| row.get(0),
		)?;
		let hash = serde_json::from_str::<crate::block::Block>(&json)?
			.compute_hash()?;
		Ok(hash)
	}

	/// Is shorthand for `self.get_blocks_count`.
	#[inline]
	pub fn len(&self) -> Result<usize, GetBlocksCountError> {
		self.get_blocks_count()
	}

	#[inline]
	pub fn is_empty(&self) -> Result<bool, GetBlocksCountError> {
		Ok(self.len()? == 0)
	}

	/// Starts block mining.
	///
	/// You can stop mining for any reason by setting `false` to
	/// `IS_MINING`. This can be done, for example, in another
	/// thread.
	///
	/// # Panics
	///
	/// If `IS_MINING` is `true` or there is no pending transactions.
	#[tracing::instrument(skip(self))]
	pub fn mine_block(
		&mut self,
	) -> Result<crate::block::Block, MineBlockError> {
		use std::sync::atomic::Ordering;

		debug_assert!(!self.is_empty()?, "Mine the genesis block first.");
		debug_assert!(!self.preparing_block_state.transactions.is_empty());
		assert!(!IS_MINING.load(Ordering::Acquire));

		// To avoid immutable and mutable accesses in one moment
		self.make_storage_transaction(
			self.miner.address().to_owned(),
			crate::consts::MINING_REWARD,
		)?;

		// Getting the necessary fields
		let transactions =
			std::mem::take(&mut self.preparing_block_state.transactions);
		let previous_hash = self.get_last_block_hash()?;
		let balance_state =
			std::mem::take(&mut self.preparing_block_state.balance_state);

		// Creating the base of the block and mine
		let mut block = crate::block::Block::new(
			self.miner.address().to_owned(),
			Some(previous_hash),
			transactions,
			balance_state,
		);
		IS_MINING.store(true, Ordering::SeqCst);
		if let Err(e) = block.generate_proof_of_work() {
			// Because can't to implement `PartialEq` for `Error`
			if matches!(e, GenerateBlockProofOfWorkError::Stopped) {
				tracing::info!("Mining has been stopped.");
			}
			return Err(e)?;
		}
		IS_MINING.store(false, Ordering::SeqCst);

		// Signing and adding a block
		block.sign(&self.miner)?;
		self.add_block(&block, false)?;

		tracing::info!("Block: {block:?}");
		Ok(block)
	}

	/// Mines a genesis block by setting the initial balance to the
	/// miner in `consts::GENESIS_BLOCK_REWARD`, and the storage in
	/// `consts::STORAGE_START_BALANCE`.
	///
	/// # Panics
	///
	/// If `IS_MINING` is `true`.
	#[tracing::instrument(skip(self))]
	pub fn mine_genesis_block(
		&mut self,
	) -> Result<crate::block::Block, MineGenesisBlockError> {
		use std::sync::atomic::Ordering;

		debug_assert!(self.is_empty()?);
		assert!(!IS_MINING.load(Ordering::Acquire));

		// Creating the base balance state
		let mut state = crate::helpers::BalanceState::new();
		state.insert(
			self.miner.address().to_owned(),
			crate::consts::GENESIS_BLOCK_REWARD,
		);
		state.insert(
			crate::consts::STORAGE_ADDRESS.to_owned(),
			crate::consts::STORAGE_START_BALANCE,
		);

		// Create, mine, sign and add the block
		let mut block = crate::block::Block::new(
			self.miner.address().to_owned(),
			None::<&str>,
			crate::block::Transactions::new(),
			state,
		);
		IS_MINING.store(true, Ordering::SeqCst);
		block.generate_proof_of_work()?;
		IS_MINING.store(false, Ordering::SeqCst);

		// Signing and adding a block
		block.sign(&self.miner)?;
		self.add_block(&block, true)?;

		tracing::info!("Block: {block:?}");
		Ok(block)
	}

	#[inline]
	#[must_use]
	pub fn minable(&self) -> bool {
		self.preparing_block_state.filled()
	}

	/// Checks the integrity of the block and enters it into the database.
	#[tracing::instrument(skip(self))]
	pub fn add_block(
		&mut self,
		block: &crate::block::Block,
		is_genesis: bool,
	) -> Result<(), AddBlockError> {
		// Remove state
		self.preparing_block_state.clear();

		// Stop mining
		if IS_MINING.load(std::sync::atomic::Ordering::Relaxed) {
			IS_MINING.store(false, std::sync::atomic::Ordering::Relaxed);
		}

		if is_genesis {
			debug_assert!(self.is_empty()?);
		} else {
			block.validate_integrity(self)?;
		}
		self.add_block_to_database(block)?;
		Ok(())
	}

	/// Adds a new pending transaction to `self.preparing_block_state`.
	#[tracing::instrument(skip(self))]
	pub fn add_transaction(
		&mut self,
		transaction: crate::transaction::Transaction<'a>,
	) -> Result<(), AddTransactionError> {
		debug_assert!(!self.is_empty()?, "Mine the genesis block first.");

		// Validate transaction
		if transaction.sender() != crate::consts::STORAGE_ADDRESS
			&& self.preparing_block_state.transactions.len()
				== crate::consts::USER_TRANSACTIONS_PER_BLOCK as usize
		{
			return Err(AddTransactionError::LimitReached);
		}
		transaction.validate_integrity(self)?;

		// Withdrawal of sender costs
		let sender_costs = unsafe {
			std::num::NonZeroU64::new_unchecked(
				u64::from(transaction.amount())
					+ transaction.amount_to_storage(),
			)
		};
		self.remove_from_balance(transaction.sender(), sender_costs)?;

		// Adding profit to the recipient and the storage
		self.add_to_balance(transaction.recipient(), transaction.amount())
			.map_err(AddTransactionError::AddToRecipientBalance)?;
		if transaction.amount_to_storage() != 0 {
			let amount = unsafe {
				std::num::NonZeroU64::new_unchecked(
					transaction.amount_to_storage(),
				)
			};
			self.add_to_balance(crate::consts::STORAGE_ADDRESS, amount)
				.map_err(AddTransactionError::AddToStorageBalance)?;
		}

		self.preparing_block_state.transactions.push(transaction);
		Ok(())
	}

	/// Tries to get the user's balance from the `self.balance_state`. If it
	/// fails, it tries to get it with `self.get_balance_from_database`.
	#[tracing::instrument(level = tracing::Level::DEBUG, ret, skip(self))]
	pub fn get_balance(&self, address: &str) -> Result<u64, GetBalanceError> {
		let rv = match self.preparing_block_state.balance_state.get(address) {
			Some(b) => *b,
			None => self.get_balance_from_database(address, None)?,
		};
		Ok(rv)
	}

	/// Gets the balance from the database.
	pub(crate) fn get_balance_from_database(
		&self,
		address: &str,
		before_block: Option<&crate::block::Block>,
	) -> Result<u64, GetBalanceFromDatabaseError> {
		let mut rv = 0;
		// Obtaining blocks and reversing them (Looking for a fresh balance)
		let mut blocks = self.get_blocks(before_block)?;
		blocks.reverse();
		// Looking for balance
		for block in blocks {
			if let Some(b) = block.balance_state().get(address) {
				rv = *b;
				break;
			}
		}
		Ok(rv)
	}

	/// Gets all existing blocks using the query `consts::DB_GET_ALL_QUERY`.
	#[tracing::instrument(level = tracing::Level::TRACE, ret, skip(self))]
	fn get_blocks(
		&self,
		before_block: Option<&crate::block::Block>,
	) -> Result<Vec<crate::block::Block>, GetBlocksError> {
		let mut rv = Vec::<crate::block::Block>::new();

		// Getting connection and results with jsons
		let connection = self.db_pool.get()?;
		let mut statement = connection
			.prepare(crate::consts::DB_GET_ALL_QUERY)
			.map_err(GetBlocksError::PrepareDbStatement)?;
		let json_results = statement
			.query_map([], |row| row.get(0))
			.map_err(GetBlocksError::QueryDb)?;

		// Unwrap results and push to vector
		for json_result in json_results {
			let mut break_ = false;
			// Convert JSON into a object
			let json: String =
				json_result.map_err(GetBlocksError::UnwrapDbResult)?;
			let block: crate::block::Block = serde_json::from_str(&json)?;
			// Check if the current block was the last one
			if let Some(before_block) = before_block {
				let block_hash = block.compute_hash()?;
				if before_block
					.previous_hash()
					.map_or(false, |ph| ph == block_hash)
				{
					break_ = true;
				}
			}
			rv.push(block);
			if break_ {
				break;
			}
		}
		Ok(rv)
	}

	/// Makes a transaction on behalf of the storage.
	///
	/// # Arguments
	///
	/// `recipient` is a `String` to avoid immutable (`self.miner.address()`)
	/// and mutable (`&mut self`) accesses.
	#[tracing::instrument(ret, skip(self))]
	fn make_storage_transaction(
		&mut self,
		recipient: String,
		amount: std::num::NonZeroU64,
	) -> Result<crate::transaction::Transaction<'a>, MakeStorageTransactionError>
	{
		let transaction = crate::transaction::Transaction::new(
			crate::consts::STORAGE_ADDRESS,
			recipient,
			amount,
			self.get_last_block_hash()?,
		);
		self.add_transaction(transaction.clone())?;
		Ok(transaction)
	}

	/// Gives us the number of existing blocks in the database.
	#[tracing::instrument(level = tracing::Level::DEBUG, ret, skip(self))]
	fn get_blocks_count(&self) -> Result<usize, GetBlocksCountError> {
		let count = self.db_pool.get()?.query_row(
			crate::consts::DB_GET_COUNT_QUERY,
			[],
			|row| row.get(0),
		)?;
		Ok(count)
	}

	/// Adds a block to the database in JSON format.
	fn add_block_to_database(
		&self,
		block: &crate::block::Block,
	) -> Result<(), AddBlockToDatabaseError> {
		let json = serde_json::to_string(block)?;
		self.db_pool
			.get()?
			.execute(crate::consts::DB_INSERT_QUERY_TEMPLATE, [json])?;
		Ok(())
	}

	/// Adds `amount` to the user's current balance and enters the new
	/// balance in balance state.
	fn add_to_balance(
		&mut self,
		address: &str,
		amount: std::num::NonZeroU64,
	) -> Result<(), AddToBalanceError> {
		let balance = self.get_balance(address)?;
		self.preparing_block_state
			.balance_state
			.insert(address.to_owned(), balance + u64::from(amount));
		Ok(())
	}

	/// Removes `amount` from the user's current balance and enters a new
	/// balance in balance state.
	fn remove_from_balance(
		&mut self,
		address: &str,
		amount: std::num::NonZeroU64,
	) -> Result<(), RemoveFromBalanceError> {
		// Checking if we can withdraw that much from the balance
		let balance = self.get_balance(address)?;
		let amount_raw = u64::from(amount);
		if amount_raw > balance {
			return Err(RemoveFromBalanceError::NotEnoughMoney);
		}
		self.preparing_block_state
			.balance_state
			.insert(address.to_owned(), balance - amount_raw);
		Ok(())
	}
}

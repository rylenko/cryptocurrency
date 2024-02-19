use crate::error::{
	ComputeBlockHashError, GenerateBlockProofOfWorkError, SignBlockError,
	ValidateBlockBalanceStateError, ValidateBlockCreatedAtError,
	ValidateBlockIntegrityError, ValidateBlockIsSignedError,
	ValidateBlockMinerSignatureError, ValidateBlockPreviousHashError,
	ValidateBlockProofOfWorkError, ValidateBlockTransactionsError,
};

pub(crate) type Transactions<'a> = arrayvec::ArrayVec<
	crate::transaction::Transaction<'a>,
	{ crate::consts::TRANSACTIONS_PER_BLOCK as usize },
>;

/// The structure that represents the block, accompanied by transactions in the
/// quantity `crate::consts::TRANSACTIONS_PER_BLOCK`.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Block<'a> {
	miner: std::borrow::Cow<'a, str>,
	previous_hash: Option<std::borrow::Cow<'a, str>>,
	transactions: Transactions<'a>,
	balance_state: crate::helpers::BalanceState,
	nonce: u64,
	created_at: f64,
	miner_signature: Option<String>,
}

impl<'a> Block<'a> {
	common::accessor!(& miner -> &str);

	common::accessor!(as_deref previous_hash -> Option<&str>);

	common::accessor!(& balance_state -> &crate::helpers::BalanceState);

	common::accessor!(copy nonce -> u64);

	#[must_use = "Add block via `crate::blockchain::Blockchain`."]
	pub(crate) fn new(
		miner: impl Into<std::borrow::Cow<'a, str>>,
		previous_hash: Option<impl Into<std::borrow::Cow<'a, str>>>,
		transactions: Transactions<'a>,
		balance_state: crate::helpers::BalanceState,
	) -> Self {
		Self {
			miner: miner.into(),
			previous_hash: previous_hash.map(Into::into),
			transactions,
			balance_state,
			nonce: 0,
			created_at: crate::helpers::get_timestamp(),
			miner_signature: None,
		}
	}

	/// Simplification for calling all integrity validating functions.
	///
	/// The function is designed to validate only new blocks that have not yet
	/// been entered into the database. To find out why, see
	/// `self.validate_previous_hash` description.
	#[tracing::instrument(level = tracing::Level::DEBUG, skip(blockchain), ret)]
	pub fn validate_integrity(
		&self,
		blockchain: &crate::blockchain::Blockchain,
	) -> Result<(), ValidateBlockIntegrityError> {
		self.validate_previous_hash(blockchain)?;
		self.validate_proof_of_work()?;
		self.validate_is_signed()?;
		self.validate_miner_signature()?;
		self.validate_transactions(blockchain)?;
		self.validate_created_at(blockchain)?;
		Ok(())
	}

	/// Signs the hash of the block and puts it in `self.miner_signature`.
	#[tracing::instrument(level = tracing::Level::DEBUG, skip(miner), ret)]
	pub fn sign(
		&mut self,
		miner: &crate::user::User,
	) -> Result<(), SignBlockError> {
		let hash = self.compute_hash()?;
		let signature = miner.sign(&hash)?;
		tracing::debug!("Signature: {signature}");
		self.miner_signature = Some(signature);
		Ok(())
	}

	/// Calculates block body hash (without `self.miner_signature` and
	/// `self.proof_of_work`).
	#[tracing::instrument(level = tracing::Level::TRACE, ret)]
	pub fn compute_hash(&self) -> Result<String, ComputeBlockHashError> {
		use sha2::Digest as _;

		let json = serde_json::json!({
			"miner": self.miner,
			"previous_hash": self.previous_hash,
			"transactions": serde_json::to_string(&self.transactions)
				.map_err(ComputeBlockHashError::TransactionsToJson)?,
			"balance_state": serde_json::to_string(&self.balance_state)
				.map_err(ComputeBlockHashError::BalanceStateToJson)?,
			"nonce": self.nonce,
			"created_at": self.created_at,
		});
		let hash = sha2::Sha256::digest(json.to_string().as_bytes());
		Ok(hex::encode(hash))
	}

	/// Generates a proof of work or, in other words, starts mining. Mining
	/// and, accordingly, increasing `self.nonce` occurs until the hash
	/// contains `crate::consts::PROOF_OF_WORK_DIFFICULTY` zeros.
	///
	/// # Errors
	///
	/// `GenerateBlockProofOfWorkError::Stopped` If you set
	/// `crate::blockchain::IS_MINING` to `false` while using
	/// this function (For example, in another thread).
	///
	/// # Debug panic
	///
	/// If `crate::blockchain::IS_MINING` is `false`.
	#[tracing::instrument(level = tracing::Level::DEBUG)]
	pub(crate) fn generate_proof_of_work(
		&mut self,
	) -> Result<(), GenerateBlockProofOfWorkError> {
		use std::sync::atomic::Ordering;

		debug_assert!(crate::blockchain::IS_MINING.load(Ordering::Acquire));
		loop {
			if !crate::blockchain::IS_MINING.load(Ordering::Acquire) {
				return Err(GenerateBlockProofOfWorkError::Stopped);
			}
			match self.validate_proof_of_work() {
				Ok(()) => break,
				Err(ValidateBlockProofOfWorkError::Invalid) => {}
				Err(e) => return Err(e)?,
			}
			self.nonce += 1;
		}
		Ok(())
	}

	/// This function could find the previous block from the current block and
	/// compare `self.previous_hash` already with it. This is not implemented,
	/// because it is simply not used. This function is designed for blocks
	/// that have not yet been added to the database. That is,
	/// `self.previous_hash` is compared to the hash of the last block from the
	/// database using the `crate::blockchain::Blockchain::get_last_block_hash`
	/// method.
	fn validate_previous_hash(
		&self,
		blockchain: &crate::blockchain::Blockchain,
	) -> Result<(), ValidateBlockPreviousHashError> {
		let previous_hash = blockchain.get_last_block_hash()?;
		if self.previous_hash.as_deref().map_or(false, |h| h != previous_hash)
		{
			return Err(ValidateBlockPreviousHashError::HashesNotEquals);
		}
		Ok(())
	}

	fn validate_proof_of_work(
		&self,
	) -> Result<(), ValidateBlockProofOfWorkError> {
		let hash = self.compute_hash()?;
		if !hash.starts_with(&*crate::consts::PROOF_OF_WORK_DIFFICULTY_STRING)
		{
			return Err(ValidateBlockProofOfWorkError::Invalid);
		}
		Ok(())
	}

	fn validate_is_signed(&self) -> Result<(), ValidateBlockIsSignedError> {
		if self.miner_signature.is_none() {
			return Err(ValidateBlockIsSignedError::NotSigned);
		}
		Ok(())
	}

	/// Checks the signature via `crate::user::User::validate_signature`.
	///
	/// # Panic
	///
	/// If block is not signed.
	fn validate_miner_signature(
		&self,
	) -> Result<(), ValidateBlockMinerSignatureError> {
		let signature = self.miner_signature.as_ref().unwrap();
		let hash = self.compute_hash()?;
		crate::user::User::validate_signature(signature, &hash, &self.miner)?;
		Ok(())
	}

	/// Checks the validity of `self.created_at` in the whole plan, and then
	/// relative to the previous one.
	fn validate_created_at(
		&self,
		blockchain: &crate::blockchain::Blockchain,
	) -> Result<(), ValidateBlockCreatedAtError> {
		if crate::helpers::get_timestamp() - self.created_at < 0.0 {
			return Err(ValidateBlockCreatedAtError::InFuture);
		}
		// `Option::unwrap` because we have at least a genesis block
		let previous = blockchain.get_block_before_block(self)?.unwrap();
		if self.created_at - previous.created_at <= 0.0 {
			return Err(ValidateBlockCreatedAtError::PreviousInFuture);
		}
		Ok(())
	}

	/// Validates the integrity of `transactions`. Also uses
	/// [`validate_balance_state`](Block::validate_balance_state) and method
	/// [`validate_integrity`](Transaction::validate_integrity).
	fn validate_transactions(
		&self,
		blockchain: &crate::blockchain::Blockchain,
	) -> Result<(), ValidateBlockTransactionsError> {
		let count = self.transactions.len();
		let storage_count = self
			.transactions
			.iter()
			.filter(|t| t.sender() == crate::consts::STORAGE_ADDRESS)
			.count();

		// Validate counts
		if count != (crate::consts::USER_TRANSACTIONS_PER_BLOCK + 1) as usize {
			return Err(ValidateBlockTransactionsError::InvalidUserCount);
		} else if storage_count != 1 {
			return Err(ValidateBlockTransactionsError::InvalidStorageCount);
		}

		// Checking the uniqueness of `self.random_string'
		for i in 0..count - 1 {
			for j in i + 1..count {
				let t1 = &self.transactions[i];
				let t2 = &self.transactions[j];
				if t1.random_string() == t2.random_string() {
					return Err(
						ValidateBlockTransactionsError::RandomStringNotUnique,
					);
				}
			}
		}

		for transaction in &self.transactions {
			transaction.validate_integrity(blockchain)?;

			// Validate reward
			if transaction.sender() == crate::consts::STORAGE_ADDRESS {
				if transaction.recipient() != self.miner {
					return Err(
						ValidateBlockTransactionsError::RewardedNotMiner,
					);
				} else if transaction.amount() != crate::consts::MINING_REWARD
				{
					return Err(ValidateBlockTransactionsError::InvalidReward);
				}
			}

			// Validate previous block hash
			if !self
				.previous_hash
				.as_ref()
				.map_or(false, |h| h == transaction.previous_block_hash())
			{
				return Err(
					ValidateBlockTransactionsError::PreviousHashesNotEquals,
				);
			}
			// Validate balance state for sender and recipient
			self.validate_balance_state(transaction.sender(), blockchain)
				.map_err(
					ValidateBlockTransactionsError::ValidateSenderBalanceState,
				)?;
			self.validate_balance_state(transaction.recipient(), blockchain)
				.map_err(
				ValidateBlockTransactionsError::ValidateRecipientBalanceState,
			)?;
		}

		Ok(())
	}

	/// Calculates the balance using the data in the transactions and compares
	/// the calculations to those specified in `self.balance_state`.
	fn validate_balance_state(
		&self,
		address: &str,
		blockchain: &crate::blockchain::Blockchain,
	) -> Result<(), ValidateBlockBalanceStateError> {
		if let Some(state_balance) = self.balance_state.get(address) {
			let (mut spent, mut received) = (0, 0);
			let balance =
				blockchain.get_balance_from_database(address, Some(self))?;

			// Calculation of costs and receipts
			for transaction in &self.transactions {
				if address == transaction.sender() {
					spent += u64::from(transaction.amount())
						+ transaction.amount_to_storage();
				} else if address == transaction.recipient() {
					received += u64::from(transaction.amount());
				} else if address == crate::consts::STORAGE_ADDRESS {
					received += transaction.amount_to_storage();
				}
			}

			// Comparison of balance and expectations
			match balance.checked_add(received) {
				Some(b) => match b.checked_sub(spent) {
					Some(b) => {
						if b == *state_balance {
							Ok(())
						} else {
							Err(ValidateBlockBalanceStateError::BalancesNotEquals)
						}
					}
					None => Err(ValidateBlockBalanceStateError::SubOverflow),
				},
				None => Err(ValidateBlockBalanceStateError::AddOverflow),
			}
		} else {
			Err(ValidateBlockBalanceStateError::NoBalanceInState)
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_sign() {
		let (user, mut block) = crate::test_helpers::create_test_block();
		block.sign(&user).unwrap();
		block.validate_miner_signature().unwrap();
		Ok(())
	}

	#[test]
	fn test_generate_proof_of_work() -> Result<()> {
		let (_user, mut block) = crate::test_helpers::create_test_block();
		block.generate_proof_of_work().unwrap();
		block.validate_proof_of_work().unwrap();
		Ok(())
	}
}

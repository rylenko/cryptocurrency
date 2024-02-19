use crate::error::{
	SignTransactionError, ValidateTransactionIntegrityError,
	ValidateTransactionPreviousBlockHashError,
	ValidateTransactionRecipientError,
	ValidateTransactionSenderSignatureError,
};

/// Structure, which is the transaction of money from one user to another.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Transaction<'a> {
	sender: std::borrow::Cow<'a, str>,
	recipient: std::borrow::Cow<'a, str>,
	amount: std::num::NonZeroU64,
	amount_to_storage: u64,
	previous_block_hash: std::borrow::Cow<'a, str>,
	random_string: String,
	sender_signature: Option<String>,
}

impl<'a> Transaction<'a> {
	common::accessor!(& sender -> &str);

	common::accessor!(& recipient -> &str);

	common::accessor!(copy amount -> std::num::NonZeroU64);

	common::accessor!(copy amount_to_storage -> u64);

	common::accessor!(& previous_block_hash -> &str);

	common::accessor!(& random_string -> &str);

	#[must_use = "Add transaction via `blockchain::Blockchain`."]
	pub fn new(
		sender: impl Into<std::borrow::Cow<'a, str>>,
		recipient: impl Into<std::borrow::Cow<'a, str>>,
		amount: std::num::NonZeroU64,
		previous_block_hash: impl Into<std::borrow::Cow<'a, str>>,
	) -> Self {
		let amount_to_storage = if u64::from(amount)
			>= crate::consts::STORAGE_REWARD_STARTING_FROM
		{
			crate::consts::STORAGE_REWARD
		} else {
			0
		};
		Self {
			sender: sender.into(),
			recipient: recipient.into(),
			amount,
			amount_to_storage,
			previous_block_hash: previous_block_hash.into(),
			random_string: crate::helpers::generate_random_string(),
			sender_signature: None,
		}
	}

	/// Signs the hash of the transaction and puts it in
	/// `self.sender_signature`.
	#[tracing::instrument(level = tracing::Level::DEBUG)]
	pub fn sign(
		&mut self,
		sender: &crate::user::User,
	) -> Result<(), SignTransactionError> {
		let signature = sender.sign(&self.compute_hash())?;
		self.sender_signature = Some(signature);
		Ok(())
	}

	/// Simplification for calling all integrity validating functions.
	///
	/// Call it only if last blockchain block is previous block.
	#[tracing::instrument(level = tracing::Level::DEBUG, ret, skip(blockchain))]
	pub fn validate_integrity(
		&self,
		blockchain: &crate::blockchain::Blockchain,
	) -> Result<(), ValidateTransactionIntegrityError> {
		self.validate_recipient()?;
		self.validate_sender_signature()?;
		self.validate_previous_block_hash(blockchain)?;
		Ok(())
	}

	fn validate_recipient(
		&self,
	) -> Result<(), ValidateTransactionRecipientError> {
		if self.recipient == crate::consts::STORAGE_ADDRESS {
			return Err(ValidateTransactionRecipientError::IsStorage);
		}
		Ok(())
	}

	fn validate_sender_signature(
		&self,
	) -> Result<(), ValidateTransactionSenderSignatureError> {
		if self.sender != crate::consts::STORAGE_ADDRESS {
			if let Some(ref s) = self.sender_signature {
				let hash = self.compute_hash();
				crate::user::User::validate_signature(s, &hash, &self.sender)?;
				return Ok(());
			}
			return Err(ValidateTransactionSenderSignatureError::IsEmpty);
		}
		Ok(())
	}

	/// Call it only if last blockchain block is previous block.
	fn validate_previous_block_hash(
		&self,
		blockchain: &crate::blockchain::Blockchain,
	) -> Result<(), ValidateTransactionPreviousBlockHashError> {
		if self.previous_block_hash() != blockchain.get_last_block_hash()? {
			return Err(
				ValidateTransactionPreviousBlockHashError::HashesNotEquals,
			);
		}
		Ok(())
	}

	/// Calculated transaction body hash (without `self.sender_signature` and
	/// `self.current_hash`).
	#[tracing::instrument(level = tracing::Level::TRACE, ret)]
	#[must_use]
	fn compute_hash(&self) -> String {
		use sha2::Digest as _;
		let json = serde_json::json!({
			"sender": self.sender,
			"recipient": self.recipient,
			"amount": self.amount,
			"amount_to_storage": self.amount_to_storage,
			"previous_block_hash": self.previous_block_hash,
			"random_string": self.random_string,
		});
		let hash = sha2::Sha256::digest(json.to_string().as_bytes());
		hex::encode(hash)
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_sign() {
		let user = crate::test_helpers::create_test_user();
		let mut transaction = super::Transaction::new(
			user.address(),
			"recipient",
			unsafe { std::num::NonZeroU64::new_unchecked(50) },
			"",
		);
		transaction.sign(&user).unwrap();
		transaction.validate_sender_signature().unwrap();
	}
}

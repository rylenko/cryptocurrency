#![allow(clippy::module_name_repetitions)]

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AddBlockError {
	#[error("Failed to add block to the db.")]
	AddToDb(#[from] AddBlockToDatabaseError),
	#[error("Failed to get blocks count.")]
	GetBlocksCount(#[from] GetBlocksCountError),
	#[error("Failed to validate integrity.")]
	ValidateIntegrity(#[from] ValidateBlockIntegrityError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AddBlockToDatabaseError {
	#[error("Failed to get a connection to db.")]
	GetConnection(#[from] r2d2::Error),
	#[error("Failed to execute a query.")]
	ExecuteQuery(#[from] rusqlite::Error),
	#[error("Failed to convert block to JSON.")]
	ToJson(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AddToBalanceError {
	#[error("Failed to get balance.")]
	Get(#[from] GetBalanceError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AddTransactionError {
	#[error("Failed to add to recipient's balance.")]
	AddToRecipientBalance(#[source] AddToBalanceError),
	#[error("Failed to add to storage's balance.")]
	AddToStorageBalance(#[source] AddToBalanceError),
	#[error("Failed to get blocks count.")]
	GetBlocksCount(#[from] GetBlocksCountError),
	#[error("Limit reached.")]
	LimitReached,
	#[error("Failed to remove from balance.")]
	RemoveFromBalance(#[from] RemoveFromBalanceError),
	#[error("Failed to validate integrity.")]
	ValidateIntegrity(#[from] ValidateTransactionIntegrityError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BlockchainFromStrError {
	#[error("Failed to add block.")]
	AddBlock(#[from] AddBlockError),
	#[error("Failed to convert JSON to blocks.")]
	FromJson(#[from] serde_json::Error),
	#[error("Failed to load or create the blockchain.")]
	LoadOrCreateBlockchain(#[from] LoadOrCreateBlockchainError),
	#[error("Failed to create new blockchain.")]
	NewBlockchain(#[from] NewBlockchainError),
	#[error("Failed to create a new connections pool to db.")]
	NewDbPool(#[from] r2d2::Error),
	#[error("Failed to remove temp db.")]
	RemoveTempDb(#[source] std::io::Error),
	#[error("Failed to replace db.")]
	ReplaceDb(#[source] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BlockchainToStringError {
	#[error("Failed to get block.")]
	GetBlocks(#[from] GetBlocksError),
	#[error("Failed to convert block to json.")]
	ToJson(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ComputeBlockHashError {
	#[error("Failed to convert transactions to json.")]
	TransactionsToJson(#[source] serde_json::Error),
	#[error("Failed to convert balance state to json.")]
	BalanceStateToJson(#[source] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConvertPublicKeyToAddressError {
	#[error("Failed to convert a hex to address.")]
	FromHex(#[from] hex::FromHexError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GenerateBlockProofOfWorkError {
	#[error("Generation was stopped.")]
	Stopped,
	#[error("Failed to validate.")]
	Validate(#[from] ValidateBlockProofOfWorkError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GetBalanceError {
	#[error("Failed to get balance from database.")]
	FromDatabase(#[from] GetBalanceFromDatabaseError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GetBalanceFromDatabaseError {
	#[error("Failed to get blocks.")]
	GetBlocks(#[from] GetBlocksError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GetBlockBeforeBlockError {
	#[error("Failed to compute block hash.")]
	ComputeBlockHash(#[from] ComputeBlockHashError),
	#[error("Failed to get a connection to db.")]
	GetConnection(#[from] r2d2::Error),
	#[error("Failed to convert JSON to block.")]
	FromJson(#[from] serde_json::Error),
	#[error("Failed to prepare db statement.")]
	PrepareDbStatement(#[source] rusqlite::Error),
	#[error("Failed to query db.")]
	QueryDb(#[source] rusqlite::Error),
	#[error("Failed to unwrap db result.")]
	UnwrapDbResult(#[source] rusqlite::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GetBlocksError {
	#[error("Failed to compute block hash.")]
	ComputeBlockHash(#[from] ComputeBlockHashError),
	#[error("Failed to convert JSON to block.")]
	FromJson(#[from] serde_json::Error),
	#[error("Failed to get a connection to db.")]
	GetConnection(#[from] r2d2::Error),
	#[error("Failed to prepare db statement.")]
	PrepareDbStatement(#[source] rusqlite::Error),
	#[error("Failed to query db.")]
	QueryDb(#[source] rusqlite::Error),
	#[error("Failed to unwrap db result.")]
	UnwrapDbResult(#[source] rusqlite::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GetBlocksCountError {
	#[error("Failed to get a connection to db.")]
	GetConnection(#[from] r2d2::Error),
	#[error("Failed to query db.")]
	QueryDb(#[from] rusqlite::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GetLastBlockHashError {
	#[error("Failed to compute block hash.")]
	ComputeBlockHash(#[from] ComputeBlockHashError),
	#[error("Failed to convert JSON to block.")]
	FromJson(#[from] serde_json::Error),
	#[error("Failed to get a connection to db.")]
	GetConnection(#[from] r2d2::Error),
	#[error("Failed to query db.")]
	QueryDb(#[from] rusqlite::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoadOrCreateBlockchainError {
	#[error("Failed to make a new blockchain.")]
	New(#[from] NewBlockchainError),
	#[error("Failed to create a new connections pool to db.")]
	NewDbPool(#[from] r2d2::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LoadOrCreateUserError {
	#[error("Failed to convert bytes to user.")]
	FromBytes(#[from] k256::ecdsa::Error),
	#[error("Failed to make new user.")]
	New(#[from] NewUserError),
	#[error("Failed to read.")]
	Read(#[source] std::io::Error),
	#[error("Failed to write.")]
	Write(#[source] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MakeStorageTransactionError {
	#[error("Failed to add transaction.")]
	Add(#[from] AddTransactionError),
	#[error("Failed to get last block hash.")]
	GetLastBlockHash(#[from] GetLastBlockHashError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MineBlockError {
	#[error("Failed to add a block.")]
	AddBlock(#[from] AddBlockError),
	#[error("Failed to generate block's proof of work.")]
	GenerateBlockProofOfWork(#[from] GenerateBlockProofOfWorkError),
	#[error("Failed to get blocks count.")]
	GetBlocksCount(#[from] GetBlocksCountError),
	#[error("Failed to get last block hash.")]
	GetLastBlockHash(#[from] GetLastBlockHashError),
	#[error("Failed to make storage transaction.")]
	MakeStorageTransaction(#[from] MakeStorageTransactionError),
	#[error("Failed to sign a block.")]
	SignBlock(#[from] SignBlockError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MineGenesisBlockError {
	#[error("Failed to add a block.")]
	AddBlock(#[from] AddBlockError),
	#[error("Failed to generate block's proof of work.")]
	GenerateBlockProofOfWork(#[from] GenerateBlockProofOfWorkError),
	#[error("Failed to get blocks count.")]
	GetBlocksCount(#[from] GetBlocksCountError),
	#[error("Failed to get last block hash.")]
	GetLastBlockHash(#[from] GetLastBlockHashError),
	#[error("Failed to sign a block.")]
	SignBlock(#[from] SignBlockError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NewBlockchainError {
	#[error("Failed to get a connection to db.")]
	GetConnection(#[from] r2d2::Error),
	#[error("Failed to execute a query.")]
	ExecuteQuery(#[from] rusqlite::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NewUserError {
	#[error("Failed to convert public key to address.")]
	ConvertPublicKeyToAdress(#[from] ConvertPublicKeyToAddressError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RemoveFromBalanceError {
	#[error("Failed to get balance.")]
	Get(#[from] GetBalanceError),
	#[error("Not enough money.")]
	NotEnoughMoney,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SignBlockError {
	#[error("Failed to compute block hash.")]
	ComputeBlockHash(#[from] ComputeBlockHashError),
	#[error("Failed to use user to sign a transaction.")]
	UserSign(#[from] UserSignError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SignTransactionError {
	#[error("Failed to use user to sign a transaction.")]
	UserSign(#[from] UserSignError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateBlockBalanceStateError {
	#[error("Add overflow.")]
	AddOverflow,
	#[error("Balances are not equals.")]
	BalancesNotEquals,
	#[error("Failed to get balance from database before block.")]
	GetBalanceFromDatabase(#[from] GetBalanceFromDatabaseError),
	#[error("Not balance in state.")]
	NoBalanceInState,
	#[error("Sub overflow.")]
	SubOverflow,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateBlockCreatedAtError {
	#[error("Failed to get block before block.")]
	GetBlockBeforeBlock(#[from] GetBlockBeforeBlockError),
	#[error("Block in the future.")]
	InFuture,
	#[error("Previous block in the future.")]
	PreviousInFuture,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateBlockIntegrityError {
	#[error("Failed to validate timestamp.")]
	ValidateCreatedAt(#[from] ValidateBlockCreatedAtError),
	#[error("Failed to validate that is signed.")]
	ValidateIsSigned(#[from] ValidateBlockIsSignedError),
	#[error("Failed to validate miner signature.")]
	ValidateMinerSignature(#[from] ValidateBlockMinerSignatureError),
	#[error("Failed to validate previous hash.")]
	ValidatePreviousHash(#[from] ValidateBlockPreviousHashError),
	#[error("Failed to validate proof of work.")]
	ValidateProofOfWork(#[from] ValidateBlockProofOfWorkError),
	#[error("Failed to validate transactions.")]
	ValidateTransactions(#[from] ValidateBlockTransactionsError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateBlockIsSignedError {
	#[error("Block is not signed.")]
	NotSigned,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateBlockMinerSignatureError {
	#[error("Failed to compute hash.")]
	ComputeBlockHash(#[from] ComputeBlockHashError),
	#[error("Failed to validate signature.")]
	ValidateUserSignature(#[from] ValidateUserSignatureError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateBlockPreviousHashError {
	#[error("Failed to get last block hash.")]
	GetLastBlockHash(#[from] GetLastBlockHashError),
	#[error("Previous and last hashes are not equals.")]
	HashesNotEquals,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateBlockProofOfWorkError {
	#[error("Failed to compute block hash.")]
	ComputeBlockHash(#[from] ComputeBlockHashError),
	#[error("Invalid hash.")]
	Invalid,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateBlockTransactionsError {
	#[error("Invalid reward.")]
	InvalidReward,
	#[error("Invalid storage transactions count.")]
	InvalidStorageCount,
	#[error("Invalid user transactions count.")]
	InvalidUserCount,
	#[error("Previous hashes are not equals.")]
	PreviousHashesNotEquals,
	#[error("Random string is not unique.")]
	RandomStringNotUnique,
	#[error("Rewarded user is not a miner.")]
	RewardedNotMiner,
	#[error("Failed to validate recipient's balance state.")]
	ValidateRecipientBalanceState(#[source] ValidateBlockBalanceStateError),
	#[error("Failed to validate sender's balance state.")]
	ValidateSenderBalanceState(#[source] ValidateBlockBalanceStateError),
	#[error("Failed to validate transaction integrity.")]
	ValidateTransactionIntegrity(#[from] ValidateTransactionIntegrityError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateTransactionIntegrityError {
	#[error("Failed to vaidate previous block hash.")]
	PreviousBlockHash(#[from] ValidateTransactionPreviousBlockHashError),
	#[error("Failed to vaidate recipient.")]
	Recipient(#[from] ValidateTransactionRecipientError),
	#[error("Failed to vaidate sender signature.")]
	SenderSignature(#[from] ValidateTransactionSenderSignatureError),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateTransactionPreviousBlockHashError {
	#[error("Failed to get last block hash.")]
	GetLastBlockHash(#[from] GetLastBlockHashError),
	#[error("Hashes are not equals")]
	HashesNotEquals,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateTransactionRecipientError {
	#[error("Recipient is storage.")]
	IsStorage,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateTransactionSenderSignatureError {
	#[error(transparent)]
	Validate(#[from] ValidateUserSignatureError),
	#[error("Signature is empty.")]
	IsEmpty,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ValidateUserSignatureError {
	#[error("Addresses are not equals.")]
	AddressesNotEquals,
	#[error("Failed to convert public key to address.")]
	ConvertPublicKeyToAddress(#[from] ConvertPublicKeyToAddressError),
	#[error("Failed to convert base58 to signature.")]
	FromBase58(base58::FromBase58Error),
	#[error("Failed to parse a signature.")]
	Parse(#[source] k256::ecdsa::Error),
	#[error("Failed to recover a key.")]
	RecoverKey(#[source] k256::ecdsa::Error),
	#[error("Failed to verify a signature.")]
	Verify(#[source] k256::ecdsa::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UserSignError {
	#[error("Failed to sign.")]
	Sign(#[from] k256::ecdsa::Error),
}
